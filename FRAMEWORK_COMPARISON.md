# Rust Web Frameworks Comparison: Actix vs Axum vs Rocket

## Overview
This document compares three popular Rust web frameworks: Actix Web, Axum, and Rocket.

---

## 1. Cargo.toml

### Actix Web
```toml
[package]
name = "actix-app"

[dependencies]
actix-web = "4.9.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
env_logger = "0.11"
```

### Axum
```toml
[package]
name = "axum-app"

[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
tower = "0.4"
tower-http = { version = "0.5", features = ["trace"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

### Rocket
```toml
[package]
name = "rocket-app"

[dependencies]
rocket = { version = "0.5", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
chrono = { version = "0.4", features = ["serde"] }
```

**Comparison:**
- **Actix**: Minimal dependencies, uses env_logger
- **Axum**: Most dependencies (requires tokio, tower ecosystem)
- **Rocket**: Simplest, batteries-included

---

## 2. main.rs

### Actix Web
```rust
use actix_web::{middleware::Logger, web, App, HttpServer};
use std::sync::RwLock;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let shared_state = web::Data::new(AppState {
        person_collection: RwLock::new(person::create_person_collection()),
        greeting_text,
    });

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(shared_state.clone())
            .service(landing_page)
            .service(persons)
            .service(add_person)
            .service(update_person)
            .service(delete_person)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
```

### Axum
```rust
use axum::Router;
use tower_http::trace::TraceLayer;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let shared_state = Arc::new(AppState {
        person_collection: RwLock::new(person::create_person_collection()),
        greeting_text,
    });

    let app = Router::new()
        .merge(routes::create_routes())
        .layer(TraceLayer::new_for_http())
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .unwrap();
    
    axum::serve(listener, app).await.unwrap();
}
```

### Rocket
```rust
#[macro_use] extern crate rocket;

use std::sync::RwLock;
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
```

**Comparison:**
- **Actix**: `#[actix_web::main]`, HttpServer pattern, returns `Result`
- **Axum**: `#[tokio::main]`, manual TcpListener setup, uses Arc
- **Rocket**: `#[launch]`, simplest setup, no Arc needed

---

## 3. routes.rs - Route Definition

### Actix Web
```rust
use actix_web::{delete, get, post, put, web, HttpResponse};

#[get("/")]
async fn landing_page(data: web::Data<AppState>) -> HttpResponse {
    HttpResponse::Ok().body(response_body)
}

#[get("/api/persons")]
async fn persons(data: web::Data<AppState>) -> Result<HttpResponse, HttpAppError> {
    let persons = data.person_collection.read()?;
    Ok(HttpResponse::Ok().json(persons.deref()))
}

#[get("/api/person/{id}")]
async fn single_person(path: web::Path<u32>, data: web::Data<AppState>) -> Result<HttpResponse, HttpAppError> {
    let id = path.into_inner();
    // ...
}

#[post("/api/person")]
async fn add_person(person: web::Json<Person>, data: web::Data<AppState>) -> Result<HttpResponse, HttpAppError> {
    let person = person.into_inner();
    // ...
    Ok(HttpResponse::Created().finish())
}
```

### Axum
```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use std::sync::Arc;

pub fn create_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(landing_page))
        .route("/api/persons", get(persons))
        .route("/api/person/:id", get(single_person))
        .route("/api/person", post(add_person))
}

async fn landing_page(State(state): State<Arc<AppState>>) -> Html<String> {
    Html(response_body)
}

async fn persons(State(state): State<Arc<AppState>>) -> Result<Json<Vec<Person>>, HttpAppError> {
    let persons = state.person_collection.read()?;
    Ok(Json(persons.clone()))
}

async fn single_person(
    Path(id): Path<u32>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Person>, HttpAppError> {
    // ...
}

async fn add_person(
    State(state): State<Arc<AppState>>,
    Json(person): Json<Person>,
) -> Result<StatusCode, HttpAppError> {
    // ...
    Ok(StatusCode::CREATED)
}
```

### Rocket
```rust
use rocket::{State, Route};
use rocket::serde::json::Json;
use rocket::http::Status;

pub fn get_routes() -> Vec<Route> {
    routes![landing_page, persons, single_person, add_person]
}

#[get("/")]
fn landing_page(state: &State<AppState>) -> RawHtml<String> {
    RawHtml(response_body)
}

#[get("/api/persons")]
fn persons(state: &State<AppState>) -> Result<Json<Vec<Person>>, Status> {
    let persons = state.person_collection.read()
        .map_err(|_| Status::InternalServerError)?;
    Ok(Json(persons.clone()))
}

#[get("/api/person/<id>")]
fn single_person(id: u32, state: &State<AppState>) -> Result<Json<Person>, Status> {
    // ...
}

#[post("/api/person", data = "<person>")]
fn add_person(person: Json<Person>, state: &State<AppState>) -> Result<Status, Status> {
    // ...
    Ok(Status::Created)
}
```

