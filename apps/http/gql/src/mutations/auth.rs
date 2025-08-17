use std::sync::Arc;

use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{PasswordHasher, PasswordVerifier};
use async_graphql::{Context, Error, InputObject, Object, Result};
use chrono::Utc;
use database_connection::Database;
use jsonwebtoken::{EncodingKey, Header, encode};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::Config;
use crate::objects::users::SanitizedUser;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
	pub sub: i32,
	pub exp: usize,
}

#[derive(InputObject)]
struct LoginInput {
	username: String,
	password: String,
}

#[derive(InputObject)]
struct RegisterInput {
	username: String,
	password: String,
}

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
	async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<SanitizedUser> {
		let db = ctx.data::<Arc<Database>>()?;
		let config = ctx.data::<Arc<Config>>()?;

		let exists = database_entities::users::Entity::find()
			.filter(database_entities::users::Column::Username.eq(&input.username))
			.one(&db.conn)
			.await?;

		if exists.is_some() {
			return Err(Error::new("Username already exists"));
		}

		let salt = SaltString::generate(&mut OsRng);
		let password_hash = argon2::Argon2::default()
			.hash_password(input.password.as_bytes(), &salt)?
			.to_string();

		let user = database_entities::users::ActiveModel {
			username: Set(input.username),
			hashed_password: Set(password_hash),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let user: database_entities::users::Model = user.insert(&db.conn).await?;

		let token = generate_jwt(user.id, &config.secret_jwt, config.jwt_duration_days)?;
		ctx.append_http_header(
			"Set-Cookie",
			format!(
				"token={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
				token,
				config.jwt_duration_days * 24 * 60 * 60
			),
		);

		Ok(SanitizedUser::from(user))
	}

	async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<SanitizedUser> {
		let db = ctx.data::<Arc<Database>>()?;
		let config = ctx.data::<Arc<Config>>()?;

		let user = database_entities::users::Entity::find()
			.filter(database_entities::users::Column::Username.eq(&input.username))
			.one(&db.conn)
			.await?
			.ok_or_else(|| Error::new("Invalid credentials"))?;

		if !verify_password(&input.password, user.id, &db).await? {
			return Err(Error::new("Invalid credentials"));
		}

		let token = generate_jwt(user.id, &config.secret_jwt, config.jwt_duration_days)?;
		ctx.append_http_header(
			"Set-Cookie",
			format!(
				"token={}; HttpOnly; Secure; SameSite=Lax; Path=/; Max-Age={}",
				token,
				config.jwt_duration_days as u64 * 24 * 60 * 60
			),
		);

		Ok(SanitizedUser::from(user))
	}

	async fn logout(&self, ctx: &Context<'_>) -> Result<bool> {
		ctx.append_http_header("Set-Cookie", "token=; Secure; HttpOnly; SameSite=Lax; Path=/; Max-Age=0");
		Ok(true)
	}
}

fn generate_jwt(user_id: i32, secret: &str, duration: u16) -> Result<String> {
	let expiration = Utc::now()
		.checked_add_signed(chrono::Duration::days(duration.into()))
		.expect("Invalid timestamp")
		.timestamp() as usize;

	let claims = Claims {
		sub: user_id,
		exp: expiration,
	};

	encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
		.map_err(|e| Error::new(format!("JWT creation failed: {}", e)))
}

async fn verify_password(password: &str, user_id: i32, db: &Arc<Database>) -> Result<bool> {
	let user = database_entities::users::Entity::find_by_id(user_id)
		.one(&db.conn)
		.await?
		.ok_or_else(|| Error::new("User not found"))?;

	match argon2::PasswordHash::new(&user.hashed_password) {
		Ok(parsed_phc) => {
			let ok = argon2::Argon2::default()
				.verify_password(password.as_bytes(), &parsed_phc)
				.is_ok();

			if ok {
				return Ok(true);
			} else {
				return Ok(false);
			}
		}

		Err(_) => {
			let bcrypt_ok =
				bcrypt::verify(password, &user.hashed_password).map_err(|_| Error::new("Password verification failed"))?;

			if !bcrypt_ok {
				return Ok(false);
			}

			let salt = SaltString::generate(&mut OsRng);
			let new_hash = argon2::Argon2::default()
				.hash_password(password.as_bytes(), &salt)?
				.to_string();

			let user_update = database_entities::users::ActiveModel {
				id: Set(user_id),
				hashed_password: Set(new_hash),
				..Default::default()
			};
			user_update.update(&db.conn).await?;

			return Ok(true);
		}
	}
}
