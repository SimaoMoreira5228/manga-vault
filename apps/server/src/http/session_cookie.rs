#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionCookie {
	pub secure: bool,
	pub cross_origin: bool,
}

impl SessionCookie {
	pub fn new(secure: bool, cross_origin: bool) -> Self {
		Self {
			secure: secure || cross_origin,
			cross_origin,
		}
	}

	pub fn name(self) -> &'static str {
		if self.secure {
			"__Host-session_token"
		} else {
			"session_token"
		}
	}

	fn attributes(self) -> String {
		let mut attributes = if self.cross_origin {
			String::from("; SameSite=None")
		} else {
			String::from("; SameSite=Lax")
		};
		if self.secure {
			attributes.push_str("; Secure");
		}
		attributes
	}

	pub fn set_header(self, token: uuid::Uuid, ttl: std::time::Duration) -> String {
		format!(
			"{}={token}; Path=/; HttpOnly; Max-Age={}{}",
			self.name(),
			ttl.as_secs(),
			self.attributes()
		)
	}

	pub fn clear_header(self) -> String {
		format!("{}=; Path=/; HttpOnly; Max-Age=0{}", self.name(), self.attributes())
	}
}

#[cfg(test)]
mod tests {
	use super::SessionCookie;

	#[test]
	fn secure_cookies_carry_the_host_prefix() {
		let cookie = SessionCookie::new(true, false);
		assert_eq!(cookie.name(), "__Host-session_token");
		let header = cookie.set_header(uuid::Uuid::nil(), std::time::Duration::from_secs(60));
		assert_eq!(
			header,
			"__Host-session_token=00000000-0000-0000-0000-000000000000; Path=/; HttpOnly; Max-Age=60; SameSite=Lax; Secure"
		);
	}

	#[test]
	fn the_host_prefix_is_never_emitted_without_secure() {
		for cross_origin in [false, true] {
			let header =
				SessionCookie::new(true, cross_origin).set_header(uuid::Uuid::nil(), std::time::Duration::from_secs(1));
			assert_eq!(header.contains("__Host-"), header.contains("; Secure"));
		}
		assert!(
			!SessionCookie::new(false, false)
				.set_header(uuid::Uuid::nil(), std::time::Duration::from_secs(1))
				.contains("; Secure")
		);
		assert!(
			!SessionCookie::new(false, false)
				.set_header(uuid::Uuid::nil(), std::time::Duration::from_secs(1))
				.contains("__Host-")
		);
	}

	#[test]
	fn insecure_cookies_omit_the_prefix_so_http_development_works() {
		let cookie = SessionCookie::new(false, false);
		assert_eq!(cookie.name(), "session_token");
		let header = cookie.clear_header();
		assert_eq!(header, "session_token=; Path=/; HttpOnly; Max-Age=0; SameSite=Lax");
		assert!(!header.contains("__Host-"));
	}

	#[test]
	fn cross_origin_cookies_must_be_secure() {
		let cookie = SessionCookie::new(false, true);
		assert!(cookie.secure, "SameSite=None is rejected by browsers without Secure");
		assert_eq!(cookie.name(), "__Host-session_token");
		assert!(
			cookie
				.set_header(uuid::Uuid::nil(), std::time::Duration::from_secs(1))
				.ends_with("; SameSite=None; Secure")
		);
	}

	#[test]
	fn same_origin_deployments_keep_lax_even_when_secure() {
		let cookie = SessionCookie::new(true, false);
		let header = cookie.set_header(uuid::Uuid::nil(), std::time::Duration::from_secs(1));
		assert!(header.contains("; SameSite=Lax"));
		assert!(!header.contains("SameSite=None"));
	}
}
