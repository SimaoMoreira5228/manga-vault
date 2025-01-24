---@diagnostic disable: undefined-global

PLUGIN_NAME = "manhuafast"
PLUGIN_VERSION = "0.1.1"

local function reverseTable(t)
  local reversed = {}
  local n = #t -- Length of the table
  for i = n, 1, -1 do
    table.insert(reversed, t[i])
  end
  return reversed
end

function Scrape_chapter(url)
  local html = http.get(url).text

  local img_selector = "img.wp-manga-chapter-img"
  local img_elements = scraping.select_elements(html, img_selector)

  local imgs = {}

  for _, img in ipairs(img_elements) do
    local img_url = scraping.get_image_url(img)
    table.insert(imgs, img_url)
  end

  return imgs
end

function Scrape_latest(page)
  local url = "https://manhuafast.com/page/" .. tostring(page) .. "/?s&post_type=wp-manga&m_orderby=latest"
  local html = http.get(url).text

  local manga_divs = scraping.select_elements(html, "div.c-tabs-item")
  local manga_items = {}

  for _, manga_div_html in ipairs(manga_divs) do
    local content_divs = scraping.select_elements(manga_div_html, "div.c-tabs-item__content")

    for _, content_html in ipairs(content_divs) do
      local img_elements = scraping.select_elements(content_html, "img.img-responsive")
      local img_url = scraping.get_image_url(img_elements[1]) or ""

      local title_elements = scraping.select_elements(content_html, "div.post-title h3.h4 a")
      local title = scraping.get_text(title_elements[1]) or ""

      local url_elements = scraping.select_elements(content_html, "div.post-title h3.h4 a")
      local url = scraping.get_url(url_elements[1]) or ""

      local manga_item = {
        title = title,
        img_url = img_url,
        url = url
      }
      table.insert(manga_items, manga_item)
    end
  end

  return manga_items
end

function Scrape_trending(page)
  local url = "https://manhuafast.com/page/" .. tostring(page) .. "/?s&post_type=wp-manga&m_orderby=trending"
  local html = http.get(url).text

  local manga_divs = scraping.select_elements(html, "div.c-tabs-item")
  local manga_items = {}

  for _, manga_div_html in ipairs(manga_divs) do
    local content_divs = scraping.select_elements(manga_div_html, "div.c-tabs-item__content")

    for _, content_html in ipairs(content_divs) do
      local img_elements = scraping.select_elements(content_html, "img.img-responsive")
      local img_url = scraping.get_image_url(img_elements[1]) or ""

      local title_elements = scraping.select_elements(content_html, "div.post-title h3.h4")
      local title = scraping.get_text(title_elements[1]) or ""

      local url_elements = scraping.select_elements(content_html, "div.post-title h3.h4 a")
      local url = scraping.get_url(url_elements[1]) or ""

      local manga_item = {
        title = title,
        img_url = img_url,
        url = url
      }
      table.insert(manga_items, manga_item)
    end
  end

  return manga_items
end

function Scrape_search(query, page)
  local url = "https://manhuafast.com/page/" ..
      tostring(page) .. "/?s=" .. query .. "&post_type=wp-manga&op&author&artist&release&adult"
  local html = http.get(url).text

  local manga_divs = scraping.select_elements(html, "div.c-tabs-item")
  local manga_items = {}

  for _, manga_div_html in ipairs(manga_divs) do
    local content_divs = scraping.select_elements(manga_div_html, "div.c-tabs-item__content")

    for _, content_html in ipairs(content_divs) do
      local img_elements = scraping.select_elements(content_html, "img.img-responsive")
      local img_url = scraping.get_image_url(img_elements[1]) or ""

      local title_elements = scraping.select_elements(content_html, "div.post-title h3.h4")
      local title = scraping.get_text(title_elements[1]) or ""

      local url_elements = scraping.select_elements(content_html, "div.post-title h3.h4 a")
      local url = scraping.get_url(url_elements[1]) or ""

      local manga_item = {
        title = title,
        img_url = img_url,
        url = url
      }
      table.insert(manga_items, manga_item)
    end
  end

  return manga_items
end

function Scrape_manga(url)
  local html = http.get(url).text

  local title = scraping.get_text(scraping.select_elements(html, "div.post-title h1")[1]) or ""
  local img_url = scraping.get_image_url(scraping.select_elements(html, "div.summary_image img")[1]) or ""

  local summary_content_div = scraping.select_elements(html, "div.summary_content_wrap div.summary_content")
  local post_content_item = scraping.select_elements(summary_content_div[1], "div.post-content div.post-content_item")

  local genres = {}
  local alternative_names = {}

  for _, item in ipairs(post_content_item) do
    if string.find(scraping.get_text(scraping.select_elements(item, "div.summary-heading h5")[1]), "Gen") then
      local genres_div = scraping.select_elements(item, "div.summary-content div.genres-content a")
      for _, genre in ipairs(genres_div) do
        table.insert(genres, scraping.get_text(genre) or "")
      end
    elseif string.find(scraping.get_text(scraping.select_elements(item, "div.summary-heading h5")[1]), "Alternative") then
      local alternative_names_string = scraping.get_text(scraping.select_elements(item, "div.summary-content")[1])
      ---@diagnostic disable-next-line: undefined-field
      alternative_names = string.split(alternative_names_string, ",")
    end
  end

  local description = scraping.get_text(scraping.select_elements(html, "div.description-summary div.summary__content p")
    [1]) or ""

  local chapters = {}
  local chapters_url = ""
  if string.sub(url, -1) == "/" then
    chapters_url = url .. "ajax/chapters/"
  else
    chapters_url = url .. "/ajax/chapters/"
  end

  local chapters_html = http.post(
    chapters_url,
    "",
    {
      ["Referer"] = "https://manhuafast.com/",
      ["X-Requested-With"] = "XMLHttpRequest"
    }
  ).text

  for _, chapter in ipairs(scraping.select_elements(chapters_html, "div.listing-chapters_wrap ul li")) do
    local chapter_title = scraping.get_text(scraping.select_elements(chapter, "a")[1]) or ""
    local chapter_url = scraping.get_url(scraping.select_elements(chapter, "a")[1]) or ""
    local chapter_date = scraping.get_text(scraping.select_elements(chapter, "span i")[1]) or "New"
    table.insert(chapters, { title = chapter_title, url = chapter_url, date = chapter_date })
  end

  local page = {
    title = title,
    url = url,
    img_url = img_url,
    genres = genres,
    alternative_names = alternative_names,
    authors = {},
    artists = {},
    status = "",
    manga_type = "",
    release_date = "",
    description = description,
    chapters = reverseTable(chapters)
  }

  return page
end

function Scrape_genres_list()
  local url = "https://manhuafast.com/?s=&post_type=wp-manga"
  local html = http.get(url).text

  local genres = {}
  for _, genre_element in ipairs(scraping.select_elements(html, "div.checkbox-group div.checkbox label")) do
    local name = scraping.get_text(genre_element) or ""
    local url = "https://manhuafast.com/?s=&post_type=wp-manga&genre%5B%5D=" ..
        string.gsub(name, " ", "-") .. "&op=&author=&artist=&release=&adult="

    table.insert(genres, { name = name, url = url })
  end

  return genres
end

function Get_info()
  return {
    id = "manhuafast",
    name = "Manhuafast",
    img_url = "https://manhuafast.com/wp-content/uploads/2017/10/Untitled-1-e1703870904418.png"
  }
end
