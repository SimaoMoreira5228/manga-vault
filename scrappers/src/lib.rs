use async_trait::async_trait;
use tokio::io::AsyncWriteExt;

mod mangaread_org;

#[async_trait]
pub trait ScrapperTraits {
    async fn scrape_chapter(&self, url: &str) -> Result<Vec<String>, reqwest::Error>;
    async fn scrape_latest(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
    async fn scrape_trending(&self, page: u16) -> Result<Vec<MangaItem>, reqwest::Error>;
    async fn scrape_search(
        &self,
        query: &str,
        page: u16,
    ) -> Result<Vec<MangaItem>, reqwest::Error>;
    async fn scrape_manga(&self, url: &str) -> Result<Vec<Manga>, reqwest::Error>;
    fn get_scrapper_type(&self) -> ScrapperType;
}

pub struct Scrapper;

impl Scrapper {
    pub fn new(type_: ScrapperType) -> Box<dyn ScrapperTraits> {
        match type_ {
            ScrapperType::MangareadOrg => Box::new(mangaread_org::MangaReadOrgScrapper::new()),
        }
    }
}

impl Scrapper {
    pub async fn download_img(url: &str) -> Result<(), reqwest::Error> {
        let res = reqwest::get(url).await;
        if res.is_err() {
            println!("Error: {:?}", res.err());
            return Ok(());
        }
        let bytes = res.unwrap().bytes().await.unwrap();

        let file_name = url.split("/").last().unwrap();
        let mut file = tokio::fs::File::create(format!("./imgs/{}", file_name))
            .await
            .unwrap();
        let result = file.write_all(&bytes).await;
        if result.is_err() {
            println!("Error: {:?}", result.err());
        }
        Ok(())
    }
}

pub enum ScrapperType {
    MangareadOrg,
}

#[derive(Debug)]
pub struct MangaItem {
    title: String,
    url: String,
    img_url: String,
}

#[derive(Debug)]
pub struct Manga {
    title: String,
    url: String,
    img_url: String,
    description: String,
    chapters: Vec<Chapter>,
}

#[derive(Debug)]
pub struct Chapter {
    title: String,
    url: String,
}
