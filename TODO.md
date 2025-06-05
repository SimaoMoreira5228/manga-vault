- [ ] database

  - [ ] ~~think how going away from sea-orm (maybe to something async disel/diesel-async)~~
  - [x] add a way to use different databases (sqlite, postgres, mysql)
  - [x] remove db backups older than 1 week (or some configurable time)

- [ ] refactor api (gql then rest and think about grpc)

  - [ ] websocket for the rest api
  - [ ] rethink how the whole synchronization works

- [ ] add tests for rust code and test in rust for lua code (still not sure how to test scrapers)

- [x] refactor scraper repo/file

  - [x] look at the code and refactor/make it more readable if needed
  - [x] add a way for the server to whitelist/blacklist scrapers from the repository

- [ ] rethink the way of sharing scrapers (currently they are uploaded to catbox)
- [x] overhaul the way we handle the config file

- [ ] scrapers

  - [ ] refactor how magadex scraper works (look at suwayomi impl) or just remove it
  - [x] add headsless browser support for lua with fantoccini
  - [x] make all http and headless methods async
  - [ ] add a flags (only boolean) system (for nsfw, etc) where you get te avaliable from the scraper by the Get_info() function then pass that to the client, then when scraping you pass the flags as a parameter of the scraping functions
  - [ ] maybe make the lua plugins a cargo feature
  - [ ] maybe add support for a lua like static language

- [ ] refactor website

  - [ ] redesign the website
  - [ ] serve it properly (currently using nodejs)

- [ ] new chapter features

  - [ ] download chapters as (cbz, zip, pdf, etc)
  - [ ] add a option to download detected upcoming chapters
  - [ ] add a way to download chapters in bulk

- [x] refactor the scrapers build system

- [ ] refactor the whole updating system (maybe just warn that its outdated and not update it automatically, but what about the scrapers? just stop updating them?)
- [ ] update the readme
