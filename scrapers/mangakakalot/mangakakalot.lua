BASE_URL = "https://www.mangakakalot.gg"

local function http_get(url, headers)
	if url == "https://" or string.match(url, "^https?://[^/]+//") then
		log.warn("[mangakakalot] Invalid URL provided to http_get: " .. url)
		return { ok = false, text = "", status = 0, headers = {} }
	end

	headers = headers or { Referer = BASE_URL .. "/" }
	local response = http:get(url, headers)

	if not response.ok then
		if response.status == 404 then
			utils.raise_error("not_found", "Page not found: " .. url, false)
		else
			utils.raise_error("network", "HTTP error " .. tostring(response.status) .. " for: " .. url, true)
		end
	end

	if
		flaresolverr:using_flaresolverr() and http:has_cloudflare_protection(response.text, response.status, response.headers)
	then
		log.info("[mangakakalot] Cloudflare detected, using Flaresolverr for: " .. url)
		response = flaresolverr:get(url)
		if not response.ok then
			utils.raise_error(
				response.error and response.error.kind or "cloudflare",
				response.error and response.error.message or "Flaresolverr failed",
				true
			)
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

local function select_one_required(html, selector, label)
	local res = scraping:try_select_element(html, selector)
	if not res.ok then
		utils.raise_error("parse", "Failed to select element for " .. label .. ": " .. selector, false)
	end
	if res.value == nil then
		utils.raise_error("parse", "Missing required element for " .. label .. ": '" .. selector .. "'", false)
	end
	return tostring(res.value)
end

local function select_many(html, selector, label)
	local res = scraping:try_select_elements(html, selector)
	if not res.ok then
		utils.raise_error("parse", "Failed to select elements for " .. label .. ": " .. selector, false)
	end
	return res.value or {}
end

local function extract_js_array(html, name)
	local s = string.match(html, name .. "%s*=%s*(%[.-%])")
	if s then
		return s
	end
	return ""
end

function Scrape_chapter(url)
	local response = http_get(url)
	local html = response.text

	local imgs = {}

	local cdn_json = extract_js_array(html, "cdns") or extract_js_array(html, "backupImage")
	local chapter_images_json = extract_js_array(html, "chapterImages") or extract_js_array(html, "chapter_images")

	if cdn_json and chapter_images_json then
		local cdn = string.match(cdn_json, "\"(.-)\"") or string.match(cdn_json, "'(.-)'")
		if cdn then
			cdn = string.gsub(cdn, "\\", "")
			for img_path in string.gmatch(chapter_images_json, "\"(.-)\"") do
				img_path = string.gsub(img_path, "\\/", "/")
				local full = cdn
				if not string.match(full, "/$") then
					full = full .. "/"
				end
				table.insert(imgs, full .. img_path)
			end
			if #imgs > 0 then
				return imgs
			end
		end
	end

	local image_elements =
		select_many(html, "div.vung-doc img, div.container-chapter-reader img, .reader-img img", "chapter images")
	for _, img_element in ipairs(image_elements) do
		local img_url = scraping:get_image_url(img_element)
		if img_url and img_url ~= "" then
			table.insert(imgs, img_url)
		end
	end

	if #imgs == 0 then
		utils.raise_error("parse", "No images found in chapter: " .. url, false)
	end

	return imgs
end

local function scrape_manga_list(url)
	local response = http_get(url)
	local html = response.text
	local manga_divs =
		scraping:select_elements(html, "div.truyen-list > div.list-truyen-item-wrap, div.comic-list > .list-comic-item-wrap")
	local manga_items = {}

	for _, manga_div_html in ipairs(manga_divs) do
		local url_element = scraping:select_elements(manga_div_html, "h3 a")[1]
		local url = url_element and normalize_url(scraping:get_url(url_element)) or ""
		local title = url_element and scraping:get_text(url_element) or ""
		local img_element = scraping:select_elements(manga_div_html, "img")[1]
		local img_url = img_element and normalize_url(scraping:get_image_url(img_element)) or ""

		if url ~= "" then
			table.insert(manga_items, {
				title = title,
				img_url = img_url,
				url = url,
			})
		end
	end
	return manga_items
end

function Scrape_latest(page)
	local url = BASE_URL .. "/manga-list/latest-manga?page=" .. tostring(page)
	return scrape_manga_list(url)
end

function Scrape_trending(page)
	local url = BASE_URL .. "/manga-list/hot-manga?page=" .. tostring(page)
	return scrape_manga_list(url)
end

function Scrape_search(query, page)
	local url = BASE_URL .. "/search/story/" .. string.gsub(query, " ", "_") .. "?page=" .. tostring(page)
	local response = http_get(url)
	local html = response.text

	local manga_divs =
		scraping:select_elements(html, ".panel_story_list .story_item, div.list-truyen-item-wrap, div.list-comic-item-wrap")
	local manga_items = {}
	for _, manga_div_html in ipairs(manga_divs) do
		local url_element = scraping:select_elements(manga_div_html, "h3 a")[1]
		local url = url_element and normalize_url(scraping:get_url(url_element)) or ""
		local title = url_element and scraping:get_text(url_element) or ""
		local img_element = scraping:select_elements(manga_div_html, "img")[1]
		local img_url = img_element and normalize_url(scraping:get_image_url(img_element)) or ""

		if url ~= "" then
			table.insert(manga_items, {
				title = title,
				img_url = img_url,
				url = url,
			})
		end
	end
	return manga_items
