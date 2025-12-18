use rocket::{State, Route};
use rocket::serde::json::Json;
use rocket::http::Status;
use rocket::response::content::RawHtml;
use crate::person::Person;
use crate::AppState;

pub fn get_routes() -> Vec<Route> {
    routes![landing_page, health, persons, single_person, add_person, update_person, delete_person]
}

#[get("/")]
fn landing_page(state: &State<AppState>) -> RawHtml<String> {
    use chrono::Utc;
    let current_time = Utc::now().to_rfc3339();
    let response_body = format!("Rust-Rocket {} <br> Current UTC time: {}", state.greeting_text, current_time);
    RawHtml(response_body)
}

#[get("/health")]
fn health() -> &'static str {
    "OK"
}

#[get("/api/persons")]
fn persons(state: &State<AppState>) -> Result<Json<Vec<Person>>, Status> {
    let persons = state.person_collection.read()
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(persons.clone()))
}

#[get("/api/person/<id>")]
fn single_person(id: u32, state: &State<AppState>) -> Result<Json<Person>, Status> {
    let persons_guard = state.person_collection.read()
        .map_err(|_| Status::InternalServerError)?;
    let filtered = persons_guard.iter().find(|t| t.id == id);
    match filtered {
        Some(filtered) => Ok(Json(filtered.clone())),
        None => Err(Status::NotFound),
    }
}

#[post("/api/person", data = "<person>")]
fn add_person(person: Json<Person>, state: &State<AppState>) -> Result<Status, Status> {
    let mut persons_guard = state.person_collection.write()
        .map_err(|_| Status::InternalServerError)?;
    let filtered = persons_guard.iter().any(|t| t.id == person.id);
    if !filtered {
        persons_guard.push(person.into_inner());
        Ok(Status::Created)
    } else {
        Err(Status::Conflict)
    }
}

#[put("/api/person", data = "<person>")]
fn update_person(person: Json<Person>, state: &State<AppState>) -> Result<Status, Status> {
    let mut persons_guard = state.person_collection.write()
        .map_err(|_| Status::InternalServerError)?;
    let person = person.into_inner();
    let filtered = persons_guard.iter_mut().find(|t| t.id == person.id);
    match filtered {
        Some(p) => {
            p.age = person.age;
            p.date = person.date;
            p.name = person.name;
            Ok(Status::NoContent)
        }
        None => Err(Status::NotFound),
    }
}

#[delete("/api/person/<id>")]
fn delete_person(id: u32, state: &State<AppState>) -> Result<Status, Status> {
    let mut persons_guard = state.person_collection.write()
        .map_err(|_| Status::InternalServerError)?;
    let index = persons_guard.iter().position(|t| t.id == id);
    match index {
        Some(index) => {
            persons_guard.remove(index);
            Ok(Status::NoContent)
        }
        None => Err(Status::NotFound),
    }
}
