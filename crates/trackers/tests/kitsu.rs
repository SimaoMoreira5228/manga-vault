use std::io::{Read, Write};
use std::net::TcpListener;

use trackers::anilist::AniListProvider;
use trackers::kitsu::KitsuProvider;
use trackers::{Credentials, Tokens, TrackerProvider};

fn serve_multi(responder: impl Fn(&str) -> String + Send + 'static) -> String {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	let address = listener.local_addr().unwrap();
	std::thread::spawn(move || {
		for stream in listener.incoming().flatten() {
			let mut stream = stream;
			let mut buffer = [0u8; 16384];
			let _ = stream.read(&mut buffer);
			let request = String::from_utf8_lossy(&buffer).into_owned();
			let body = responder(&request);
			let response = format!(
				"HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
				body.len()
			);
			if stream.write_all(response.as_bytes()).is_err() {
				break;
			}
		}
	});
	format!("http://{address}")
}

#[tokio::test]
async fn kitsu_credentials_exchange_and_search() {
	let server = serve_multi(|request| {
		if request.contains("/api/oauth/token") {
			assert!(request.contains("grant_type=password"));
			assert!(request.contains("client_id="));
			return r#"{"access_token":"kt","refresh_token":"rt"}"#.into();
		}
		if request.contains("/api/edge/users") {
			return r#"{"data":[{"id":"4567","attributes":{"name":"dewn"}}]}"#.into();
		}
		assert!(request.contains("filter[text]"));
		r#"{"data":[
			{"id":"12","attributes":{"canonicalTitle":"Berserk","chapterCount":364,"posterImage":{"small":"p.png"}}},
			{"id":"13","attributes":{"canonicalTitle":"Other","chapterCount":null,"posterImage":{"small":null}}}
		]}"#
		.into()
	});

	let provider = KitsuProvider::new(server);
	let tokens = provider
		.authenticate(&Credentials::UsernamePassword {
			username: "dewn".into(),
			password: "secret".into(),
		})
		.await
		.unwrap();
	assert_eq!(tokens.access_token, "kt");
	assert_eq!(tokens.account_label.as_deref(), Some("dewn"));

	let hits = provider.search(&tokens, "berserk").await.unwrap();
	assert_eq!(hits.len(), 2);
	assert_eq!(hits[0].remote_id, "12");
	assert_eq!(hits[0].total_chapters, Some(364.0));
}

#[tokio::test]
async fn kitsu_progress_creates_entry_when_missing() {
	let server = serve_multi(|request| {
		if request.contains("/api/oauth/token") {
			return r#"{"access_token":"kt"}"#.into();
		}
		if request.contains("/api/edge/users") {
			return r#"{"data":[{"id":"4567"}]}"#.into();
		}
		if request.contains("/api/edge/manga/99") {
			return r#"{"data":{"attributes":{"chapterCount":50}}}"#.into();
		}
		if request.contains("filter[media_id]") {
			return r#"{"data":[]}"#.into();
		}
		if request.contains("POST") && request.contains("libraryEntries") {
			assert!(request.contains("\"progress\":3"));
			assert!(request.contains("CURRENT"));
			return r#"{"data":{"id":"new-entry"}}"#.into();
		}
		r#"{"data":[]}"#.into()
	});

	let provider = KitsuProvider::new(server);
	let tokens = Tokens {
		account_label: None,
		access_token: "kt".into(),
		refresh_token: None,
	};
	provider.update_progress(&tokens, "99", 3.0).await.unwrap();
}

#[tokio::test]
async fn kitsu_progress_patches_existing_entry_and_completes() {
	let server = serve_multi(|request| {
		if request.contains("/api/oauth/token") {
			return r#"{"access_token":"kt"}"#.into();
		}
		if request.contains("/api/edge/users") {
			return r#"{"data":[{"id":"4567"}]}"#.into();
		}
		if request.contains("/api/edge/manga/99") {
			return r#"{"data":{"attributes":{"chapterCount":5}}}"#.into();
		}
		if request.contains("filter[media_id]") {
			return r#"{"data":[{"id":"entry-1","attributes":{"progress":4,"status":"CURRENT"}}]}"#.into();
		}
		if request.contains("PATCH") && request.contains("library-entries/entry-1") {
			assert!(request.contains("\"progress\":5"));
			assert!(request.contains("COMPLETED"));
			return r#"{"data":{"id":"entry-1"}}"#.into();
		}
		r#"{"data":[]}"#.into()
	});

	let provider = KitsuProvider::new(server);
	let tokens = Tokens {
		account_label: None,
		access_token: "kt".into(),
		refresh_token: None,
	};
	provider.update_progress(&tokens, "99", 5.0).await.unwrap();
}

#[tokio::test]
async fn anilist_still_registered() {
	let provider = AniListProvider::new("http://127.0.0.1:1");
	assert_eq!(provider.id(), "anilist");
}
