BASE_URL = "https://mangak.io"

local LEGACY_URLS = {
	"https://mangabuddy.com/",
	"https://www.mangabuddy.com/",
}

local function http_get(url, headers)
	if url == "https://" or string.match(url, "^https?://[^/]+//") then
		log.warn("[mangabuddy] Invalid URL provided to http_get: " .. url)
		return { text = "", status = 0, headers = {}, ok = false }
	end

	headers = headers or { Referer = BASE_URL .. "/" }
	local response = http:get(url, headers)

	if not response.ok then
		if response.status == 404 then
			log.warn("[mangabuddy] URL not found: " .. url)
			utils.raise_error("not_found", "Page not found: " .. url, false)
		else
			log.error("[mangabuddy] HTTP request failed: " .. (response.error and response.error.message or "Unknown error"))
			error(response.error)
		end
	end

	if
		flaresolverr:using_flaresolverr() and http:has_cloudflare_protection(response.text, response.status, response.headers)
	then
		log.info("[mangabuddy] Cloudflare protection detected, using Flaresolverr")
		response = flaresolverr:get(url)
		if not response.ok then
			log.error(
				"[mangabuddy] Flaresolverr request failed: " .. (response.error and response.error.message or "Unknown error")
			)
			error(response.error)
		end
	end

	return response
end

local function normalize_url(url)
	url = url or ""
	if url == "" then
		return ""
	end
	if string.match(url, "^https?://") then
		return url
	end
	if string.sub(url, 1, 1) == "/" then
		return BASE_URL .. url
	end
	return BASE_URL .. "/" .. url
end

local function extract_next_data_page_props(html)
	if not html or html == "" then
		return nil
	end

	local next_data_json = string.match(html, "<script[^>]-id=\"__NEXT_DATA__\"[^>]*>(.-)</script>")
		or string.match(html, "<script[^>]-id='__NEXT_DATA__'[^>]*>(.-)</script>")

	if not next_data_json or next_data_json == "" then
		return nil
	end

	local decoded, err = utils.json_parse(next_data_json)
	if err ~= nil then
		log.warn("[mangabuddy] Failed to parse __NEXT_DATA__ JSON: " .. tostring(err))
		return nil
	end

	if type(decoded) ~= "table" or type(decoded.props) ~= "table" or type(decoded.props.pageProps) ~= "table" then
		return nil
	end

	return decoded.props.pageProps
end

local function parse_named_values_from_table(items)
	if type(items) ~= "table" then
		return {}
	end

	local out = {}
	for _, value in ipairs(items) do
		if type(value) == "string" then
			local cleaned = string.trim(value)
			if cleaned ~= "" then
				table.insert(out, cleaned)
			end
		elseif type(value) == "table" and type(value.name) == "string" then
			local cleaned = string.trim(value.name)
			if cleaned ~= "" then
				table.insert(out, cleaned)
			end
		end
	end

	return out
end

local function parse_chapters_from_table(items)
	if type(items) ~= "table" then
		return {}
	end

	local chapters = {}
	for _, chapter in ipairs(items) do
		if type(chapter) == "table" then
			local chapter_url = normalize_url(chapter.url or "")
			local chapter_title = chapter.name or chapter.title or ""
			local chapter_date = chapter.updatedAt or chapter.date or ""
			if chapter_url ~= "" and chapter_title ~= "" then
				table.insert(chapters, {
					title = chapter_title,
					url = chapter_url,
					date = chapter_date,
				})
			end
		end
	end

	return chapters
end

local function json_unescape(s)
	if not s or s == "" then
		return ""
	end

	s = string.gsub(s, "\\/", "/")
	s = string.gsub(s, "\\\"", "\"")
	s = string.gsub(s, "\\n", "\n")
	s = string.gsub(s, "\\r", "\r")
	s = string.gsub(s, "\\t", "\t")
	s = string.gsub(s, "\\u003c", "<")
	s = string.gsub(s, "\\u003e", ">")
	s = string.gsub(s, "\\u0026", "&")
	s = string.gsub(s, "\\u0027", "'")
	return s
end

local function extract_balanced_block(text, start_pos, open_char, close_char)
	if not text or start_pos < 1 or start_pos > #text then
		return nil
	end
	if string.sub(text, start_pos, start_pos) ~= open_char then
		return nil
	end

	local depth = 0
	local in_string = false
	local escaped = false

	for i = start_pos, #text do
		local ch = string.sub(text, i, i)
		if in_string then
			if escaped then
				escaped = false
			elseif ch == "\\" then
				escaped = true
			elseif ch == "\"" then
				in_string = false
			end
		else
			if ch == "\"" then
				in_string = true
			elseif ch == open_char then
				depth = depth + 1
			elseif ch == close_char then
				depth = depth - 1
				if depth == 0 then
					return string.sub(text, start_pos, i)
				end
			end
		end
	end

	return nil