**Comparison:**
- **Actix**: Macro on function `#[get("/path")]`, `web::Data`, `web::Path`, `.into_inner()`
- **Axum**: Function-based routing, `State(state): State<Arc<T>>`, destructured extractors
- **Rocket**: Macro on function `#[get("/path")]`, `&State`, path params in signature

---

## 4. Error Handling

### Actix Web
```rust
use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpAppError {
    #[error("Not found")]
    NotFound,
}

impl ResponseError for HttpAppError {
    fn status_code(&self) -> StatusCode {
        match self {
            HttpAppError::NotFound => StatusCode::NOT_FOUND,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .body(self.to_string())
    }
}
```

### Axum
```rust
use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpAppError {
    #[error("Not found")]
    NotFound,
}

impl IntoResponse for HttpAppError {
    fn into_response(self) -> Response {
        let status = match self {
            HttpAppError::NotFound => StatusCode::NOT_FOUND,
        };
        (status, Json(self.to_string())).into_response()
    }
}
```

### Rocket
```rust
use rocket::http::Status;

// No custom error type needed!
// Just use Status enum directly:
// - Status::NotFound
// - Status::InternalServerError
// - Status::Conflict
// - Status::Created
```

**Comparison:**
- **Actix**: `ResponseError` trait, separate `status_code()` and `error_response()`
- **Axum**: `IntoResponse` trait, simpler implementation
- **Rocket**: Built-in `Status` enum, no custom error handling needed

---

## 5. State Management

### Actix Web
```rust
pub struct AppState {
    pub person_collection: RwLock<Vec<Person>>,
}

// In main.rs
let state = web::Data::new(AppState { ... });

// In handlers
async fn handler(data: web::Data<AppState>) {
    let persons = data.person_collection.read()?;
}
```

### Axum
```rust
pub struct AppState {
    pub person_collection: RwLock<Vec<Person>>,
}

// In main.rs
let state = Arc::new(AppState { ... });

// In handlers
async fn handler(State(state): State<Arc<AppState>>) {
    let persons = state.person_collection.read()?;
}
```

### Rocket
```rust
pub struct AppState {
    pub person_collection: RwLock<Vec<Person>>,
}

// In main.rs
rocket::build().manage(AppState { ... })

// In handlers
fn handler(state: &State<AppState>) {
    let persons = state.person_collection.read()?;
}
```

**Comparison:**
- **Actix**: `web::Data<T>` (Arc wrapper)
- **Axum**: Manual `Arc<T>`
- **Rocket**: `.manage()` (Arc wrapper, simplest)

---

## 6. Path Parameters

### Actix Web
```rust
#[get("/api/person/{id}")]
async fn handler(path: web::Path<u32>) -> Result<HttpResponse, Error> {
    let id = path.into_inner();
    // use id
}
```

### Axum
```rust
async fn handler(Path(id): Path<u32>) -> Result<Json<Person>, Error> {
    // use id directly
}
```

### Rocket
```rust
#[get("/api/person/<id>")]
fn handler(id: u32) -> Result<Json<Person>, Status> {
    // use id directly
}
```

**Comparison:**
- **Actix**: Requires `.into_inner()`
- **Axum**: Destructured in parameter
- **Rocket**: Direct parameter (simplest)

---

## 7. JSON Handling

### Actix Web
```rust
#[post("/api/person")]
async fn add_person(person: web::Json<Person>) -> HttpResponse {
    let person = person.into_inner();
    // use person
    HttpResponse::Created().json(person)
}
```

### Axum
```rust
async fn add_person(Json(person): Json<Person>) -> StatusCode {
    // use person directly
    StatusCode::CREATED
}
```

### Rocket
```rust
#[post("/api/person", data = "<person>")]
fn add_person(person: Json<Person>) -> Status {
    let person = person.into_inner();
    // use person
    Status::Created
}
```

**Comparison:**
- **Actix**: `.into_inner()`, `HttpResponse::Created().json()`
- **Axum**: Destructured, return `StatusCode` or `Json<T>`
- **Rocket**: `.into_inner()`, return `Status`

