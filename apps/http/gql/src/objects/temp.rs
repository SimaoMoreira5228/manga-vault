use async_graphql::SimpleObject;
use chrono::NaiveDateTime;

#[allow(dead_code)]
#[derive(SimpleObject, Clone)]
pub struct Temp {
	pub id: i32,
	pub key: String,
	pub value: String,
	pub expires_at: NaiveDateTime,
}

impl From<database_entities::temp::Model> for Temp {
	fn from(temp: database_entities::temp::Model) -> Self {
		Self {
			id: temp.id,
			key: temp.key,
			value: String::from_utf8(temp.value).unwrap_or_default(),
			expires_at: temp.expires_at,
		}
	}
}
