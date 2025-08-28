---@diagnostic disable: undefined-global, undefined-field

PLUGIN_NAME = "natomanga"
PLUGIN_VERSION = "0.2.0"

function Scrape_chapter(url)
    local request = http:get(url)
    local html = request.text
    if http:has_cloudflare_protection(html, request.status, request.headers) then
        html = flaresolverr:get(url).text
    end

    local cdn_json = string.match(html, "var cdns = (%[.-%]);")
    local chapter_images_json = string.match(html, "var chapterImages = (%[.-%]);")

    if not cdn_json or not chapter_images_json then
        return {}
    end

    local cdn = string.match(cdn_json, '"(.-)"')
    if cdn then
        cdn = string.gsub(cdn, "\\", "")
    else
        return {}
    end

    local imgs = {}
    for img_path in string.gmatch(chapter_images_json, '"(.-)"') do
        img_path = string.gsub(img_path, "\\/", "/")

        if not string.match(cdn, "/$") then
            cdn = cdn .. "/"
        end

        table.insert(imgs, cdn .. img_path)
    end

    return imgs
end

function Scrape_latest(page)
    local url = "https://www.natomanga.com/manga-list/latest-manga?page=" .. tostring(page)
    local request = http:get(url)
    local html = request.text
    if http:has_cloudflare_protection(html, request.status, request.headers) then
        html = flaresolverr:get(url).text
    end

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
            url = url
        }
        table.insert(manga_items, manga_item)
    end

    return manga_items
end

function Scrape_trending(page)
    local url = "https://www.natomanga.com/manga-list/hot-manga?page=" .. tostring(page)
    local request = http:get(url)
    local html = request.text
    if http:has_cloudflare_protection(html, request.status, request.headers) then
        html = flaresolverr:get(url).text
    end

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
            url = url
        }
        table.insert(manga_items, manga_item)
    end

    return manga_items
end

function Scrape_search(query, page)
    local url = "https://www.natomanga.com/search/story/" .. query .. "?page=" .. tostring(page)
    local request = http:get(url, { referer = "https://www.natomanga.com/" })
    local html = request.text
    if http:has_cloudflare_protection(html, request.status, request.headers) then
        html = flaresolverr:get(url).text
    end

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
            url = url
        }
        table.insert(manga_items, manga_item)
    end

    return manga_items
end

function Scrape_manga(url)
    local request = http:get(url)
    local html = request.text
    if http:has_cloudflare_protection(html, request.status, request.headers) then
        html = flaresolverr:get(url).text
    end

    local title = scraping:get_text(scraping:select_elements(html, ".manga-info-content h1")[1]) or ""
    local img_url = scraping:get_image_url(scraping:select_elements(html, ".manga-info-pic img")[1]) or ""
    local description = scraping:get_text(scraping:select_elements(html, "#contentBox")[1]) or ""

    local genres = {}
    local genres_div = scraping:select_elements(html, ".genres-wrap .genre-list a")
    for _, genre in ipairs(genres_div) do
        table.insert(genres, scraping:get_text(genre) or "")
    end

    local info_elements = scraping:select_elements(html, ".manga-info-text li")
    local authors = {}
    local status = ""
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
        end
    end


    local chapters = {}
    local chapter_elements = scraping:select_elements(html, ".chapter-list .row")
    for _, chapter in ipairs(chapter_elements) do
        local chapter_title = scraping:get_text(scraping:select_elements(chapter, "span a")[1]) or ""
        local chapter_url = scraping:get_url(scraping:select_elements(chapter, "span a")[1]) or ""
        local chapter_date = scraping:get_text(scraping:select_elements(chapter, "span")[3]) or "New"
        table.insert(chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
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
    local request = http:get(url)
    local html = request.text
    if http:has_cloudflare_protection(html, request.status, request.headers) then
        html = flaresolverr:get(url).text
    end

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
        id = "natomanga",
        name = "NatoManga",
        img_url = "https://www.natomanga.com/images/favicon-manganato.webp",
        referer_url = "https://www.natomanga.com/"
    }
end
