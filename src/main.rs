use scrappers::Scrapper;

#[tokio::main]
async fn main() {
    let mangaread_org = Scrapper::new(scrappers::ScrapperType::MangareadOrg);
    let mangas = mangaread_org.scrape_search("love", 1).await.unwrap();

    for manga in mangas {
        println!("{:?}", manga);
    }
}
