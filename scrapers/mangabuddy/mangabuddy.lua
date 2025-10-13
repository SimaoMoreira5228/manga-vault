BASE_URL = "https://mangabuddy.com"

local function make_request(url)
	local options = { headers = { Referer = BASE_URL .. "/" } }
	return http:get(url, options)
end

function Scrape_chapter(url)
	local request = make_request(url)
	local html = request.text
	local imgs = {}

	local main_server = string.match(html, "var mainServer = \"([^\"]+)\"")
	local chap_images_str = string.match(html, "var chapImages = '([^']+)'")

	if main_server and chap_images_str then
		local scheme_prefix = string.match(main_server, "^//") and "https:" or ""
		for img_path in string.gmatch(chap_images_str, "([^,]+)") do
			table.insert(imgs, scheme_prefix .. main_server .. img_path)
		end
		return imgs
	end

	local image_elements = scraping:select_elements(html, "#chapter-images .chapter-image img, .chapter-image[data-src]")
	for _, img_html in ipairs(image_elements) do
		local img_url = scraping:get_image_url(img_html)
		if img_url then
			table.insert(imgs, img_url)
		end
	end
	return imgs
end

function Scrape_manga_list(url)
	local request = make_request(url)
	local html = request.text

	local manga_divs = scraping:select_elements(html, "div.book-detailed-item")
	local manga_items = {}

	for _, manga_div_html in ipairs(manga_divs) do
		local link_element = scraping:select_element(manga_div_html, "h3 a") or ""
		local title = scraping:get_text(link_element) or ""
		local manga_url = scraping:get_url(link_element) or ""

		local img_element = scraping:select_element(manga_div_html, ".thumb img")
		local img_url = scraping:get_image_url(img_element or "") or ""

		if manga_url ~= "" then
			table.insert(manga_items, {
				title = title,
				img_url = img_url,
				url = manga_url,
			})
		end
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

function Scrape_manga(url)
	local request = make_request(url)
	local html = request.text

	local title = scraping:get_text(scraping:select_element(html, ".detail h1") or "") or ""
	local img_url = scraping:get_image_url(scraping:select_element(html, "#cover img") or "") or ""
	local description = scraping:get_text(scraping:select_element(html, ".summary .content") or "") or ""

	local genres = {}
	local genre_elements = scraping:select_elements(html, ".detail .meta p a[href*='/genres/']")
	for _, genre_html in ipairs(genre_elements) do
		local trimmed_genre = string.trim(scraping:get_text(genre_html) or "")
		local genre = string.gsub(trimmed_genre, ",", "")
		table.insert(genres, genre)
	end

	local authors = {}
	local author_elements = scraping:select_elements(html, ".detail .meta p a[href*='/authors/']")
	for _, author_html in ipairs(author_elements) do
		table.insert(authors, scraping:get_text(author_html) or "")
	end

	local status_text = scraping:get_text(scraping:select_element(html, ".detail .meta p a[href*='/status/']") or ""):lower()
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

	local chapters = {}
	local initial_chapter_elements = scraping:select_elements(html, "#chapter-list > li")
	for _, chapter_html in ipairs(initial_chapter_elements) do
		local link_element = scraping:select_element(chapter_html, "a") or ""
		local chapter_url = scraping:get_url(link_element) or ""
		local chapter_title = scraping:get_text(scraping:select_element(link_element, ".chapter-title") or "") or ""
		local chapter_date = scraping:get_text(scraping:select_element(link_element, ".chapter-update") or "") or ""
		table.insert(chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
	end

	local book_slug = string.match(html, "var bookSlug = \"([^\"]+)\"")
	if book_slug and scraping:select_element(html, "div#show-more-chapters") then
		local api_url = BASE_URL .. "/api/manga/" .. book_slug .. "/chapters?source=detail"
		local api_request = make_request(api_url)
		local api_html = api_request.text
		local api_chapter_elements = scraping:select_elements(api_html, "#chapter-list > li")

		local api_chapters = {}
		for _, chapter_html in ipairs(api_chapter_elements) do
			local link_element = scraping:select_element(chapter_html, "a") or ""
			local chapter_url = scraping:get_url(link_element) or ""
			local chapter_title = scraping:get_text(scraping:select_element(link_element, ".chapter-title") or "") or ""
			local chapter_date = scraping:get_text(scraping:select_element(link_element, ".chapter-update") or "") or ""
			table.insert(api_chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
		end
		chapters = api_chapters
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
	local request = make_request(url)
	local html = request.text
	local genres = {}
	local genre_elements = scraping:select_elements(html, "ul.genres__wrapper.clearfix a")
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
		version = "0.3.2",
		name = "MangaBuddy",
		img_url = "https://mangabuddy.com/favicon.ico",
		referer_url = "https://mangabuddy.com/",
	}
end

Tests = {
	Test_Scrape_manga = function()
		local manga = Scrape_manga("https://mangabuddy.com/solo-leveling")
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