end

local function extract_json_block_after_key(text, key, open_char, close_char)
	local token = "\"" .. key .. "\""
	local key_pos = string.find(text, token, 1, true)
	if not key_pos then
		return nil
	end

	local block_start = string.find(text, open_char, key_pos + #token, true)
	if not block_start then
		return nil
	end

	return extract_balanced_block(text, block_start, open_char, close_char)
end

local function extract_json_string_field(text, key)
	local token = "\"" .. key .. "\""
	local key_pos = string.find(text, token, 1, true)
	if not key_pos then
		return ""
	end

	local quote_start = string.find(text, "\"", key_pos + #token)
	if not quote_start then
		return ""
	end

	local i = quote_start + 1
	local escaped = false
	while i <= #text do
		local ch = string.sub(text, i, i)
		if escaped then
			escaped = false
		elseif ch == "\\" then
			escaped = true
		elseif ch == "\"" then
			return json_unescape(string.sub(text, quote_start + 1, i - 1))
		end
		i = i + 1
	end

	return ""
end

local function extract_top_level_objects(array_block)
	local objects = {}
	if not array_block or #array_block < 2 then
		return objects
	end

	local depth = 0
	local in_string = false
	local escaped = false
	local obj_start = nil

	for i = 2, (#array_block - 1) do
		local ch = string.sub(array_block, i, i)
		if in_string then
			if escaped then
				escaped = false
			elseif ch == "\\" then
				escaped = true
			elseif ch == "\"" then
				in_string = false
			end
		else
			if ch == "\"" then
				in_string = true
			elseif ch == "{" then
				if depth == 0 then
					obj_start = i
				end
				depth = depth + 1
			elseif ch == "}" then
				depth = depth - 1
				if depth == 0 and obj_start then
					table.insert(objects, string.sub(array_block, obj_start, i))
					obj_start = nil
				end
			end
		end
	end

	return objects
end

local function extract_json_string_array(array_block)
	local values = {}
	if not array_block or #array_block < 2 then
		return values
	end

	local i = 2
	while i < #array_block do
		local quote_start = string.find(array_block, "\"", i, true)
		if not quote_start then
			break
		end

		local j = quote_start + 1
		local escaped = false
		while j <= #array_block do
			local ch = string.sub(array_block, j, j)
			if escaped then
				escaped = false
			elseif ch == "\\" then
				escaped = true
			elseif ch == "\"" then
				table.insert(values, json_unescape(string.sub(array_block, quote_start + 1, j - 1)))
				break
			end
			j = j + 1
		end

		i = j + 1
	end

	return values
end

local function parse_named_objects_array(parent_obj, key)
	local arr = extract_json_block_after_key(parent_obj, key, "[", "]")
	if not arr then
		return {}
	end

	local names = {}
	for _, obj in ipairs(extract_top_level_objects(arr)) do
		local name = string.trim(extract_json_string_field(obj, "name"))
		if name ~= "" then
			table.insert(names, name)
		end
	end
	return names
end

local function parse_chapters_from_initial_manga(parent_obj)
	local arr = extract_json_block_after_key(parent_obj, "chapters", "[", "]")
	if not arr then
		return {}
	end

	local chapters = {}
	for _, chapter_obj in ipairs(extract_top_level_objects(arr)) do
		local title = extract_json_string_field(chapter_obj, "name")
		local chapter_url = normalize_url(extract_json_string_field(chapter_obj, "url"))
		local chapter_date = extract_json_string_field(chapter_obj, "updatedAt")
		if chapter_date == "" then
			chapter_date = extract_json_string_field(chapter_obj, "date")
		end

		if title ~= "" and chapter_url ~= "" then
			table.insert(chapters, { title = title, url = chapter_url, date = chapter_date })
		end
	end

	return chapters
end

local function parse_items_from_next_data(html)
	local page_props = extract_next_data_page_props(html)
	if page_props then
		local keys = { "items", "ssrItems", "initialItems" }
		for _, key in ipairs(keys) do
			local source = page_props[key]
			if type(source) == "table" then
				local items = {}
				for _, entry in ipairs(source) do
					if type(entry) == "table" then
						local url = entry.url or ""
						local title = entry.name or entry.title or ""
						local img_url = entry.cover or entry.image or ""
						if url ~= "" and title ~= "" and not string.find(url, "/chapter-", 1, true) then
							table.insert(items, {
								title = title,
								img_url = img_url,
								url = normalize_url(url),
							})
						end
					end
				end

				if #items > 0 then
					return items
				end
			end
		end
	end

	local keys = { "items", "ssrItems", "initialItems" }
	for _, key in ipairs(keys) do
		local arr = extract_json_block_after_key(html, key, "[", "]")
		if arr then
			local items = {}
			for _, obj in ipairs(extract_top_level_objects(arr)) do
				local url = extract_json_string_field(obj, "url")
				local title = extract_json_string_field(obj, "name")
				local img_url = extract_json_string_field(obj, "cover")
				if url ~= "" and title ~= "" and not string.find(url, "/chapter-", 1, true) then
					table.insert(items, {
						title = title,
						img_url = img_url,
						url = normalize_url(url),
					})
				end
			end

			if #items > 0 then
				return items
			end
		end
	end

	return {}
end

function Scrape_chapter(url)
	local request = http_get(url)
	local html = request.text

	local page_props = extract_next_data_page_props(html)
	if page_props and type(page_props.initialChapter) == "table" and type(page_props.initialChapter.images) == "table" then
		local images = {}
		for _, image in ipairs(page_props.initialChapter.images) do
			if type(image) == "string" and image ~= "" then
				table.insert(images, image)
			end
		end
		if #images > 0 then
			return images
		end
	end

	local initial_chapter_obj = extract_json_block_after_key(html, "initialChapter", "{", "}")
	if initial_chapter_obj then
		local images_array = extract_json_block_after_key(initial_chapter_obj, "images", "[", "]")
		if images_array then
			local images = extract_json_string_array(images_array)
			if #images > 0 then
				return images
			end
		end
	end

	local imgs = {}
	local image_elements = scraping:select_elements(html, "#images img, #images source, img[data-src]") or {}
	for _, element_html in ipairs(image_elements) do
		local image_url = scraping:get_image_url(element_html) or ""
		if image_url == "" then
			local srcset = element_html:match("srcset%s*=%s*\"([^\"]+)\"") or element_html:match("srcset%s*=%s*'([^']+)'")
			if srcset then
				local first_src = srcset:match("(%S+)")
				image_url = first_src or ""
			end
		end
		if image_url ~= "" then
			table.insert(imgs, image_url)
		end
	end

	if #imgs == 0 then
		log.warn("[mangabuddy] No images found for chapter: " .. tostring(url))
		utils.raise_error("parse", "No images found in chapter", false)
	end

	return imgs
end

function Scrape_manga_list(url)
	local request = http_get(url)
	local html = request.text

	local items = parse_items_from_next_data(html)
	if #items > 0 then
		return items
	end

	local function looks_like_manga_path(path)
		if not path or path == "" then
			return false
		end
		if not string.match(path, "^/[^/?#]+$") then
			return false
		end
		local reserved = {
			home = true,
			latest = true,
			ranking = true,
			search = true,
			filters = true,
			community = true,
			contact = true,
			["feature-requests"] = true,
			["privacy-policy"] = true,
			["terms-of-service"] = true,
			dmca = true,
			["api-docs"] = true,
			sitemap = true,
		}
		local slug = string.sub(path, 2)
		if reserved[slug] then
			return false
		end
		return true
	end

	local function title_from_slug(path)
		local slug = string.sub(path or "", 2)
		if slug == "" then
			return ""
		end
		local words = {}
		for part in string.gmatch(slug, "[^%-]+") do
			local w = string.sub(part, 1, 1):upper() .. string.sub(part, 2)
			table.insert(words, w)
		end
		return table.concat(words, " ")
	end

	local map = {}
	local order = {}

	local anchor_elements = scraping:select_elements(html, "a[href]") or {}
	for _, anchor_html in ipairs(anchor_elements) do
		local raw_href = scraping:get_url(anchor_html) or ""
		local href_path = raw_href
		if string.match(href_path, "^https?://") then
			href_path = string.match(href_path, "^https?://[^/]+(/[^?#]*)") or ""
		end

		local manga_path = ""
		if string.find(href_path, "/chapter-", 1, true) then
			manga_path = string.sub(href_path, 1, string.find(href_path, "/chapter-", 1, true) - 1)
		elseif looks_like_manga_path(href_path) then
			manga_path = href_path
		end

		if manga_path ~= "" and looks_like_manga_path(manga_path) then
			local manga_url = normalize_url(manga_path)
			if not map[manga_url] then
				map[manga_url] = { title = "", img_url = "", url = manga_url }
				table.insert(order, manga_url)
			end

			local title = string.trim(scraping:get_text(anchor_html) or "")
			if title ~= "" and map[manga_url].title == "" then
				map[manga_url].title = title
			end

			local img_url = scraping:get_image_url(anchor_html) or ""
			if img_url ~= "" and map[manga_url].img_url == "" then
				map[manga_url].img_url = img_url
			end
		end
	end

	local manga_items = {}
	for _, manga_url in ipairs(order) do
		local item = map[manga_url]
		if item.title == "" then
			local path = string.match(manga_url, "^https?://[^/]+(/[^?#]*)") or ""
			item.title = title_from_slug(path)
		end
		table.insert(manga_items, item)
	end

	return manga_items
end

function Scrape_latest(page)
	local url = BASE_URL .. "/latest?page=" .. tostring(page)
	return Scrape_manga_list(url)
end

function Scrape_trending(page)
	local url = BASE_URL .. "/ranking?page=" .. tostring(page)
	return Scrape_manga_list(url)
end

function Scrape_search(query, page)
	local url = BASE_URL .. "/search?q=" .. http:url_encode(query) .. "&sort=latest&page=" .. tostring(page)
	return Scrape_manga_list(url)
end

function Scrape(url)
	local request = http_get(url)
	local html = request.text

	local page_props = extract_next_data_page_props(html)
	if page_props and type(page_props.initialManga) == "table" then
		local manga = page_props.initialManga
		local status = "Unknown"
		local status_text = tostring(manga.status or "")
		local lowered = string.lower(status_text)
		if lowered == "ongoing" then
			status = "Ongoing"
		elseif lowered == "completed" then
			status = "Completed"
		end

		local chapters = parse_chapters_from_table(manga.chapters)
		if #chapters > 0 then
			chapters = table.reverse(chapters)
		end

		return {
			title = manga.name or manga.title or "",
			url = url,
			img_url = manga.cover or manga.image or "",
			genres = parse_named_values_from_table(manga.genres),
			alternative_names = parse_named_values_from_table(manga.altNames),
			authors = parse_named_values_from_table(manga.authors),
			artists = parse_named_values_from_table(manga.artists),
			status = status,
			manga_type = "",
			release_date = "",
			description = manga.summary or manga.description or "",
			chapters = chapters,
		}
	end

	local initial_manga_obj = extract_json_block_after_key(html, "initialManga", "{", "}")
	if initial_manga_obj then
		local title = extract_json_string_field(initial_manga_obj, "name")
		local img_url = extract_json_string_field(initial_manga_obj, "cover")
		local description = extract_json_string_field(initial_manga_obj, "summary")
		local genres = parse_named_objects_array(initial_manga_obj, "genres")
		local authors = parse_named_objects_array(initial_manga_obj, "authors")
		local alternative_names = parse_named_objects_array(initial_manga_obj, "altNames")
		local status_text = extract_json_string_field(initial_manga_obj, "status")
		local chapters = parse_chapters_from_initial_manga(initial_manga_obj)

		local status = "Unknown"
		local lowered = string.lower(status_text)
		if lowered == "ongoing" then
			status = "Ongoing"
		elseif lowered == "completed" then
			status = "Completed"
		end

		if #chapters > 0 then
			chapters = table.reverse(chapters)
		end

		return {
			title = title,
			url = url,
			img_url = img_url,
			genres = genres,
			alternative_names = alternative_names,
			authors = authors,
			artists = {},
			status = status,
			manga_type = "",
			release_date = "",
			description = description,
			chapters = chapters,
		}
	end

	local title_el = scraping:select_element(html, "h1")
	local title = title_el and (scraping:get_text(title_el) or "") or ""
	local cover_res = scraping:select_element(html, "img[src*='/covers/']")
	local img_url = cover_res and (scraping:get_image_url(cover_res) or "") or ""
	local desc_el = scraping:select_element(html, "h2 + p")
	local description = desc_el and (scraping:get_text(desc_el) or "") or ""

	local genres = {}
	for _, genre_html in ipairs(scraping:select_elements(html, "a[href*='/genres/']") or {}) do
		local name = string.trim(scraping:get_text(genre_html) or "")
		if name ~= "" then
			table.insert(genres, name)
		end
	end

	local authors = {}
	for _, author_html in ipairs(scraping:select_elements(html, "a[href*='/authors/']") or {}) do
		local name = string.trim(scraping:get_text(author_html) or "")
		if name ~= "" then
			table.insert(authors, name)
		end
	end

	local chapters = {}
	for _, chapter_html in ipairs(scraping:select_elements(html, "a[data-chapter-row='true']") or {}) do
		local chapter_url = normalize_url(scraping:get_url(chapter_html) or "")
		local chapter_title = string.trim(scraping:get_text(chapter_html) or "")
		if chapter_url ~= "" then
			table.insert(chapters, { title = chapter_title, url = chapter_url, date = "" })
		end
	end

	return {
		title = title,
		url = url,
		img_url = img_url,
		genres = genres,
		alternative_names = {},
		authors = authors,
		artists = {},
		status = "Unknown",
		manga_type = "",
		release_date = "",
		description = description,
		chapters = table.reverse(chapters),
	}
end

function Scrape_genres_list()
	local request = http_get(BASE_URL .. "/filters")
	local html = request.text
	local genres = {}

	local page_props = extract_next_data_page_props(html)
	if page_props and type(page_props.initialGenres) == "table" then
		for _, genre in ipairs(page_props.initialGenres) do
			if type(genre) == "table" then
				local name = string.trim(genre.name or "")
				local slug = string.trim(genre.slug or "")
				if name ~= "" and slug ~= "" then
					table.insert(genres, { name = name, url = BASE_URL .. "/genres/" .. slug })
				end
			end
		end
		if #genres > 0 then
			return genres
		end
	end

	local initial_genres = extract_json_block_after_key(html, "initialGenres", "[", "]")
	if initial_genres then
		for _, genre_obj in ipairs(extract_top_level_objects(initial_genres)) do
			local name = string.trim(extract_json_string_field(genre_obj, "name"))
			local slug = string.trim(extract_json_string_field(genre_obj, "slug"))
			if name ~= "" and slug ~= "" then
				table.insert(genres, { name = name, url = BASE_URL .. "/genres/" .. slug })
			end
		end
	end

	if #genres > 0 then
		return genres
	end

	for _, genre_html in ipairs(scraping:select_elements(html, "a[href*='/genres/']") or {}) do
		local name = string.trim(scraping:get_text(genre_html) or "")
		local genre_url = normalize_url(scraping:get_url(genre_html) or "")
		if name ~= "" and genre_url ~= "" then
			table.insert(genres, { name = name, url = genre_url })
		end
	end

	return genres
end

function Get_info()
	return {
		id = "mangabuddy",
		version = "0.6.0",
		name = "MangaK",
		img_url = "https://mangak.io/static/sites/mangak/icons/favicon.ico",
		referer_url = "https://mangak.io/",
		base_url = "https://mangak.io/home",
		legacy_urls = LEGACY_URLS,
	}
end

Tests = {
	Test_Scrape_manga = function()
		local manga = Scrape("https://mangak.io/i-became-the-first-prince-legend-of-swords-song")
		assert(manga.title ~= "", "Manga title is empty")
		assert(manga.url == "https://mangak.io/i-became-the-first-prince-legend-of-swords-song", "Manga URL mismatch")
		assert(manga.img_url ~= "", "Manga image URL is empty")
		assert(#manga.genres > 0, "No genres found")
		assert(#manga.authors > 0, "No authors found")
		assert(manga.status ~= "", "Manga status is empty")
		assert(manga.description ~= "", "Manga description is empty")
		assert(#manga.chapters > 0, "No chapters found")
	end,

	Test_Scrape_chapter = function()
		local images = Scrape_chapter("https://mangak.io/i-became-the-first-prince-legend-of-swords-song/chapter-37")
		assert(#images > 0, "No images found")
	end,

	Test_Scrape_latest = function()
		local mangas = Scrape_latest(1)
		assert(#mangas > 0, "No mangas found in latest")
	end,

	Test_Scrape_trending = function()
		local mangas = Scrape_trending(1)
		assert(#mangas > 0, "No mangas found in trending")
	end,

	Test_Scrape_search = function()
		local mangas = Scrape_search("love", 1)
		assert(#mangas > 0, "No mangas found in search")
	end,

	Test_Scrape_genres_list = function()
		local genres = Scrape_genres_list()
		assert(#genres > 0, "No genres found")
	end,
}
