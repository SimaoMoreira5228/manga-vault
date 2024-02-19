use std::collections::HashMap;

use scraper::ElementRef;

mod scrape;

pub struct MangaReadOrgScrapper;

impl MangaReadOrgScrapper {
    pub fn new() -> Self {
        MangaReadOrgScrapper
    }

    pub fn get_image_url(&element: &ElementRef) -> String {
        let attrs = element.value().attrs().collect::<HashMap<&str, &str>>();
        let img_url: &str;
        if attrs.get("src").is_some() {
            img_url = attrs.get("src").unwrap();
        } else if attrs.get("data-src").is_some() {
            img_url = attrs.get("data-src").unwrap();
        } else if attrs.get("data-cfsrc").is_some() {
            img_url = attrs.get("data-cfsrc").unwrap();
        } else if attrs.get("data-lazy-src").is_some() {
            img_url = attrs.get("data-lazy-src").unwrap();
        } else {
            img_url = "";
        }

        img_url.to_string()
    }
}