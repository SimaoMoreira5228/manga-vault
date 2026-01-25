BASE_URL = "https://mangabuddy.com"

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

---@return string
local function select_one_required(html, selector, label)
	local res = scraping:try_select_element(html, selector)
	if not res.ok then
		utils.raise_error(
			"parse",
			"Invalid selector for " .. label .. ": '" .. selector .. "' - " .. (res.error and res.error.message or ""),
			false
		)
	end

	if res.value == nil then
		utils.raise_error("parse", "Missing required element for " .. label .. ": '" .. selector .. "'", false)
	end

	return tostring(res.value)
end

local function select_many(html, selector, label)
	local res = scraping:try_select_elements(html, selector)
	if not res.ok then
		utils.raise_error(
			"parse",
			"Invalid selector for " .. label .. ": '" .. selector .. "' - " .. (res.error and res.error.message or ""),
			false
		)
	end
	return res.value or {}
end

local function extract_js_string_var(html, name)
	local token_single = "var " .. name .. " = '"
	if string.contains(html, token_single) then
		local after = string.substring_after(html, token_single)
		return string.substring_before(after, "'")
	end

	local token_double = "var " .. name .. " = \""
	if string.contains(html, token_double) then
		local after = string.substring_after(html, token_double)
		return string.substring_before(after, "\"")
	end

	return ""
end

