- [x] database

  - [ ] ~~think how going away from sea-orm (maybe to something async disel/diesel-async)~~
  - [x] add a way to use different databases (sqlite, postgres, mysql)
  - [x] remove db backups older than 1 week (or some configurable time)

- [ ] refactor api (gql then rest and think about grpc)

  - [ ] ~~websocket for the rest api~~ <- rethink if this is still needed
  - [x] rethink how the whole synchronization works
  - [ ] be able to sync mangas, basicly the user syncs the same manga from different scrapers and then the server kinda merges them into one manga, only for favorite mangas, like add a row to favorite manga, maybe called sources, maybe a array of manga ids, the order of the array is by user choosen priority.

- [x] scheduler

- [ ] add tests for rust code and test in rust for lua code (still not sure how to test scrapers)

- [x] refactor scraper repo/file

  - [x] look at the code and refactor/make it more readable if needed
  - [x] add a way for the server to whitelist/blacklist scrapers from the repository

- [ ] rethink the way of sharing scrapers (currently they are uploaded to catbox)
- [x] overhaul the way we handle the config file

- [ ] scrapers

  - [ ] ~~refactor how magadex scraper works (look at suwayomi impl) or just remove it~~ <- maybe not needed anymore
  - [x] add headsless browser support for lua with fantoccini
  - [x] make all http and headless methods async
  - [ ] add headsless browser support for wasm with fantoccini
  - [ ] ~~add a flags (only boolean) system (for nsfw, etc) where you get te avaliable from the scraper by the Get_info() function then pass that to the client, then when scraping you pass the flags as a parameter of the scraping functions~~ <- maybe just add a nsfw parameter to the scraping functions
  - [ ] maybe make the lua plugins a cargo feature
  - [x] wasm support

- [ ] refactor website

  - [ ] redesign the website
  - [ ] serve it properly (currently using nodejs)

- [ ] new chapter features

  - [ ] download chapters as (cbz, zip, pdf, etc)
  - [ ] add a option to download detected upcoming chapters
  - [ ] add a way to download chapters in bulk

- [x] refactor the scrapers build system

- [ ] refactor the whole binary updating system (maybe just warn that its outdated and not update it automatically)
- [ ] update the readme

- [ ] releases
  - [ ] check on every pr if one of the verisons (check every lib) is outdated (the cargo toml version be heigher than the current released version)
  - [ ] if one of the components updated, automatically update the main binary version and release it too
