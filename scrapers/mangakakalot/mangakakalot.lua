FLARESOLVERR_SESSION_ID = nil
GET_REQUEST_TIMEOUT = 2000

local function get_or_create_flaresolverr_session()
	if FLARESOLVERR_SESSION_ID == nil then
		FLARESOLVERR_SESSION_ID = flaresolverr:create_session()
	end
	return FLARESOLVERR_SESSION_ID
end

local function http_get(url, headers)
	headers = headers or {}
	local request = http:get(url, headers)
	local html = request.text

	if flaresolverr:using_flaresolverr() and http:has_cloudflare_protection(html, request.status, request.headers) then
		local req = flaresolverr:get(url, get_or_create_flaresolverr_session())
		if req.status == 200 then
			html = req.text
		end
	elseif not flaresolverr:using_flaresolverr() and http:has_cloudflare_protection(html, request.status, request.headers) then
		utils.sleep(GET_REQUEST_TIMEOUT)
		request = http:get(url, headers)
		html = request.text
	end

	return html
end

function Scrape_chapter(url)
	local html = http_get(url, { referer = "https://www.mangakakalot.gg/" })

	local cdn_json = string.match(html, "var cdns = (%[.-%]);")
	local chapter_images_json = string.match(html, "var chapterImages = (%[.-%]);")

	if cdn_json and chapter_images_json then
		local cdn = string.match(cdn_json, "\"(.-)\"")
		if cdn then
			cdn = string.gsub(cdn, "\\", "")
		else
			return {}
		end

		local imgs = {}
		for img_path in string.gmatch(chapter_images_json, "\"(.-)\"") do
			img_path = string.gsub(img_path, "\\/", "/")

			if not string.match(cdn, "/$") then
				cdn = cdn .. "/"
			end

			table.insert(imgs, cdn .. img_path)
		end
		return imgs
	end

	local imgs = {}
	local image_elements = scraping:select_elements(html, "div.container-chapter-reader > img")
	for _, img_element in ipairs(image_elements) do
		local img_url = scraping:get_image_url(img_element)
		if img_url then
			table.insert(imgs, img_url)
		end
	end

	return imgs
end

function Scrape_latest(page)
	local url = "https://www.mangakakalot.gg/manga-list/latest-manga?page=" .. tostring(page)
	local html = http_get(url, { referer = "https://www.mangakakalot.gg/" })

	local manga_divs = scraping:select_elements(html, "div.list-truyen-item-wrap")
	local manga_items = {}

	for _, manga_div_html in ipairs(manga_divs) do
		local img_elements = scraping:select_elements(manga_div_html, "a.list-story-item img")
		local img_url = scraping:get_image_url(img_elements[1]) or ""

		local title_elements = scraping:select_elements(manga_div_html, "h3 a")
		local title = scraping:get_text(title_elements[1]) or ""

		local url_elements = scraping:select_elements(manga_div_html, "h3 a")
		local url = scraping:get_url(url_elements[1]) or ""

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
	local url = "https://www.mangakakalot.gg/manga-list/hot-manga?page=" .. tostring(page)
	local html = http_get(url, { referer = "https://www.mangakakalot.gg/" })

	local manga_divs = scraping:select_elements(html, "div.list-truyen-item-wrap")
	local manga_items = {}

	for _, manga_div_html in ipairs(manga_divs) do
		local img_elements = scraping:select_elements(manga_div_html, "a.list-story-item img")
		local img_url = scraping:get_image_url(img_elements[1]) or ""

		local title_elements = scraping:select_elements(manga_div_html, "h3 a")
		local title = scraping:get_text(title_elements[1]) or ""

		local url_elements = scraping:select_elements(manga_div_html, "h3 a")
		local url = scraping:get_url(url_elements[1]) or ""

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
	local url = "https://www.mangakakalot.gg/search/story/" .. query .. "?page=" .. tostring(page)
	local html = http_get(url, { referer = "https://www.mangakakalot.gg/" })

	local manga_divs = scraping:select_elements(html, "div.story_item")

	local manga_items = {}
	for _, manga_div_html in ipairs(manga_divs) do
		local img_elements = scraping:select_elements(manga_div_html, "a img")
		local img_url = scraping:get_image_url(img_elements[1]) or ""

		local title_elements = scraping:select_elements(manga_div_html, "h3.story_name a")
		local title = scraping:get_text(title_elements[1]) or ""

		local url_elements = scraping:select_elements(manga_div_html, "h3.story_name a")
		local url = scraping:get_url(url_elements[1]) or ""

		local manga_item = {
			title = title,
			img_url = img_url,
			url = url,
		}
		table.insert(manga_items, manga_item)
	end

	return manga_items
