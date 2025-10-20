BASE_URL = "https://www.mangakakalot.gg"

local function http_get(url, headers)
	if url == "https://" or string.match(url, "^https?://[^/]+//") then
		print("[mangakakalot] Invalid URL provided to http_get: " .. url)
		return { text = "", status = 0, headers = {} }
	end

	headers = headers or {}
	headers.referer = BASE_URL .. "/"
	local response = http:get(url, headers)

	if
		flaresolverr:using_flaresolverr() and http:has_cloudflare_protection(response.text, response.status, response.headers)
	then
		response = flaresolverr:get(url)
	end

	return response.text
end

function Scrape_chapter(url)
	local html = http_get(url)

	local cdn_json = string.match(html, "var cdns = (%[.-%]);") or string.match(html, "var backupImage = (%[.-%]);")
	local chapter_images_json = string.match(html, "var chapterImages = (%[.-%]);")

	if cdn_json and chapter_images_json then
		local cdn = string.match(cdn_json, "\"(.-)\"")
		if cdn then
			cdn = string.gsub(cdn, "\\", "")
			local imgs = {}
			for img_path in string.gmatch(chapter_images_json, "\"(.-)\"") do
				img_path = string.gsub(img_path, "\\/", "/")
				if not string.match(cdn, "/$") then
					cdn = cdn .. "/"
				end
				table.insert(imgs, cdn .. img_path)
			end
			if #imgs > 0 then
				return imgs
			end
		end
	end

	local imgs = {}
	local image_elements = scraping:select_elements(html, "div.vung-doc img, div.container-chapter-reader img")
	for _, img_element in ipairs(image_elements) do
		local img_url = scraping:get_image_url(img_element)
		if img_url then
			table.insert(imgs, img_url)
		end
	end

	return imgs
end

local function scrape_manga_list(url)
	local html = http_get(url)
	local manga_divs =
		scraping:select_elements(html, "div.truyen-list > div.list-truyen-item-wrap, div.comic-list > .list-comic-item-wrap")
	local manga_items = {}

	for _, manga_div_html in ipairs(manga_divs) do
		local url_element = scraping:select_elements(manga_div_html, "h3 a")[1]
		local url = url_element and scraping:get_url(url_element) or ""
		local title = url_element and scraping:get_text(url_element) or ""
		local img_element = scraping:select_elements(manga_div_html, "img")[1]
		local img_url = img_element and scraping:get_image_url(img_element) or ""

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
	local html = http_get(url)

	local manga_divs =
		scraping:select_elements(html, ".panel_story_list .story_item, div.list-truyen-item-wrap, div.list-comic-item-wrap")
	local manga_items = {}
	for _, manga_div_html in ipairs(manga_divs) do
		local url_element = scraping:select_elements(manga_div_html, "h3 a")[1]
		local url = url_element and scraping:get_url(url_element) or ""
		local title = url_element and scraping:get_text(url_element) or ""
		local img_element = scraping:select_elements(manga_div_html, "img")[1]
		local img_url = img_element and scraping:get_image_url(img_element) or ""

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

function Scrape_manga(url)
	local html = http_get(url)

	local main_info_element = scraping:select_elements(html, "div.manga-info-top, div.panel-story-info")[1]
	if not main_info_element then
		return {}
	end

	local title_element = scraping:select_elements(main_info_element, "h1, h2")[1]
	local title = title_element and scraping:get_text(title_element) or ""

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
		local chapter_url = link_element and scraping:get_url(link_element) or ""
		local date_element = scraping:select_elements(chapter_html, "span")[3]
		local chapter_date = date_element and scraping:get_text(date_element) or ""

		if chapter_url ~= "" then
			table.insert(chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
		end
	end
	table.reverse(chapters)

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
	local html = http_get(url)

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
		version = "0.4.2",
		name = "MangaKakalot",
		img_url = BASE_URL .. "/images/favicon.ico",
		referer_url = BASE_URL .. "/",
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
