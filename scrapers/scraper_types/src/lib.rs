use mlua::{FromLua, IntoLua, Lua, Value};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct MangaItem {
    pub title: String,
    pub url: String,
    pub img_url: String,
}

impl IntoLua for MangaItem {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("title", self.title)?;
        table.set("url", self.url)?;
        table.set("img_url", self.img_url)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for MangaItem {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: mlua::Table = FromLua::from_lua(value, lua)?;
        Ok(MangaItem {
            title: table.get("title")?,
            url: table.get("url")?,
            img_url: table.get("img_url")?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct MangaPage {
    pub title: String,
    pub url: String,
    pub img_url: String,
    pub alternative_names: Vec<String>,
    pub authors: Vec<String>,
    pub artists: Option<Vec<String>>,
    pub status: String,
    pub manga_type: Option<String>,
    pub release_date: Option<String>,
    pub description: String,
    pub genres: Vec<String>,
    pub chapters: Vec<Chapter>,
}

impl IntoLua for MangaPage {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("title", self.title)?;
        table.set("url", self.url)?;
        table.set("img_url", self.img_url)?;
        table.set("alternative_names", self.alternative_names)?;
        table.set("authors", self.authors)?;
        table.set("artists", self.artists)?;
        table.set("status", self.status)?;
        table.set("manga_type", self.manga_type)?;
        table.set("release_date", self.release_date)?;
        table.set("description", self.description)?;
        table.set("genres", self.genres)?;
        table.set("chapters", self.chapters)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for MangaPage {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: mlua::Table = FromLua::from_lua(value, lua)?;
        Ok(MangaPage {
            title: table.get("title")?,
            url: table.get("url")?,
            img_url: table.get("img_url")?,
            alternative_names: table.get("alternative_names")?,
            authors: table.get("authors")?,
            artists: table.get("artists")?,
            status: table.get("status")?,
            manga_type: table.get("manga_type")?,
            release_date: table.get("release_date")?,
            description: table.get("description")?,
            genres: table.get("genres")?,
            chapters: table.get("chapters")?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Chapter {
    pub title: String,
    pub url: String,
    pub date: String,
}

impl IntoLua for Chapter {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("title", self.title)?;
        table.set("url", self.url)?;
        table.set("date", self.date)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for Chapter {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: mlua::Table = FromLua::from_lua(value, lua)?;
        Ok(Chapter {
            title: table.get("title")?,
            url: table.get("url")?,
            date: table.get("date")?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Genre {
    pub name: String,
    pub url: String,
}

impl IntoLua for Genre {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("name", self.name)?;
        table.set("url", self.url)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for Genre {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: mlua::Table = FromLua::from_lua(value, lua)?;
        Ok(Genre {
            name: table.get("name")?,
            url: table.get("url")?,
        })
    }
}

#[derive(Debug, Serialize)]
pub struct ScraperInfo {
    pub id: String,
    pub name: String,
    pub img_url: String,
}

impl IntoLua for ScraperInfo {
    fn into_lua(self, lua: &Lua) -> mlua::Result<Value> {
        let table = lua.create_table()?;
        table.set("id", self.id)?;
        table.set("name", self.name)?;
        table.set("img_url", self.img_url)?;
        Ok(Value::Table(table))
    }
}

impl FromLua for ScraperInfo {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let table: mlua::Table = FromLua::from_lua(value, lua)?;
        Ok(ScraperInfo {
            id: table.get("id")?,
            name: table.get("name")?,
            img_url: table.get("img_url")?,
        })
    }
}
