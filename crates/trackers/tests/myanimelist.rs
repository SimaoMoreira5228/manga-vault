use std::io::{Read, Write};
use std::net::TcpListener;

use trackers::myanimelist::MyAnimeListProvider;
use trackers::{Credentials, Tokens, TrackerError, TrackerProvider};

fn serve(responder: impl Fn(&str, &str) -> String + Send + 'static) -> String {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	let address = listener.local_addr().unwrap();
	std::thread::spawn(move || {
		for stream in listener.incoming().flatten() {
			let mut stream = stream;
			let mut buffer = [0u8; 16384];
			let read = stream.read(&mut buffer).unwrap_or(0);
			let request = String::from_utf8_lossy(&buffer[..read]).into_owned();
			let (head, body) = request.split_once("\r\n\r\n").unwrap_or((&request, ""));
			let response_body = responder(head, body);
			let status = if response_body == "UNAUTHORIZED" { "401 Unauthorized" } else { "200 OK" };
			let response = format!(
				"HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response_body}",
				response_body.len()
			);
			if stream.write_all(response.as_bytes()).is_err() {
				break;
			}
		}
	});
	format!("http://{address}")
}

fn provider_for_base(oauth_base: String, api_base: String) -> MyAnimeListProvider {
	MyAnimeListProvider::new(oauth_base, api_base, "test-client-id")
}

#[tokio::test]
async fn oauth_code_exchanges_for_tokens_and_resolves_label() {
	let server = serve(|head, body| {
		if head.starts_with("POST /token") {
			assert!(body.contains("grant_type=authorization_code"));
			assert!(body.contains("code=abc123"));
			assert!(body.contains("code_verifier=verifier-1"));
			assert!(body.contains("client_id=test-client-id"));
			assert!(body.contains("redirect_uri=https%3A%2F%2Fvault.example%2Fcallback"));
			r#"{"access_token":"at-1","refresh_token":"rt-1"}"#.into()
		} else {
			assert!(head.contains("GET /users/@me"));
			assert!(head.to_lowercase().contains("authorization: bearer at-1"));
			r#"{"name":"mal-user"}"#.into()
		}
	});

	let provider = provider_for_base(server.clone(), server);
	let tokens = provider
		.authenticate(&Credentials::OAuthCode {
			code: "abc123".into(),
			verifier: Some("verifier-1".into()),
			redirect_uri: Some("https://vault.example/callback".into()),
		})
		.await
		.unwrap();
	assert_eq!(tokens.access_token, "at-1");
	assert_eq!(tokens.refresh_token.as_deref(), Some("rt-1"));
	assert_eq!(tokens.account_label.as_deref(), Some("mal-user"));
}

#[tokio::test]
async fn refresh_rotates_tokens_with_bearer_header() {
	let server = serve(|head, body| {
		assert!(head.starts_with("POST /token"));
		assert!(head.to_lowercase().contains("authorization: bearer old-at"));
		assert!(body.contains("grant_type=refresh_token"));
		assert!(body.contains("refresh_token=rt-old"));
		r#"{"access_token":"at-new","refresh_token":"rt-new"}"#.into()
	});

	let provider = provider_for_base(server.clone(), server);
	let refreshed = provider
		.refresh(&Tokens {
			account_label: Some("mal-user".into()),
			access_token: "old-at".into(),
			refresh_token: Some("rt-old".into()),
		})
		.await
		.unwrap();
	assert_eq!(refreshed.access_token, "at-new");
	assert_eq!(refreshed.refresh_token.as_deref(), Some("rt-new"));
}

#[tokio::test]
async fn search_maps_hits_and_filters_novels() {
	let server = serve(|_head, _body| {
		r#"{"data":[
			{"node":{"id":1,"title":"Vinland Saga","media_type":"manga","num_chapters":218,"main_picture":{"large":"c.png"}}},
			{"node":{"id":2,"title":"Some Novel","media_type":"novel","num_chapters":9}}
		]}"#
			.into()
	});

	let provider = provider_for_base(server.clone(), server);
	let hits = provider
		.search(
			&Tokens {
				account_label: None,
				access_token: "at".into(),
				refresh_token: None,
			},
			"vinland",
		)
		.await
		.unwrap();
	assert_eq!(hits.len(), 1);
	assert_eq!(hits[0].remote_id, "1");
	assert_eq!(hits[0].total_chapters, Some(218.0));
}

#[tokio::test]
async fn track_state_reads_my_list_status_when_present() {
	let server = serve(|_head, _body| {
		r#"{"id":1,"num_chapters":218,"my_list_status":{"status":"reading","score":8,"num_chapters_read":42}}"#.into()
	});

	let provider = provider_for_base(server.clone(), server);
	let state = provider
		.track_state(
			&Tokens {
				account_label: None,
				access_token: "at".into(),
				refresh_token: None,
			},
			"1",
		)
		.await
		.unwrap();
	assert_eq!(state.chapters_read, Some(42.0));
	assert_eq!(state.score, Some(8.0));
	assert_eq!(state.remote_status.as_deref(), Some("reading"));
	assert_eq!(state.total_chapters, Some(218.0));
}

#[tokio::test]
async fn progress_marks_completed_at_final_chapter_and_keeps_existing_status() {
	let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
	let state_response = r#"{"id":1,"num_chapters":100,"my_list_status":{"status":"on_hold","score":0,"num_chapters_read":10}}"#;

	let listener_calls = calls.clone();
	let server = serve(move |head, body| {
		listener_calls.lock().unwrap().push((head.to_string(), body.to_string()));
		if head.contains("/manga/7") && head.starts_with("GET") {
			state_response.into()
		} else {
			String::from("{}")
		}
	});

	let provider = provider_for_base(server.clone(), server);
	provider
		.update_progress(
			&Tokens {
				account_label: None,
				access_token: "at".into(),
				refresh_token: None,
			},
			"7",
			55.0,
		)
		.await
		.unwrap();
	provider
		.update_progress(
			&Tokens {
				account_label: None,
				access_token: "at".into(),
				refresh_token: None,
			},
			"7",
			100.0,
		)
		.await
		.unwrap();

	let logged = calls.lock().unwrap();
	let puts: Vec<&(String, String)> = logged.iter().filter(|(head, _)| head.starts_with("PUT")).collect();
	assert_eq!(puts.len(), 2);
	assert!(puts[0].1.contains("status=on_hold"), "keeps existing non-final status");
	assert!(puts[0].1.contains("num_chapters_read=55"));
	assert!(puts[1].1.contains("status=completed"), "final chapter completes the entry");
	assert!(puts[1].1.contains("num_chapters_read=100"));
	for (head, _) in puts {
		assert!(head.contains("/manga/7/my_list_status"));
	}
}

#[tokio::test]
async fn unauthorized_responses_surface_as_retryable_rotation_signal() {
	let server = serve(|_head, _body| String::from("UNAUTHORIZED"));

	let provider = provider_for_base(server.clone(), server);
	let error = provider
		.track_state(
			&Tokens {
				account_label: None,
				access_token: "expired".into(),
				refresh_token: None,
			},
			"1",
		)
		.await
		.unwrap_err();
	assert!(matches!(error, TrackerError::Unauthorized(_)));
}
