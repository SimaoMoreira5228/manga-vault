use actix_multipart::Multipart;
use actix_web::{get, post, web, HttpResponse, Responder};
use database::Database;
use futures_util::StreamExt;
use std::{
	ffi::OsStr,
	io::Write,
	path::Path,
	sync::{Arc, Mutex},
};

use crate::CONFIG;

#[post("/upload")]
async fn upload_file(db: web::Data<Arc<Mutex<Database>>>, mut payload: Multipart) -> impl Responder {
	let path = format!("{}/uploads", CONFIG.directory);

	let path = Path::new(&path);
	if !path.exists() {
		std::fs::create_dir_all(&path).expect("Failed to create uploads directory");
	}

	while let Some(Ok(mut field)) = payload.next().await {
		let content_disposition = field.content_disposition().clone();

		let old_name = content_disposition.get_filename().unwrap();
		let id = uuid::Uuid::new_v4().to_string();

		let file_extension = Path::new(&old_name).extension().and_then(OsStr::to_str);

		if let Some(extension) = file_extension {
			if !["jpg", "jpeg", "png", "gif"].contains(&extension) {
				return HttpResponse::BadRequest().body("Unsupported file type");
			}
		} else {
			return HttpResponse::BadRequest().body("No file extension found");
		}

		let filepath = format!("{}/{}", path.display(), id);
		println!("filepath: {}", filepath);
		let mut file = match std::fs::File::create(filepath) {
			Ok(file) => file,
			Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
		};

		while let Some(chunk) = field.next().await {
			let data = match chunk {
				Ok(data) => data,
				Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
			};

			file = match file.write_all(&data) {
				Ok(_) => file,
				Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
			};
		}

		let db = db.lock().unwrap();
		db.insert_file(&id, &old_name).unwrap();

		file = match file.sync_all() {
			Ok(_) => file,
			Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
		};

		match file.flush() {
			Ok(_) => file,
			Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
		};
	}

	HttpResponse::Ok().body("File uploaded")
}

#[get("/image/{id}")]
async fn get_image(db: web::Data<Arc<Mutex<Database>>>, id: web::Path<String>) -> impl Responder {
	let db = db.lock().unwrap();
	let file = db.get_file(&id).unwrap();

	let path = format!("{}/uploads", CONFIG.directory);
	let path = Path::new(&path);

	let file = match std::fs::read(format!("{}/{}", path.display(), file.id)) {
		Ok(file) => file,
		Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
	};

	let file_extension = path.extension().and_then(OsStr::to_str);

	let content_type = match file_extension {
		Some("jpg") => "image/jpeg",
		Some("jpeg") => "image/jpeg",
		Some("png") => "image/png",
		Some("gif") => "image/gif",
		_ => "application/octet-stream",
	};

	HttpResponse::Ok().content_type(content_type).body(file)
}

#[get("download/{id}")]
async fn download_file(db: web::Data<Arc<Mutex<Database>>>, id: web::Path<String>) -> impl Responder {
	let db = db.lock().unwrap();
	let db_file = db.get_file(&id).unwrap();

	let path = format!("{}/uploads", CONFIG.directory);
	let path = Path::new(&path);

	let file = match std::fs::read(format!("{}/{}", path.display(), db_file.id)) {
		Ok(file) => file,
		Err(e) => return HttpResponse::InternalServerError().body(format!("Error: {}", e)),
	};

	let stream = futures_util::stream::once(async move { Ok::<_, std::io::Error>(actix_web::web::Bytes::from(file)) });

	HttpResponse::Ok()
		.content_type("application/octet-stream")
		.append_header(("Content-Disposition", format!("attachment; filename={}", db_file.name)))
		.streaming(stream)
}
