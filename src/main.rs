#[macro_use] extern crate rocket;

mod person;
mod routes;

use std::sync::RwLock;
use std::env;
use rocket::Config;

pub struct AppState {
    pub person_collection: RwLock<Vec<person::Person>>,
    pub greeting_text: String,
}

#[launch]
fn rocket() -> _ {
    let greeting_text = env::var("GREETING_TEXT").unwrap_or_else(|_| "Hi!".to_string());

    let config = Config {
        address: "0.0.0.0".parse().unwrap(),
        port: 8080,
        ..Config::default()
    };

    rocket::custom(config)
        .manage(AppState {
            person_collection: RwLock::new(person::create_person_collection()),
            greeting_text,
        })
        .mount("/", routes::get_routes())
}
