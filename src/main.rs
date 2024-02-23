fn main() {
	let config = config::load_config();
	api::run(&config).unwrap();
}
