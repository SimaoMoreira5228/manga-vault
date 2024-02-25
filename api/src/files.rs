use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, EntityTrait};
use std::{io::Write, path::Path};

use crate::CONFIG;

#[post("/upload")]
async fn upload_file(db: web::Data<connection::Connection>, mut payload: Multipart) -> impl Responder {
	let path = format!("{}/uploads", CONFIG.directory);

	let path = Path::new(&path);
	if !path.exists() {
		std::fs::create_dir_all(&path).expect("Failed to create uploads directory");
	}

	let mut data = Vec::new();

	while let Some(item) = payload.next().await {
		let mut field = item.unwrap().inspect_err(|e| println!("Error: {}", e)).map_err(|e| {
			println!("Error: {}", e);
			HttpResponse::InternalServerError().body("Error")
		});
		while let Some(chunk) = field.next().await {
			data.extend_from_slice(&chunk.unwrap());
		}
	}

	if !infer::is_image(&data) {
		return HttpResponse::BadRequest().body("Invalid image");
	}

	let unique_signature = uuid::Uuid::new_v4().to_string();
	let kind = infer::get(&data).unwrap();
	let image_url = format!("{}.{}", unique_signature, kind.extension());

	let file_path = format!("{}/{}", path.display(), image_url);
	let mut file = std::fs::File::create(&file_path).expect("Failed to create file");

	file.write_all(&data).expect("Failed to write to file");

	let _ = crate::entities::files::ActiveModel {
		name: Set(image_url.clone()),
		created_at: Set(chrono::Utc::now().naive_utc().to_string()),
		..Default::default()
	}
	.insert(db.get_ref())
	.await;

	HttpResponse::Ok().body(format!("File uploaded: {}", image_url))
}

#[get("/image/{id}")]
async fn get_image(db: web::Data<connection::Connection>, id: web::Path<String>) -> impl Responder {
	let file: crate::entities::files::Model = crate::entities::files::Entity::find_by_id(id.into_inner()).one(db.get_ref()).await.unwrap().expect("File not found");

	let path = format!("{}/uploads", CONFIG.directory);
	let path = Path::new(&path);

	let file = match std::fs::read(format!("{}/{}", path.display(), file.name)) {
		Ok(file) => file,
		Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
	};

	let kind = infer::get(&file).unwrap();
	let mime = kind.mime_type();

	HttpResponse::Ok().content_type(mime).body(file)
}

#[get("download/{id}")]
async fn download_file(db: web::Data<connection::Connection>, id: web::Path<String>) -> impl Responder {
	let db_file: Option<crate::entities::files::Model> = crate::entities::files::Entity::find_by_id(id.into_inner())
		.one(db.get_ref())
		.await
		.unwrap();

	let path = format!("{}/uploads", CONFIG.directory);
	let path = Path::new(&path);

	let file = match std::fs::read(format!("{}/{}", path.display(), db_file.clone().unwrap().name)) {
		Ok(file) => file,
		Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
	};

	let kind = infer::get(&file).unwrap();
	let mime = kind.mime_type();

	HttpResponse::Ok()
		.content_type(mime)
		.insert_header(("Content-Disposition", format!("attachment; filename={}", db_file.clone().unwrap().name)))
		.body(file)
}

pub async fn fetch_external_image(url: &str) -> HttpResponse {
	let res = reqwest::get(url).await;
	if res.is_err() {
		println!("Failed to use url: {}", url);
		HttpResponse::BadRequest().body("The specified website failed to respond.");
	}

	let res = res.unwrap();

	if !res.status().is_success() {
		HttpResponse::BadRequest().body("The specified website failed to respond.");
	}

	let data = res.bytes().await.unwrap().to_vec();

	if !infer::is_image(&data) {
		return HttpResponse::BadRequest().body("Invalid image");
	}

	let kind = infer::get(&data).unwrap();
	let mime = kind.mime_type();

	HttpResponse::Ok().content_type(mime).body(data)
}
