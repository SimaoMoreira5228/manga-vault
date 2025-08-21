---@diagnostic disable: undefined-global, undefined-field

PLUGIN_NAME = "manhuafast"
PLUGIN_VERSION = "0.2.0"

local function scrape_manga_list(url)
    local html = http:get(url).text
    local manga_items = {}

    local manga_divs = scraping:select_elements(html, "div.c-tabs-item")

    for _, manga_div_html in ipairs(manga_divs) do
        local content_divs = scraping:select_elements(manga_div_html, "div.c-tabs-item__content")

        for _, content_html in ipairs(content_divs) do
            local img_element = scraping:select_elements(content_html, "img.img-responsive")[1]
            local img_url = ""
            if img_element then
                img_url = scraping:get_image_url(img_element)
            end

            local title_element = scraping:select_elements(content_html, "div.post-title h3.h4 a")[1]
            local title = ""
            local manga_url = ""
            if title_element then
                title = scraping:get_text(title_element)
                manga_url = scraping:get_url(title_element)
            else
                local heading_element = scraping:select_elements(content_html, "div.post-title h3.h4")[1]
                if heading_element then
                    title = scraping:get_text(heading_element)
                    local url_element = scraping:select_elements(heading_element, "a")[1]
                    if url_element then
                        manga_url = scraping:get_url(url_element)
                    end
                end
            end

            if title ~= "" and manga_url ~= "" then
                table.insert(manga_items, {
                    title = title,
                    img_url = img_url,
                    url = manga_url
                })
            end
        end
    end

    return manga_items
end

function Scrape_chapter(url)
    local html = http:get(url).text
    local img_elements = scraping:select_elements(html, "img.wp-manga-chapter-img")
    local imgs = {}

    for _, img in ipairs(img_elements) do
        local img_url = scraping:get_image_url(img)
        if img_url then
            table.insert(imgs, img_url)
        end
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
    local url = "https://manhuafast.com/page/" ..
        tostring(page) .. "/?s=" .. http:url_encode(query) .. "&post_type=wp-manga&op&author&artist&release&adult"
    return scrape_manga_list(url)
end

local function scrape_manga_details(html)
    local details = {
        genres = {},
        alternative_names = {}
    }

    local summary_content_div = scraping:select_elements(html, "div.summary_content_wrap div.summary_content")[1]
    if not summary_content_div then return details end

    local post_content_items = scraping:select_elements(summary_content_div, "div.post-content div.post-content_item")

    for _, item in ipairs(post_content_items) do
        local heading_element = scraping:select_elements(item, "div.summary-heading h5")[1]
        if heading_element then
            local heading = scraping:get_text(heading_element)
            if string.find(heading, "Gen") then
                local genre_elements = scraping:select_elements(item, "div.summary-content div.genres-content a")
                for _, genre in ipairs(genre_elements) do
                    table.insert(details.genres, scraping:get_text(genre))
                end
            elseif string.find(heading, "Alternative") then
                local alt_names_string = scraping:get_text(scraping:select_elements(item, "div.summary-content")[1])
                details.alternative_names = {}
                for name in string.gmatch(alt_names_string, "([^,]+)") do
                    table.insert(details.alternative_names, string.trim(name))
                end
            end
        end
    end

    return details
end

local function scrape_manga_chapters(url)
    local chapters = {}
    local chapters_url = (string.sub(url, -1) == "/" and url or url .. "/") .. "ajax/chapters/"

    local chapters_html = http:post(
        chapters_url,
        "",
        {
            ["Referer"] = "https://manhuafast.com/",
            ["X-Requested-With"] = "XMLHttpRequest"
        }
    ).text

    local chapter_elements = scraping:select_elements(chapters_html, "div.listing-chapters_wrap ul li")
    for _, chapter in ipairs(chapter_elements) do
        local link_element = scraping:select_elements(chapter, "a")[1]
        local date_element = scraping:select_elements(chapter, "span i")[1]
        if link_element then
            table.insert(chapters, {
                title = scraping:get_text(link_element) or "",
                url = scraping:get_url(link_element) or "",
                date = date_element and scraping:get_text(date_element) or "New"
            })
        end
    end

    return table.reverse(chapters)
end

function Scrape_manga(url)
    local html = http:get(url).text

    local title_element = scraping:select_elements(html, "div.post-title h1")[1]
    local title = title_element and scraping:get_text(title_element) or ""

    local img_element = scraping:select_elements(html, "div.summary_image img")[1]
    local img_url = img_element and scraping:get_image_url(img_element) or ""

    local description_parts = {}
    local description_elements = scraping:select_elements(html, "div.description-summary div.summary__content p")
    for _, element in ipairs(description_elements) do
        table.insert(description_parts, scraping:get_text(element))
    end
    local description = table.concat(description_parts, "\n")

    local details = scrape_manga_details(html)
    local chapters = scrape_manga_chapters(url)

    return {
        title = title,
        url = url,
        img_url = img_url,
        genres = details.genres,
        alternative_names = details.alternative_names,
        authors = {},
        artists = {},
        status = "",
        manga_type = "",
        release_date = "",
        description = description,
        chapters = chapters,
    }
end

function Scrape_genres_list()
    local url = "https://manhuafast.com/?s=&post_type=wp-manga"
    local html = http:get(url).text
    local genres = {}

    local genre_elements = scraping:select_elements(html, "div.checkbox-group div.checkbox label")
    for _, genre_element in ipairs(genre_elements) do
        local name = scraping:get_text(genre_element) or ""
        if name ~= "" then
            local genre_url = "https://manhuafast.com/?s=&post_type=wp-manga&genre%5B%5D=" ..
                http:url_encode(string.gsub(name, " ", "-")) .. "&op=&author&artist&release&adult="
            table.insert(genres, { name = name, url = genre_url })
        end
    end

    return genres
end

function Get_info()
    return {
        id = "manhuafast",
        name = "Manhuafast",
        img_url = "https://manhuafast.com/wp-content/uploads/2021/01/cropped-Dark-Star-Emperor-Manga-193x278-1-32x32.jpg",
        referer_url = "https://manhuafast.com/"
    }
end