---

## 8. Async/Sync

### Actix Web
```rust
#[get("/")]
async fn handler() -> HttpResponse {
    // All handlers are async
}
```

### Axum
```rust
async fn handler() -> String {
    // All handlers are async
}
```

### Rocket
```rust
#[get("/")]
fn handler() -> String {
    // Handlers can be sync or async
}

#[get("/async")]
async fn async_handler() -> String {
    // Async when needed
}
```

**Comparison:**
- **Actix**: Always async
- **Axum**: Always async
- **Rocket**: Sync or async (flexible)

---

## Summary Table

| Feature | Actix Web | Axum | Rocket |
|---------|-----------|------|--------|
| **Learning Curve** | Medium | Medium | Easy |
| **Boilerplate** | Medium | Medium | Low |
| **Route Definition** | Macro `#[get]` | Function-based | Macro `#[get]` |
| **State** | `web::Data<T>` | `Arc<T>` | `.manage()` |
| **Path Params** | `.into_inner()` | Destructured | Direct |
| **JSON** | `.into_inner()` | Destructured | `.into_inner()` |
| **Error Handling** | `ResponseError` | `IntoResponse` | `Status` enum |
| **Async** | Required | Required | Optional |
| **Runtime** | Built-in | Tokio | Built-in |
| **Dependencies** | Few | Many | Few |
| **Performance** | Excellent | Excellent | Very Good |
| **Type Safety** | High | High | Very High |
| **Compile Time** | Fast | Slow | Medium |
| **Documentation** | Excellent | Good | Excellent |
| **Community** | Large | Growing | Large |
| **Maturity** | Mature | New | Mature |

---

## Code Complexity Comparison (Same Endpoint)

### GET /api/person/:id

**Actix Web** (7 lines)
```rust
#[get("/api/person/{id}")]
async fn single_person(path: web::Path<u32>, data: web::Data<AppState>) -> Result<HttpResponse, HttpAppError> {
    let id = path.into_inner();
    let persons = data.person_collection.read()?;
    let person = persons.iter().find(|p| p.id == id).ok_or(HttpAppError::NotFound)?;
    Ok(HttpResponse::Ok().json(person))
}
```

**Axum** (6 lines)
```rust
async fn single_person(Path(id): Path<u32>, State(state): State<Arc<AppState>>) -> Result<Json<Person>, HttpAppError> {
    let persons = state.person_collection.read()?;
    let person = persons.iter().find(|p| p.id == id).ok_or(HttpAppError::NotFound)?;
    Ok(Json(person.clone()))
}
```

**Rocket** (6 lines)
```rust
#[get("/api/person/<id>")]
fn single_person(id: u32, state: &State<AppState>) -> Result<Json<Person>, Status> {
    let persons = state.person_collection.read().map_err(|_| Status::InternalServerError)?;
    let person = persons.iter().find(|p| p.id == id).ok_or(Status::NotFound)?;
    Ok(Json(person.clone()))
}
```

**Winner**: Rocket (cleanest, most readable)

---

## Recommendations

### Choose Actix Web if:
- ✅ You need maximum performance
- ✅ You want a mature, battle-tested framework
- ✅ You prefer macro-based routing
- ✅ You need WebSocket support

### Choose Axum if:
- ✅ You want modular, composable design
- ✅ You're already using Tower ecosystem
- ✅ You prefer function-based routing
- ✅ You want fine-grained control

### Choose Rocket if:
- ✅ You want the easiest learning curve
- ✅ You prioritize developer experience
- ✅ You want type-safe routing
- ✅ You need rapid prototyping
- ✅ You want minimal boilerplate

---

## Migration Difficulty

| From → To | Difficulty | Time Estimate |
|-----------|------------|---------------|
| Actix → Axum | Medium | 2-4 hours |
| Actix → Rocket | Easy | 1-2 hours |
| Axum → Actix | Medium | 2-4 hours |
| Axum → Rocket | Easy | 1-2 hours |
| Rocket → Actix | Medium | 2-3 hours |
| Rocket → Axum | Medium | 2-3 hours |

---

## Conclusion

- **Best Performance**: Actix Web
- **Most Flexible**: Axum
- **Easiest to Use**: Rocket 🏆
- **Best for Beginners**: Rocket
- **Best for Large Teams**: Rocket (less cognitive load)
- **Best for Microservices**: Axum (modular)
- **Best All-Rounder**: Rocket or Actix Web
