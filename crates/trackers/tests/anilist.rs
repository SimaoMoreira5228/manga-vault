use std::io::{Read, Write};
use std::net::TcpListener;

use trackers::anilist::AniListProvider;
use trackers::{Credentials, Tokens, TrackerProvider};

fn serve_graphql(responder: impl Fn(&str) -> String + Send + 'static) -> String {
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
async fn paste_token_is_validated_through_viewer_query() {
	let server = serve_graphql(|request| {
		assert!(request.to_lowercase().contains("authorization: bearer pasted-token"));
		assert!(request.contains("Viewer"));
		r#"{"data":{"Viewer":{"id":9,"name":"dewn"}}}"#.into()
	});

	let provider = AniListProvider::new(server);
	let tokens = provider
		.authenticate(&Credentials::Paste {
			token: "pasted-token".into(),
		})
		.await
		.unwrap();
	assert_eq!(tokens.access_token, "pasted-token");
	assert_eq!(tokens.account_label.as_deref(), Some("dewn"));
	assert_eq!(provider.id(), "anilist");
}

#[tokio::test]
async fn search_maps_media_fields_and_skips_novels() {
	let server = serve_graphql(|request| {
		assert!(request.contains("format_not_in:[NOVEL]"));
		r#"{"data":{"Page":{"media":[
			{"id":30013,"chapters":144,"title":{"romaji":"Vinland","english":"Vinland Saga"},"coverImage":{"large":"c.png"}},
			{"id":1,"chapters":null,"title":{"romaji":"Other","english":null},"coverImage":{"large":null}}
		]}}}"#
			.into()
	});
	let provider = AniListProvider::new(server);
	let tokens = Tokens {
		account_label: None,
		access_token: "t".into(),
		refresh_token: None,
	};
	let hits = provider.search(&tokens, "vinland").await.unwrap();

	assert_eq!(hits.len(), 2);
	assert_eq!(hits[0].remote_id, "30013");
	assert_eq!(hits[0].title, "Vinland Saga");
	assert_eq!(hits[0].total_chapters, Some(144.0));
	assert_eq!(hits[1].title, "Other");
}

#[tokio::test]
async fn update_progress_marks_completed_at_final_chapter() {
	let server = serve_graphql(|request| {
		if request.contains("SaveMediaListEntry") {
			assert!(request.contains("\"progress\":144"));
			assert!(request.contains("COMPLETED"), "final chapter must mark COMPLETED");
			return r#"{"data":{"SaveMediaListEntry":{"id":777}}}"#.into();
		}
		assert!(request.contains("mediaListEntry"));
		r#"{"data":{"Media":{"chapters":144,"status":"FINISHED","mediaListEntry":{"progress":143,"score":80,"status":"CURRENT"}}}}"#
			.into()
	});
	let provider = AniListProvider::new(server);
	let tokens = Tokens {
		account_label: None,
		access_token: "t".into(),
		refresh_token: None,
	};
	provider.update_progress(&tokens, "30013", 144.0).await.unwrap();
}

#[tokio::test]
async fn update_progress_keeps_existing_status_midway() {
	let server = serve_graphql(|request| {
		if request.contains("SaveMediaListEntry") {
			assert!(request.contains("\"progress\":12"));
			assert!(request.contains("PAUSED"), "existing status must be preserved");
			return r#"{"data":{"SaveMediaListEntry":{"id":777}}}"#.into();
		}
		r#"{"data":{"Media":{"chapters":200,"status":"NOT_YET_RELEASED","mediaListEntry":{"progress":11,"score":80,"status":"PAUSED"}}}}"#
			.into()
	});
	let provider = AniListProvider::new(server);
	let tokens = Tokens {
		account_label: None,
		access_token: "t".into(),
		refresh_token: None,
	};
	provider.update_progress(&tokens, "30013", 12.0).await.unwrap();
}

#[tokio::test]
async fn track_state_normalizes_score_to_unit_scale() {
	let server = serve_graphql(|_| {
		r#"{"data":{"Media":{"chapters":100,"status":"FINISHED","mediaListEntry":{"progress":50,"score":80,"status":"CURRENT"}}}}"#
			.into()
	});
	let provider = AniListProvider::new(server);
	let tokens = Tokens {
		account_label: None,
		access_token: "t".into(),
		refresh_token: None,
	};
	let state = provider.track_state(&tokens, "42").await.unwrap();
	assert_eq!(state.score, Some(0.8));
	assert_eq!(state.chapters_read, Some(50.0));
	assert_eq!(state.total_chapters, Some(100.0));
}
