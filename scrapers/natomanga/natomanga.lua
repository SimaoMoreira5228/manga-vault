BASE_URL = "https://www.natomanga.com"

local function http_get(url, headers)
	if url == "https://" or string.match(url, "^https?://[^/]+//") then
		log.warn("[natomanga] Invalid URL provided to http_get: " .. url)
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
		log.info("[natomanga] Cloudflare detected, using Flaresolverr for: " .. url)
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
	local resp = http_get(url, { referer = BASE_URL .. "/" })
	local html = resp.text

	local cdn_json = extract_js_array(html, "cdns")
	local chapter_images_json = extract_js_array(html, "chapterImages")

	if not cdn_json or not chapter_images_json then
		local imgs = {}
		local img_elements = scraping:select_elements(html, "div.container-chapter-reader > img")
		for i, img in ipairs(img_elements) do
			table.insert(imgs, scraping:get_image_url(img) or "")
		end
		return imgs
	end

	local cdn = string.match(cdn_json, "\"(.-)\"")
	if cdn then
		cdn = string.gsub(cdn, "\\", "")
		if not string.match(cdn, "/$") then
			cdn = cdn .. "/"
		end
	else
		return {}
	end

	local imgs = {}
	for img_path in string.gmatch(chapter_images_json, "\"(.-)\"") do
		img_path = string.gsub(img_path, "\\/", "/")
		img_path = string.gsub(img_path, "^/", "")

		table.insert(imgs, cdn .. img_path)
	end

	return imgs
end

function Scrape_latest(page)
	local url = "https://www.natomanga.com/manga-list/latest-manga?page=" .. tostring(page)
	local resp = http_get(url, { referer = BASE_URL .. "/" })
	local html = resp.text

	local manga_divs =
		scraping:select_elements(html, "div.truyen-list .list-truyen-item-wrap, div.comic-list .list-comic-item-wrap")
	local manga_items = {}

	for _, manga_div_html in ipairs(manga_divs) do
		local img_elements = scraping:select_elements(manga_div_html, "a img")
		local img_url = normalize_url(scraping:get_image_url(img_elements[1]) or "")

		local title_elements = scraping:select_elements(manga_div_html, "h3 a")
		local title = scraping:get_text(title_elements[1]) or ""

		local url_elements = scraping:select_elements(manga_div_html, "h3 a")
		local url = normalize_url(scraping:get_url(url_elements[1]) or "")

		local manga_item = {
			title = title,
			img_url = img_url,
			url = url,
		}
		table.insert(manga_items, manga_item)
	end

	return manga_items
end

function Scrape_trending(page)
	local url = "https://www.natomanga.com/manga-list/hot-manga?page=" .. tostring(page)
	local resp = http_get(url, { referer = BASE_URL .. "/" })
	local html = resp.text

	local manga_divs =
		scraping:select_elements(html, "div.truyen-list .list-truyen-item-wrap, div.comic-list .list-comic-item-wrap")
	local manga_items = {}

	for _, manga_div_html in ipairs(manga_divs) do
		local img_elements = scraping:select_elements(manga_div_html, "a img")
		local img_url = normalize_url(scraping:get_image_url(img_elements[1]) or "")

		local title_elements = scraping:select_elements(manga_div_html, "h3 a")
		local title = scraping:get_text(title_elements[1]) or ""

		local url_elements = scraping:select_elements(manga_div_html, "h3 a")
		local url = normalize_url(scraping:get_url(url_elements[1]) or "")

		local manga_item = {
			title = title,
			img_url = img_url,
			url = url,
		}
		table.insert(manga_items, manga_item)
	end

	return manga_items
end

function Scrape_search(query, page)
	local url = "https://www.natomanga.com/search/story/" .. query .. "?page=" .. tostring(page)
	local resp = http_get(url, { referer = BASE_URL .. "/" })
	local html = resp.text

	local manga_divs = scraping:select_elements(
		html,
		".panel_story_list .story_item, div.truyen-list .list-truyen-item-wrap, div.comic-list .list-comic-item-wrap"
	)

	local manga_items = {}
	for _, manga_div_html in ipairs(manga_divs) do
		local img_elements = scraping:select_elements(manga_div_html, "a img")
		local img_url = normalize_url(scraping:get_image_url(img_elements[1]) or "")

		local title_elements = scraping:select_elements(manga_div_html, "h3.story_name a, h3 a")
		local title = scraping:get_text(title_elements[1]) or ""

		local url_elements = scraping:select_elements(manga_div_html, "h3.story_name a, h3 a")
		local url = normalize_url(scraping:get_url(url_elements[1]) or "")

		local manga_item = {
			title = title,
			img_url = img_url,
			url = url,
		}
		table.insert(manga_items, manga_item)
	end

	return manga_items
end

