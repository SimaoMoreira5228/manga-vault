local function http_get(url, headers)
	if not url or type(url) ~= "string" or url:match("^%s*$") then
		log.warn("[manhuafast] Empty or invalid URL provided to http_get: " .. tostring(url))
		return { text = "", status = 0, headers = {}, ok = false }
	end

	if url:match("^https?://[^/]+//") then
		log.warn("[manhuafast] Malformed URL provided to http_get: " .. url)
		return { text = "", status = 0, headers = {}, ok = false }
	end

	headers = headers or {}
	local response = http:get(url, headers)

	if not response.ok then
		if response.status == 404 then
			log.warn("[manhuafast] URL not found: " .. url)
			utils.raise_error("not_found", "Page not found: " .. url, false)
		else
			log.error("[manhuafast] HTTP request failed: " .. (response.error and response.error.message or "Unknown error"))
			utils.raise_error("network", "HTTP request failed", true)
		end
	end

	if
		flaresolverr:using_flaresolverr() and http:has_cloudflare_protection(response.text, response.status, response.headers)
	then
		log.info("[manhuafast] Cloudflare protection detected, using Flaresolverr")
		response = flaresolverr:get(url)
		if not response.ok then
			log.error(
				"[manhuafast] Flaresolverr request failed: " .. (response.error and response.error.message or "Unknown error")
			)
			utils.raise_error("cloudflare", "Flaresolverr failed", true)
		end
	end

	return response
end

local function nodes(html, selector)
	if scraping.try_select_elements then
		local ok, res = scraping:try_select_elements(html, selector)
		if ok and res and res.ok then
			return res.value
		end
	end

	if scraping.select_elements then
		local ok_res = scraping:select_elements(html, selector)
		return ok_res or {}
	end

	return {}
end

local function first(html, selector)
	local n = nodes(html, selector)
	return n[1]
end

local function hex_to_bytes(hex)
	if not hex or hex == "" then
		return ""
	end

	local out = {}
	for i = 1, #hex, 2 do
		local byte = tonumber(hex:sub(i, i + 1), 16)
		if byte then
			table.insert(out, string.char(byte))
		end
	end

	return table.concat(out)
end

local function get_srcset_image(el)
	if not el then
		return nil
	end
	local s = tostring(el)
	local srcset = s:match("srcset%s*=%s*\"([^\"]+)\"") or s:match("srcset%s*=%s*'([^']+)'")
	if not srcset then
		srcset = s:match("data%-srcset%s*=%s*\"([^\"]+)\"") or s:match("data%-srcset%s*=%s*'([^']+)'")
	end
	if srcset then
		for item in srcset:gmatch("([^,]+)") do
			local url = item:match("(%S+)")
			if url and url ~= "" then
				return url
			end
		end
	end
	return nil
end

local function image_from_element(el)
	if not el then
		return nil
	end
	if scraping.get_image_url then
		local u = scraping:get_image_url(el)
		if u and u ~= "" then
			return u
		end
	end

	local s = tostring(el)
	local maybe = get_srcset_image(el)
	if maybe and maybe ~= "" then
		return maybe
	end

	local data_src = s:match("data%-src%s*=%s*\"([^\"]+)\"") or s:match("data%-src%s*=%s*'([^']+)'")
	if data_src and data_src ~= "" then
		return data_src
	end

	local src = s:match("src%s*=%s*\"([^\"]+)\"") or s:match("src%s*=%s*'([^']+)'")
	if src and src ~= "" then
		return src
	end

	return nil
end

local function scrape_manga_list(url)
	local req = http_get(url)
	local html = req.text or ""
	local out = {}

	local manga_divs = nodes(html, "div.c-tabs-item")
	for _, div in ipairs(manga_divs) do
		local contents = nodes(div, "div.c-tabs-item__content")
		for _, content in ipairs(contents) do
			local img_el = first(content, "img.img-responsive") or first(content, "img")
			local img_url = img_el and scraping:get_image_url(img_el) or ""

			local title_el = first(content, "div.post-title h3.h4 a")
				or first(content, "div.post-title h3.h4")
				or first(content, "a")
			local title = title_el and scraping:get_text(title_el) or ""
			local link = title_el and (scraping.get_url and scraping:get_url(title_el) or scraping:get_url(title_el)) or ""

			if title ~= "" and link ~= "" then
				table.insert(out, { title = title, img_url = img_url, url = link })
			end
		end
	end

	return out
end

