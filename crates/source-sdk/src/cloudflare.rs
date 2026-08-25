pub fn looks_like_cloudflare_protection(text: &str, status_code: Option<u16>, headers: &[(String, String)]) -> bool {
	let markers = [
		"Just a moment",
		"Checking your browser",
		"Attention Required! | Cloudflare",
		"__cf_chl_opt",
		"cf-browser-verification",
		"_Incapsula_Resource",
	];
	if markers.iter().any(|m| text.contains(m)) {
		return true;
	}
	let header_value = |name: &str| headers.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v);
	if status_code == Some(403) && header_value("server").is_some_and(|v| v.contains("cloudflare")) {
		return true;
	}
	if header_value("cf-mitigated").is_some() {
		return true;
	}
	false
}
