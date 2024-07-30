use actix_web::{delete, get, patch, post, web, HttpResponse, Responder};
use sea_orm::{ActiveModelTrait, ColumnTrait, DeleteResult, EntityTrait, ModelTrait, QueryFilter, Set};
use serde::{Deserialize, Serialize};

use crate::entities::categories;
use crate::entities::prelude::Categories;

#[derive(Deserialize)]
struct CreateCategoryRequest {
	name: String,
	user_id: i32,
}

#[derive(Deserialize)]
struct PatchCategoryRequest {
	name: String,
}

#[derive(Serialize)]
struct CreateCategoryResponse {
	id: i32,
	name: String,
	user_id: i32,
	created_at: String,
}

#[post("/categories/create")]
async fn create_category(
	db: web::Data<connection::Connection>,
	category_request: web::Json<CreateCategoryRequest>,
) -> impl Responder {
	let category = categories::ActiveModel {
		name: Set(category_request.name.clone()),
		user_id: Set(category_request.user_id),
		created_at: Set(chrono::Utc::now().naive_utc().to_string()),
		..Default::default()
	};

	let category: categories::Model = category.insert(db.get_ref()).await.unwrap();

	if category.id == 0 {
		return HttpResponse::InternalServerError().body("Failed to create user");
	}

	HttpResponse::Ok().json(CreateCategoryResponse {
		id: category.id,
		name: category.name,
		user_id: category.user_id,
		created_at: category.created_at,
	})
}

#[delete("/categories/delete/{id}")]
async fn delete_category(db: web::Data<connection::Connection>, params: web::Path<i32>) -> impl Responder {
	let category_id = params.into_inner();

	let category: Option<categories::Model> = Categories::find_by_id(category_id).one(db.get_ref()).await.unwrap();

	if category.is_none() {
		return HttpResponse::BadRequest().body("Category not found");
	}

	let res: DeleteResult = category.unwrap().delete(db.get_ref()).await.unwrap();

	if res.rows_affected == 0 {
		return HttpResponse::InternalServerError().body("Failed to delete category");
	}

	HttpResponse::Ok().body("Category deleted")
}

#[patch("/categories/update/{id}")]
async fn update_category(
	db: web::Data<connection::Connection>,
	params: web::Path<i32>,
	category_request: web::Json<PatchCategoryRequest>,
) -> impl Responder {
	let category_id = params.into_inner();

	let category: Option<categories::Model> = Categories::find_by_id(category_id).one(db.get_ref()).await.unwrap();

	if category.is_none() {
		return HttpResponse::BadRequest().body("Category not found");
	}

	let category = category.unwrap();

	let category = categories::ActiveModel {
		name: Set(category_request.name.clone()),
		..category.into()
	};

	let category: categories::Model = category.update(db.get_ref()).await.unwrap();

	HttpResponse::Ok().json(CreateCategoryResponse {
		id: category.id,
		name: category.name,
		user_id: category.user_id,
		created_at: category.created_at,
	})
}

#[get("/categories")]
async fn get_categories(db: web::Data<connection::Connection>) -> impl Responder {
	let categories: Vec<categories::Model> = Categories::find().all(db.get_ref()).await.unwrap();

	let mut response = vec![];

	for category in categories {
		response.push(CreateCategoryResponse {
			id: category.id,
			name: category.name,
			user_id: category.user_id,
			created_at: category.created_at,
		});
	}

	HttpResponse::Ok().json(response)
}

#[get("/categories/{id}")]
async fn get_category(db: web::Data<connection::Connection>, params: web::Path<i32>) -> impl Responder {
	let category_id = params.into_inner();

	let category: Option<categories::Model> = Categories::find_by_id(category_id).one(db.get_ref()).await.unwrap();

	if category.is_none() {
		return HttpResponse::BadRequest().body("Category not found");
	}

	let category = category.unwrap();

	HttpResponse::Ok().json(CreateCategoryResponse {
		id: category.id,
		name: category.name,
		user_id: category.user_id,
		created_at: category.created_at,
	})
}

#[get("users/{id}/categories")]
async fn get_user_categories(db: web::Data<connection::Connection>, params: web::Path<i32>) -> impl Responder {
	let user_id = params.into_inner();

	let categories: Vec<categories::Model> = Categories::find()
		.filter(categories::Column::UserId.eq(user_id))
		.all(db.get_ref())
		.await
		.unwrap();

	let mut response = vec![];

	for category in categories {
		response.push(CreateCategoryResponse {
			id: category.id,
			name: category.name,
			user_id: category.user_id,
			created_at: category.created_at,
		});
	}

	HttpResponse::Ok().json(response)
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
	cfg.service(create_category);
	cfg.service(delete_category);
	cfg.service(update_category);
	cfg.service(get_categories);
	cfg.service(get_category);
	cfg.service(get_user_categories);
}