function Scrape_chapter(url)
	local req = http_get(url)
	local html = req.text or ""

	local protector = first(html, "#chapter-protector-data") or first(html, "script#chapter-protector-data")
	if protector then
		local src = protector:match("src%s*=%s*\"([^\"]+)\"") or protector:match("src%s*=%s*'([^']+)'")
		local protector_html = src or protector
		if src and src:match("^data:text/javascript;base64,") then
			local b64 = src:match("^data:text/javascript;base64,(.+)")
			if b64 and b64 ~= "" then
				protector_html = utils.base64_decode(b64)
			end
		end

		local password = protector_html:match("wpmangaprotectornonce='([^']+)'")
			or protector_html:match("wpmangaprotectornonce=\"([^\"]+)\"")
		local chapter_data = protector_html:match("chapter_data='([^']+)'") or protector_html:match("chapter_data=\"([^\"]+)\"")
		if chapter_data and password then
			chapter_data = chapter_data:gsub("\\/", "/")
			local ct = chapter_data:match("\"ct\"%s*:%s*\"([^\"]+)\"")
			local s_hex = chapter_data:match("\"s\"%s*:%s*\"([^\"]+)\"")
			if ct and s_hex then
				local unsalted = utils.base64_decode(ct)
				local salt = hex_to_bytes(s_hex)
				local combined = "Salted__" .. salt .. unsalted
				local combined_b64 = utils.base64_encode(combined)
				local decrypted = utils.aes_decrypt(combined_b64, password)
				if decrypted and decrypted ~= "" then
					local imgs = {}
					for u in decrypted:gmatch("\"([^\"]+)\"") do
						if u and u ~= "" then
							table.insert(imgs, u)
						end
					end
					if #imgs > 0 then
						return imgs
					end
				end
			end
		end
	end

	local img_selectors = { "img.wp-manga-chapter-img", "div.page-break img", "figure img", "img" }
	local imgs = {}
	for _, sel in ipairs(img_selectors) do
		local els = nodes(html, sel)
		for _, el in ipairs(els) do
			local u = image_from_element(el) or scraping:get_image_url(el)
			if u and u ~= "" then
				table.insert(imgs, u)
			end
		end
		if #imgs > 0 then
			break
		end
	end

	if #imgs == 0 then
		log.warn("[manhuafast] No images found for chapter: " .. tostring(url))
		utils.raise_error("parse", "No images found in chapter", false)
	end

	return imgs
end

function Scrape_latest(page)
	local url = "https://manhuafast.com/page/" .. tostring(page) .. "/?s&post_type=wp-manga&m_orderby=latest"
	return scrape_manga_list(url)
end

function Scrape_trending(page)
	local url = "https://manhuafast.com/page/" .. tostring(page) .. "/?s&post_type=wp-manga&m_orderby=trending"
	return scrape_manga_list(url)
end

function Scrape_search(query, page)
	local url = "https://manhuafast.com/page/"
		.. tostring(page)
		.. "/?s="
		.. http:url_encode(query)
		.. "&post_type=wp-manga&op&author&artist&release&adult"
	return scrape_manga_list(url)
end

local function scrape_manga_details(html)
	local details = {
		genres = {},
		alternative_names = {},
		authors = {},
		artists = {},
		status = "",
		manga_type = "",
		release_date = "",
	}

	local summary = first(html, "div.summary_content_wrap div.summary_content") or first(html, "div.summary_content")
	if summary then
		local items = nodes(summary, "div.post-content div.post-content_item")
		for _, item in ipairs(items) do
			local heading = first(item, "div.summary-heading h5")
			local heading_text = heading and scraping:get_text(heading) or ""
			if heading_text:find("Gen") or heading_text:find("Genre") then
				local genre_els = nodes(item, "div.summary-content div.genres-content a")
				for _, g in ipairs(genre_els) do
					table.insert(details.genres, scraping:get_text(g))
				end
			elseif heading_text:find("Alternative") or heading_text:find("Alt") then
				local alt_el = first(item, "div.summary-content")
				local alt_txt = alt_el and scraping:get_text(alt_el) or ""
				for name in string.gmatch(alt_txt, "([^,]+)") do
					local n = (string.gsub(name, "^%s*(.-)%s*$", "%1"))
					if n ~= "" then
						table.insert(details.alternative_names, n)
					end
				end
			elseif heading_text:find("Author") then
				local author_els = nodes(item, "div.summary-content a")
				for _, a in ipairs(author_els) do
					table.insert(details.authors, scraping:get_text(a))
				end
			elseif heading_text:find("Artist") then
				local artist_els = nodes(item, "div.summary-content a")
				for _, a in ipairs(artist_els) do
					table.insert(details.artists, scraping:get_text(a))
				end
			elseif heading_text:find("Status") then
				local status_el = first(item, "div.summary-content") or first(item, "div.post-content")
				if status_el then
					details.status = scraping:get_text(status_el) or ""
				end
			end
		end
	end

	if #details.authors == 0 then
		for _, a in ipairs(nodes(html, "div.author-content > a")) do
			table.insert(details.authors, scraping:get_text(a))
		end
	end
	if #details.artists == 0 then
		for _, a in ipairs(nodes(html, "div.artist-content > a")) do
			table.insert(details.artists, scraping:get_text(a))
		end
	end

	return details
end

