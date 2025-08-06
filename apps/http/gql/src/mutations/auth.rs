use std::sync::Arc;

use argon2::{
	self, Argon2, PasswordVerifier,
	password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use async_graphql::{
	Context, Error, InputObject, Object, Request, Response, Result, ServerResult, SimpleObject,
	extensions::{Extension, ExtensionContext, ExtensionFactory, NextPrepareRequest, NextRequest},
};
use async_trait::async_trait;
use axum::http::HeaderMap;
use chrono::Utc;
use database_connection::Database;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{Config, objects::users::SanitizedUser};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
	sub: i32,
	exp: usize,
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

#[derive(SimpleObject)]
pub struct AuthPayload {
	token: String,
	user: SanitizedUser,
}

#[derive(Default)]
pub struct AuthMutation;

#[Object]
impl AuthMutation {
	async fn register(&self, ctx: &Context<'_>, input: RegisterInput) -> Result<AuthPayload> {
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
		let argon2 = Argon2::default();
		let password_hash = argon2
			.hash_password(input.password.as_bytes(), &salt)
			.map_err(|e| Error::new(format!("Password hashing failed: {}", e)))?
			.to_string();

		let user = database_entities::users::ActiveModel {
			username: Set(input.username),
			hashed_password: Set(password_hash),
			created_at: Set(Utc::now().naive_utc()),
			..Default::default()
		};

		let user: database_entities::users::Model = user.insert(&db.conn).await?;

		let token = generate_jwt(user.id, &config.secret_jwt, config.jwt_duration_days)?;

		Ok(AuthPayload {
			token,
			user: SanitizedUser::from(user),
		})
	}

	async fn login(&self, ctx: &Context<'_>, input: LoginInput) -> Result<AuthPayload> {
		let db = ctx.data::<Arc<Database>>()?;
		let config = ctx.data::<Arc<Config>>()?;

		let user = database_entities::users::Entity::find()
			.filter(database_entities::users::Column::Username.eq(&input.username))
			.one(&db.conn)
			.await?
			.ok_or_else(|| Error::new("Invalid credentials"))?;

		let argon2 = Argon2::default();
		let parsed_hash = argon2::PasswordHash::new(&user.hashed_password)
			.map_err(|e| Error::new(format!("Invalid password hash: {}", e)))?;

		argon2
			.verify_password(input.password.as_bytes(), &parsed_hash)
			.map_err(|_| Error::new("Invalid credentials"))?;

		let token = generate_jwt(user.id, &config.secret_jwt, config.jwt_duration_days)?;

		Ok(AuthPayload {
			token,
			user: SanitizedUser::from(user),
		})
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

pub struct AuthExtensionFactory;

impl ExtensionFactory for AuthExtensionFactory {
	fn create(&self) -> Arc<dyn Extension> {
		Arc::new(AuthExtension)
	}
}

pub struct AuthExtension;

#[async_trait]
impl Extension for AuthExtension {
	async fn request(&self, ctx: &ExtensionContext<'_>, next: NextRequest<'_>) -> Response {
		next.run(ctx).await
	}

	async fn prepare_request(
		&self,
		ctx: &ExtensionContext<'_>,
		mut request: Request,
		next: NextPrepareRequest<'_>,
	) -> ServerResult<Request> {
		if let Some(headers) = ctx.data_opt::<HeaderMap>() {
			if let Some(auth) = headers
				.get("Authorization")
				.and_then(|h| h.to_str().ok())
				.and_then(|s| s.strip_prefix("Bearer "))
			{
				let config = ctx
					.data::<Arc<Config>>()
					.map_err(|e| async_graphql::ServerError::new(format!("Config error: {:?}", e), None))?;
				if let Ok(token_data) = decode::<Claims>(
					auth,
					&DecodingKey::from_secret(config.secret_jwt.as_bytes()),
					&Validation::default(),
				) {
					let db = ctx
						.data::<Arc<Database>>()
						.map_err(|e| async_graphql::ServerError::new(format!("Database error: {:?}", e), None))?;
					if let Some(user_model) = database_entities::users::Entity::find()
						.filter(database_entities::users::Column::Id.eq(token_data.claims.sub))
						.one(&db.conn)
						.await
						.map_err(|e| async_graphql::ServerError::new(format!("Database query error: {:?}", e), None))?
					{
						let sanitized = SanitizedUser::from(user_model);
						request = request.data(sanitized);
					}
				}
			}
		}

		next.run(ctx, request).await
	}
}