function Scrape(url)
	local resp = http_get(url, { referer = BASE_URL .. "/" })
	local html = resp.text

	local title_el = select_one_required(html, ".manga-info-content h1, .panel-story-info-right h1", "manga title")
	local title = scraping:get_text(title_el) or ""
	local img_el = scraping:select_elements(html, ".manga-info-pic img, .story-info-left span.info-image img")[1]
	local img_url = normalize_url(scraping:get_image_url(img_el) or "")
	local description = scraping:get_text(
		scraping:select_elements(html, "#noidungm, #panel-story-info-description, #contentBox")[1]
	) or ""

	local info_elements = scraping:select_elements(html, ".manga-info-text li, .story-info-right li")
	local authors = {}
	local status = ""
	local genres = {}

	for _, item_html in ipairs(info_elements) do
		local item_text = scraping:get_text(item_html)

		if string.find(item_text, "Author(s)") then
			local author_elements = scraping:select_elements(item_html, "a")
			for _, author_element in ipairs(author_elements) do
				local author_text = scraping:get_text(author_element)
				if author_text ~= "" then
					table.insert(authors, author_text)
				end
			end
		elseif string.find(item_text, "Status") then
			status = string.trim(string.gsub(item_text, "Status :", ""))
		elseif string.find(item_text, "Genres") then
			local genre_elements = scraping:select_elements(item_html, "a")
			for _, genre_element in ipairs(genre_elements) do
				table.insert(genres, scraping:get_text(genre_element) or "")
			end
		end
	end

	if #genres == 0 then
		local genres_div = scraping:select_elements(html, ".genres-wrap .genre-list a")
		for _, genre in ipairs(genres_div) do
			table.insert(genres, scraping:get_text(genre) or "")
		end
	end

	local chapters = {}
	local chapter_elements = scraping:select_elements(html, ".chapter-list .row, ul.row-content-chapter li")
	for _, chapter in ipairs(chapter_elements) do
		local chapter_title = scraping:get_text(scraping:select_elements(chapter, "a")[1]) or ""
		local chapter_url = normalize_url(scraping:get_url(scraping:select_elements(chapter, "a")[1]) or "")
		local chapter_date = scraping:get_text(scraping:select_elements(chapter, "span")[3]) or "New"
		table.insert(chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
	end

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

	local page = {
		title = title,
		url = url,
		img_url = img_url,
		genres = genres,
		alternative_names = {},
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
	local url = "https://www.natomanga.com/"
	local resp = http_get(url, { referer = BASE_URL .. "/" })
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
		id = "natomanga",
		version = "0.5.0",
		name = "NatoManga",
		img_url = "https://www.natomanga.com/images/favicon-manganato.webp",
		referer_url = "https://www.natomanga.com/",
	}
end

Tests = {
	Test_Scrape_manga = function()
		local manga = Scrape("https://www.natomanga.com/manga/solo-leveling")
		assert(manga.title == "Solo Leveling", "Title should be 'Solo Leveling'")
		assert(manga.url == "https://www.natomanga.com/manga/solo-leveling", "URL should match")
		assert(manga.img_url ~= "", "Image URL should not be empty")
		assert(#manga.genres > 0, "Genres should not be empty")
		assert(manga.status ~= "", "Status should not be empty")
		assert(manga.description ~= "", "Description should not be empty")
	end,

	Test_Scrape_chapter = function()
		local images = Scrape_chapter("https://www.natomanga.com/manga/solo-leveling/chapter-1")
		assert(#images > 0, "There should be images in the chapter")
		for _, img_url in ipairs(images) do
			assert(string.match(img_url, "^https?://"), "Image URL should be valid: " .. img_url)
		end
	end,

	Test_Scrape_latest = function()
		local mangas = Scrape_latest(1)
		assert(#mangas > 0, "There should be mangas in the latest list")
		for _, manga in ipairs(mangas) do
			assert(manga.title ~= "", "Manga title should not be empty")
			assert(manga.url ~= "", "Manga URL should not be empty")
			assert(manga.img_url ~= "", "Manga image URL should not be empty")
		end
	end,

	Test_Scrape_trending = function()
		local mangas = Scrape_trending(1)
		assert(#mangas > 0, "There should be mangas in the trending list")
		for _, manga in ipairs(mangas) do
			assert(manga.title ~= "", "Manga title should not be empty")
			assert(manga.url ~= "", "Manga URL should not be empty")
			assert(manga.img_url ~= "", "Manga image URL should not be empty")
		end
	end,

	Test_Scrape_search = function()
		local mangas = Scrape_search("solo", 1)
		assert(#mangas > 0, "There should be mangas in the search results")
		for _, manga in ipairs(mangas) do
			assert(manga.title ~= "", "Manga title should not be empty")
			assert(manga.url ~= "", "Manga URL should not be empty")
			assert(manga.img_url ~= "", "Manga image URL should not be empty")
		end
	end,

	Test_Scrape_genres_list = function()
		local genres = Scrape_genres_list()
		assert(#genres > 0, "There should be genres in the genres list")
		for _, genre in ipairs(genres) do
			assert(genre.name ~= "", "Genre name should not be empty")
			assert(genre.url ~= "", "Genre URL should not be empty")
		end
	end,
}
