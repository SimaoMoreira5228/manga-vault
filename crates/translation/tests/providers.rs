use std::io::{Read, Write};
use std::net::TcpListener;

use translation::{OllamaTranslator, OpenAiCompatibleTranslator, Translator, sha256_key};

fn serve_once(response: &'static str) -> String {
	let listener = TcpListener::bind("127.0.0.1:0").unwrap();
	let address = listener.local_addr().unwrap();
	std::thread::spawn(move || {
		let (mut stream, _) = listener.accept().unwrap();
		let mut buffer = [0u8; 4096];
		let _ = stream.read(&mut buffer);
		let body = format!(
			"HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{response}",
			response.len()
		);
		stream.write_all(body.as_bytes()).unwrap();
	});
	format!("http://{address}")
}

#[tokio::test]
async fn ollama_translator_extracts_message_content() {
	let server = serve_once(r#"{"message":{"content":"hola mundo"}}"#);
	let translator = OllamaTranslator::new(server, "test-model");
	let translated = translator.translate("hello world", "en", "es").await.unwrap();
	assert_eq!(translated, "hola mundo");
}

#[tokio::test]
async fn openai_compatible_translator_extracts_choice_content() {
	let server = serve_once(r#"{"choices":[{"message":{"content":"bonjour"}}]}"#);
	let translator = OpenAiCompatibleTranslator::new(server, "key", "test-model");
	let translated = translator.translate("hello", "en", "fr").await.unwrap();
	assert_eq!(translated, "bonjour");
}

#[test]
fn cache_key_depends_on_content_target_and_pipeline() {
	let key = sha256_key("same", "pt");
	assert_eq!(key, sha256_key("same", "pt"));
	assert_ne!(key, sha256_key("other", "pt"));
	assert_ne!(key, sha256_key("same", "es"));
}