end

function Scrape(url)
	local response = http_get(url)
	local html = response.text

	local main_info_element = select_one_required(html, "div.manga-info-top, div.panel-story-info", "manga main info")

	local title_element = select_one_required(main_info_element, "h1, h2", "manga title")
	local title = scraping:get_text(title_element) or ""

	local img_element = scraping:select_elements(html, "div.manga-info-pic img, span.info-image img")[1]
	local img_url = img_element and scraping:get_image_url(img_element) or ""

	local desc_element = scraping:select_elements(html, "div#noidungm, div#panel-story-info-description, div#contentBox")[1]
	local description = desc_element and scraping:get_text(desc_element) or ""

	local genres = {}
	local authors = {}
	local status = ""
	local alternative_names = {}

	local info_elements = scraping:select_elements(main_info_element, "li, .variations-tableInfo tr")
	for _, item in ipairs(info_elements) do
		local item_text = scraping:get_text(item)
		if string.find(item_text, "Author(s)") then
			local author_elements = scraping:select_elements(item, "a")
			for _, author_element in ipairs(author_elements) do
				table.insert(authors, scraping:get_text(author_element))
			end
		elseif string.find(item_text, "Status") then
			status = string.trim(string.gsub(item_text, "Status :", ""))
		elseif string.find(item_text, "Alternative") then
			local alt_name_text = string.trim(string.gsub(item_text, "Alternative :", ""))
			local names = string.split(alt_name_text, "[;,]")
			for _, name in ipairs(names) do
				local trimmed_name = string.trim(name)
				if trimmed_name ~= "" then
					table.insert(alternative_names, trimmed_name)
				end
			end
		elseif string.find(item_text, "Genres") then
			local genre_elements = scraping:select_elements(item, "a")
			for _, genre_element in ipairs(genre_elements) do
				table.insert(genres, scraping:get_text(genre_element))
			end
		end
	end

	local chapters = {}
	local chapter_elements = scraping:select_elements(html, "div.chapter-list div.row, ul.row-content-chapter li")
	for _, chapter_html in ipairs(chapter_elements) do
		local link_element = scraping:select_elements(chapter_html, "a")[1]
		local chapter_title = link_element and scraping:get_text(link_element) or ""
		local chapter_url = link_element and normalize_url(scraping:get_url(link_element)) or ""
		local date_element = scraping:select_elements(chapter_html, "span")[3]
		local chapter_date = date_element and scraping:get_text(date_element) or ""

		if chapter_url ~= "" then
			table.insert(chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
		end
	end
	table.reverse(chapters)

	local book_slug = string.match(html, "var bookSlug = \"([^\"]+)\"") or string.match(html, "data%-bookslug=\"([^\"]+)\"")
	if book_slug and string.find(html, "show-more-chapters") then
		local api_page = 1
		while true do
			local api_url = BASE_URL .. "/api/manga/" .. book_slug .. "/chapters?page=" .. tostring(api_page)
			local api_resp = http_get(api_url)
			if not api_resp.ok then
				break
			end
			local ok, body = pcall(api_resp.json, api_resp)
			if not ok or not body or not body.data or not body.data.chapters then
				break
			end
			for _, ch in ipairs(body.data.chapters) do
				local chapter_url = normalize_url(BASE_URL .. "/manga/" .. book_slug .. "/" .. (ch.chapter_slug or ""))
				table.insert(chapters, { title = ch.chapter_name or "", url = chapter_url, date = ch.updated_at or "" })
			end
			if not body.data.pagination or not body.data.pagination.has_more then
				break
			end
			api_page = api_page + 1
		end
		table.reverse(chapters)
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

function Scrape_genres_list()
	local url = BASE_URL
	local resp = http_get(url)
	local html = resp.text

	local genres = {}
	local genre_rows = scraping:select_elements(html, ".panel-category table tr")
	for i = 3, #genre_rows do
		local genre_elements = scraping:select_elements(genre_rows[i], "a")
		for _, genre_element in ipairs(genre_elements) do
			local name = scraping:get_text(genre_element) or ""
			local genre_url = normalize_url(scraping:get_url(genre_element) or "")
			if name ~= "" and genre_url ~= "" then
				table.insert(genres, { name = name, url = genre_url })
			end
		end
	end
	return genres
end

function Get_info()
	return {
		id = "mangakakalotgg",
		version = "0.5.0",
		name = "MangaKakalot",
		img_url = BASE_URL .. "/images/favicon.ico",
		referer_url = BASE_URL .. "/",
	}
end

Tests = {
	Test_Scrape_manga = function()
		local manga = Scrape("https://www.mangakakalot.gg/manga/solo-leveling")
		assert(manga.title == "Solo Leveling", "Manga title mismatch")
		assert(manga.url == "https://www.mangakakalot.gg/manga/solo-leveling", "Manga URL mismatch")
		assert(manga.img_url ~= "", "Manga image URL is empty")
		assert(#manga.genres > 0, "No genres found")
		assert(manga.status ~= "", "Manga status is empty")
		assert(manga.description ~= "", "Manga description is empty")
		assert(#manga.chapters > 0, "No chapters found")
	end,

	Test_Scrape_chapter = function()
		local images = Scrape_chapter("https://www.mangakakalot.gg/manga/solo-leveling/chapter-200")
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