function Scrape_chapter(url)
	local request = http_get(url)
	local html = request.text
	local imgs = {}

	local main_server = extract_js_string_var(html, "mainServer")
	local chap_images_str = extract_js_string_var(html, "chapImages")

	local chapter_images_from_html = select_many(html, "#chapter-images img, .chapter-image[data-src]", "chapter images")

	if main_server ~= "" and chap_images_str ~= "" then
		local scheme_prefix = string.starts_with(main_server, "//") and "https:" or ""
		local main_prefix = scheme_prefix .. main_server
		local chap_images = string.split(chap_images_str, ",")

		for _, raw in ipairs(chap_images) do
			local img = string.trim(raw)
			if img ~= "" then
				if string.starts_with(img, "http://") or string.starts_with(img, "https://") then
					table.insert(imgs, img)
				else
					table.insert(imgs, main_prefix .. img)
				end
			end
		end

		if #imgs > 0 then
			return imgs
		end
	end

	if chap_images_str ~= "" then
		local chapter_images_from_js = string.split(chap_images_str, ",")
		local all_absolute = true
		local cleaned = {}
		for _, raw in ipairs(chapter_images_from_js) do
			local img = string.trim(raw)
			if img ~= "" then
				if string.starts_with(img, "http://") or string.starts_with(img, "https://") then
					table.insert(cleaned, img)
				else
					all_absolute = false
					break
				end
			end
		end

		if all_absolute and (#chapter_images_from_html < #cleaned) then
			return cleaned
		end
	end

	for _, img_html in ipairs(chapter_images_from_html) do
		local img_url = scraping:get_image_url(img_html)
		if img_url and img_url ~= "" then
			table.insert(imgs, img_url)
		end
	end

	if #imgs == 0 then
		log.warn("[mangabuddy] No images found for chapter: " .. url)
		utils.raise_error("parse", "No images found in chapter", false)
	end

	return imgs
end

function Scrape_manga_list(url)
	local request = http_get(url)
	local html = request.text

	local manga_divs = select_many(html, "div.book-detailed-item", "manga list")
	local manga_items = {}

	for _, manga_div_html in ipairs(manga_divs) do
		local link_res = scraping:try_select_element(manga_div_html, "h3 a")
		if not link_res.ok then
			utils.raise_error(
				"parse",
				"Invalid selector in manga list: 'h3 a' - " .. (link_res.error and link_res.error.message or ""),
				false
			)
		end

		local link_element = link_res.value and tostring(link_res.value) or ""
		if link_element == "" then
			goto continue
		end

		local title = scraping:get_text(link_element) or ""
		local manga_url = normalize_url(scraping:get_url(link_element) or "")

		local img_res = scraping:try_select_element(manga_div_html, ".thumb img")
		if not img_res.ok then
			utils.raise_error(
				"parse",
				"Invalid selector in manga list: '.thumb img' - " .. (img_res.error and img_res.error.message or ""),
				false
			)
		end
		local img_url = img_res.value and (scraping:get_image_url(tostring(img_res.value)) or "") or ""

		if manga_url ~= "" then
			table.insert(manga_items, {
				title = title,
				img_url = img_url,
				url = manga_url,
			})
		end

		::continue::
	end
	return manga_items
end

function Scrape_latest(page)
	local url = BASE_URL .. "/search?sort=updated_at&page=" .. tostring(page)
	return Scrape_manga_list(url)
end

function Scrape_trending(page)
	local url = BASE_URL .. "/search?sort=views&page=" .. tostring(page)
	return Scrape_manga_list(url)
end

function Scrape_search(query, page)
	local url = BASE_URL .. "/search?q=" .. http:url_encode(query) .. "&page=" .. tostring(page)
	return Scrape_manga_list(url)
end

function Scrape(url)
	local request = http_get(url)
	local html = request.text

	local title = scraping:get_text(select_one_required(html, ".detail h1", "manga title")) or ""

	local cover_res = scraping:try_select_element(html, "#cover img")
	if not cover_res.ok then
		utils.raise_error(
			"parse",
			"Invalid selector for cover: '#cover img' - " .. (cover_res.error and cover_res.error.message or ""),
			false
		)
	end
	local img_url = cover_res.value and (scraping:get_image_url(tostring(cover_res.value)) or "") or ""

	local description_parts = {}
	local description_elements = select_many(html, ".summary .content, .summary .content ~ p", "manga description")
	for _, element_html in ipairs(description_elements) do
		local t = scraping:get_text(element_html) or ""
		if t ~= "" then
			table.insert(description_parts, t)
		end
	end
	local description = table.concat(description_parts, "\n")

	local genres = {}
	local genre_elements = select_many(html, ".detail .meta p a[href*='/genres/']", "genres")
	for _, genre_html in ipairs(genre_elements) do
		local trimmed_genre = string.trim(scraping:get_text(genre_html) or "")
		local genre = string.gsub(trimmed_genre, ",", "")
		table.insert(genres, genre)
	end

	local authors = {}
	local author_elements = select_many(html, ".detail .meta p a[href*='/authors/']", "authors")
	for _, author_html in ipairs(author_elements) do
		table.insert(authors, scraping:get_text(author_html) or "")
	end

	local status_res = scraping:try_select_element(html, ".detail .meta p a[href*='/status/']")
	if not status_res.ok then
		utils.raise_error(
			"parse",
			"Invalid selector for status: '.detail .meta p a[href*=/status/]' - "
				.. (status_res.error and status_res.error.message or ""),
			false
		)
	end
	local status_text = status_res.value and (scraping:get_text(tostring(status_res.value)) or ""):lower() or ""
	local status = "Unknown"
	if status_text == "ongoing" then
		status = "Ongoing"
	elseif status_text == "completed" then
		status = "Completed"
	end

	local alt_names_header = scraping:select_element(html, ".detail h2")
	local alternative_names = {}
	if alt_names_header then
		local alt_text = scraping:get_text(alt_names_header)
		for name in string.gmatch(alt_text, "([^,;]+)") do
			local trimmed_name = string.trim(name)
			if trimmed_name ~= "" and trimmed_name ~= title then
				table.insert(alternative_names, trimmed_name)
			end
		end
	end
	if #alternative_names > 0 then
		description = description .. "\n\nAlternative names: " .. table.concat(alternative_names, ", ")
	end

	local function parse_chapters_list(chapters_html)
		local chapters = {}
		local chapter_elements = select_many(chapters_html, "#chapter-list > li", "chapter list")

		for _, chapter_html in ipairs(chapter_elements) do
			local link_el = scraping:select_element(chapter_html, "a")
			if link_el then
				local chapter_url = normalize_url(scraping:get_url(link_el) or "")
				local title_el = scraping:select_element(chapter_html, ".chapter-title")
				local date_el = scraping:select_element(chapter_html, ".chapter-update")
				local chapter_title = title_el and (scraping:get_text(title_el) or "") or (scraping:get_text(link_el) or "")
				local chapter_date = date_el and (scraping:get_text(date_el) or "") or ""

				table.insert(chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
			end
		end

		return chapters
	end

	local initial_chapters = parse_chapters_list(html)
	local chapters = initial_chapters

	local book_slug = string.match(html, "var bookSlug = \"([^\"]+)\"")
	if book_slug and scraping:select_element(html, "div#show-more-chapters") then
		local api_url = BASE_URL .. "/api/manga/" .. book_slug .. "/chapters?source=detail"
		local api_request = http_get(api_url)
		local api_html = api_request.text
		local api_chapters = parse_chapters_list(api_html)

		if #api_chapters > 0 then
			local api_urls = {}
			for _, ch in ipairs(api_chapters) do
				api_urls[ch.url] = true
			end

			local cut_index = #initial_chapters + 1
			for i, ch in ipairs(initial_chapters) do
				if api_urls[ch.url] then
					cut_index = i
					break
				end
			end

			local merged = {}
			for i = 1, (cut_index - 1) do
				table.insert(merged, initial_chapters[i])
			end
			for _, ch in ipairs(api_chapters) do
				table.insert(merged, ch)
			end
			chapters = merged
		end
	end

	local page = {
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
		chapters = table.reverse(chapters),
	}
	return page
end

function Scrape_genres_list()
	local url = BASE_URL .. "/home"
	local request = http_get(url)
	local html = request.text
	local genres = {}
	local genre_elements = select_many(html, "ul.genres__wrapper.clearfix a", "genres list")
	for _, genre_element in ipairs(genre_elements) do
		local name = scraping:get_text(genre_element) or ""
		local genre_url = scraping:get_url(genre_element) or ""
		if name ~= "" and genre_url ~= "" then
			table.insert(genres, { name = name, url = genre_url })
		end
	end
	return genres
end

function Get_info()
	return {
		id = "mangabuddy",
		version = "0.4.1",
		name = "MangaBuddy",
		img_url = "https://mangabuddy.com/favicon.ico",
		referer_url = "https://mangabuddy.com/",
	}
end

Tests = {
	Test_Scrape_manga = function()
		local manga = Scrape("https://mangabuddy.com/solo-leveling")
		assert(manga.title == "Solo Leveling", "Manga title mismatch")
		assert(manga.url == "https://mangabuddy.com/solo-leveling", "Manga URL mismatch")
		assert(manga.img_url ~= "", "Manga image URL is empty")
		assert(#manga.genres > 0, "No genres found")
		assert(#manga.authors > 0, "No authors found")
		assert(manga.status ~= "", "Manga status is empty")
		assert(manga.description ~= "", "Manga description is empty")
		assert(#manga.chapters > 0, "No chapters found")
	end,

	Test_Scrape_chapter = function()
		local images = Scrape_chapter("https://mangabuddy.com/solo-leveling/chapter-1")
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
		local mangas = Scrape_search("nano", 1)
		assert(#mangas > 0, "No mangas found in search")
	end,

	Test_Scrape_genres_list = function()
		local genres = Scrape_genres_list()
		assert(#genres > 0, "No genres found")
	end,
}