local function scrape_manga_chapters(url)
	local chapters = {}
	local base = (string.sub(url, -1) == "/" and url or url .. "/")
	local chapters_url = base .. "ajax/chapters/"

	local response =
		http:post(chapters_url, "", { ["Referer"] = "https://manhuafast.com/", ["X-Requested-With"] = "XMLHttpRequest" })
	if not response.ok then
		log.info(
			"[manhuafast] AJAX chapters failed, falling back to page HTML: " .. tostring(response.error and response.error.message)
		)
		local page_req = http_get(url)
		local page_html = page_req.text or ""
		local chapter_nodes = nodes(page_html, "li.wp-manga-chapter")
		for _, ch in ipairs(chapter_nodes) do
			local link = first(ch, "a")
			local date_el = first(ch, "span.chapter-release-date") or first(ch, "span i")
			if link then
				table.insert(chapters, {
					title = scraping:get_text(link) or "",
					url = scraping:get_url(link) or "",
					date = date_el and scraping:get_text(date_el) or "New",
				})
			end
		end
	else
		local chapters_html = response.text or ""
		local chapter_elements = nodes(chapters_html, "div.listing-chapters_wrap ul li")
		if #chapter_elements == 0 then
			chapter_elements = nodes(chapters_html, "ul li")
		end
		for _, chapter in ipairs(chapter_elements) do
			local link_element = first(chapter, "a")
			local date_element = first(chapter, "span i") or first(chapter, "span")
			if link_element then
				table.insert(chapters, {
					title = scraping:get_text(link_element) or "",
					url = scraping:get_url(link_element) or "",
					date = date_element and scraping:get_text(date_element) or "New",
				})
			end
		end
	end

	if #chapters == 0 then
		log.warn("[manhuafast] No chapters found for manga: " .. tostring(url))
	end

	if table.reverse then
		return table.reverse(chapters)
	end

	return chapters
end

function Scrape(url)
	local req = http_get(url)
	local html = req.text or ""

	local title_el = first(html, "div.post-title h1") or first(html, "div.post-title h3") or first(html, "#manga-title > h1")
	local title = title_el and scraping:get_text(title_el) or ""
	if title == "" then
		log.error("[manhuafast] Failed to parse title for: " .. tostring(url))
		utils.raise_error("parse", "Failed to parse title", false)
	end

	local img_el = first(html, "div.summary_image img") or first(html, "div.summary_image img")
	local img_url = img_el and scraping:get_image_url(img_el) or ""

	local desc_parts = {}
	for _, p in ipairs(nodes(html, "div.description-summary div.summary__content p")) do
		table.insert(desc_parts, scraping:get_text(p))
	end
	local description = table.concat(desc_parts, "\n")

	local details = scrape_manga_details(html)
	local chapters = scrape_manga_chapters(url)

	return {
		title = title,
		url = url,
		img_url = img_url,
		genres = details.genres,
		alternative_names = details.alternative_names,
		authors = details.authors,
		artists = details.artists,
		status = details.status,
		manga_type = details.manga_type,
		release_date = details.release_date,
		description = description,
		chapters = chapters,
	}
end

function Scrape_genres_list()
	local url = "https://manhuafast.com/?s=&post_type=wp-manga"
	local req = http_get(url)
	local html = req.text or ""
	local out = {}
	for _, g in ipairs(nodes(html, "div.checkbox-group div.checkbox label")) do
		local name = scraping:get_text(g) or ""
		if name ~= "" then
			local genre_url = "https://manhuafast.com/?s=&post_type=wp-manga&genre%5B%5D="
				.. http:url_encode(string.gsub(name, " ", "-"))
				.. "&op=&author&artist&release&adult="
			table.insert(out, { name = name, url = genre_url })
		end
	end
	return out
end

function Get_info()
	return {
		id = "manhuafast",
		version = "0.5.1",
		name = "Manhuafast",
		img_url = "https://manhuafast.com/wp-content/uploads/2021/01/cropped-Dark-Star-Emperor-Manga-193x278-1-32x32.jpg",
		referer_url = "https://manhuafast.com/",
	}
end

Tests = {
	Test_Scrape_manga = function()
		local manga = Scrape("https://manhuafast.com/manga/nano-machine-all-chapters/")
		assert(manga.title and manga.title ~= "", "Title missing")
		assert(type(manga.chapters) == "table" and #manga.chapters > 0, "No chapters found")
		assert(manga.img_url and manga.img_url ~= "", "Image URL is empty")
		assert(manga.description and manga.description ~= "", "Description is empty")
		assert(type(manga.genres) == "table", "Genres missing")
	end,

	Test_Scrape_chapter = function()
		local images = Scrape_chapter("https://manhuafast.com/manga/nano-machine-all-chapters/chapter-256/")
		assert(type(images) == "table" and #images > 0, "No images found")
	end,

	Test_Scrape_latest = function()
		local mangas = Scrape_latest(1)
		assert(type(mangas) == "table" and #mangas > 0, "No mangas found in latest")
	end,

	Test_Scrape_trending = function()
		local mangas = Scrape_trending(1)
		assert(type(mangas) == "table" and #mangas > 0, "No mangas found in trending")
	end,

	Test_Scrape_search = function()
		local mangas = Scrape_search("nano", 1)
		assert(type(mangas) == "table" and #mangas > 0, "No mangas found in search")
	end,

	Test_Scrape_genres_list = function()
		local genres = Scrape_genres_list()
		assert(type(genres) == "table" and #genres > 0, "No genres found")
	end,
}
