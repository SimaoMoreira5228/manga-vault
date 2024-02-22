fn main() {
	let config = config::load_config();
	let db = database::connect(&config);
	api::run(&config, &db)
}