end

function Scrape_manga(url)
	local html = http_get(url, { referer = "https://www.mangakakalot.gg/" })

	local title_element = scraping:select_elements(html, ".manga-info-top h1, .panel-story-info h1, .panel-story-info h2")[1]
	local title = title_element and scraping:get_text(title_element) or ""

	local img_element = scraping:select_elements(html, ".manga-info-pic img, .info-image img")[1]
	local img_url = img_element and scraping:get_image_url(img_element) or ""

	local desc_element = scraping:select_elements(html, "#panel-story-info-description, #noidungm, #contentBox")[1]
	local description = desc_element and scraping:get_text(desc_element) or ""

	local genres = {}
	local authors = {}
	local status = ""
	local alternative_names = {}

	local info_elements = scraping:select_elements(html, ".manga-info-text li, .variations-tableInfo tr")
	for _, item in ipairs(info_elements) do
		local item_text = scraping:get_text(item)
		if string.find(item_text, "Author(s)") then
			local author_elements = scraping:select_elements(item, "a")
			for _, author_element in ipairs(author_elements) do
				local author_text = scraping:get_text(author_element)
				if author_text ~= "" then
					table.insert(authors, author_text)
				end
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
				table.insert(genres, scraping:get_text(genre_element) or "")
			end
		end
	end

	local chapters = {}
	local chapter_elements = scraping:select_elements(html, ".row-content-chapter li, .chapter-list .row")
	for _, chapter in ipairs(chapter_elements) do
		local title_element = scraping:select_elements(chapter, "a")[1]
		local chapter_title = title_element and scraping:get_text(title_element) or ""
		local chapter_url = title_element and scraping:get_url(title_element) or ""

		local date_elements = scraping:select_elements(chapter, "span")
		local chapter_date = #date_elements > 0 and scraping:get_text(date_elements[#date_elements]) or "New"

		if chapter_url ~= "" then
			table.insert(chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
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
	local url = "https://www.mangakakalot.gg/"
	local html = http_get(url, { referer = "https://www.mangakakalot.gg/" })

	local genres = {}
	local genre_rows = scraping:select_elements(html, ".panel-category table tr")
	for i = 3, #genre_rows do
		local genre_elements = scraping:select_elements(genre_rows[i], "a")
		for _, genre_element in ipairs(genre_elements) do
			local name = scraping:get_text(genre_element) or ""
			local genre_url = scraping:get_url(genre_element) or ""
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
		version = "0.3.1",
		name = "MangaKakalot",
		img_url = "https://www.mangakakalot.gg/images/favicon.ico",
		referer_url = "https://www.mangakakalot.gg/",
	}
end

Tests = {
	Test_Scrape_manga = function()
		local manga = Scrape_manga("https://www.mangakakalot.gg/manga/solo-leveling")
		assert(manga.title == "Solo Leveling", "Manga title mismatch")
		assert(manga.url == "https://www.mangakakalot.gg/manga/solo-leveling", "Manga URL mismatch")
		assert(manga.img_url ~= "", "Manga image URL is empty")
		assert(#manga.genres > 0, "No genres found")
		assert(manga.status ~= "", "Manga status is empty")
		assert(manga.description ~= "", "Manga description is empty")
		assert(#manga.chapters > 0, "No chapters found")
	end,

	Test_Scrape_chapter = function()
		local images = Scrape_chapter("https://www.mangakakalot.gg/manga/solo-leveling/chapter-202")
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
