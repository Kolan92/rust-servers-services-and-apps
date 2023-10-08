use actix_web::{web, App, HttpResponse, HttpServer};
use chrono::Utc;
use dotenv::dotenv;
use handlers::{get_courses_for_tutor, get_single_courses, new_course};
use models::Course;
use sqlx::PgPool;
use state::AppState;
use std::{env, io, sync::Mutex};

#[path = "./iter2/state.rs"]
mod state;

#[path = "./iter2/routes.rs"]
mod routes;

#[path = "./iter2/handlers.rs"]
mod handlers;

#[path = "./iter2/models.rs"]
mod models;

pub fn general_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/health", web::get().to(health_check_handler));
}

pub fn courses_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/courses")
            .route("/", web::post().to(new_course))
            .route("/{tutor_id}", web::get().to(get_courses_for_tutor))
            .route("/{tutor_id}/{course_id}", web::get().to(get_single_courses)),
    );
}

pub async fn health_check_handler(app_state: web::Data<AppState>) -> HttpResponse {
    let health_check_response = &app_state.health_check_response;
    let mut visit_count = app_state.visit_count.lock().unwrap();

    *visit_count += 1;

    HttpResponse::Ok().json(format!(
        "Status {}, visited {} times",
        health_check_response, visit_count
    ))
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db_pool = PgPool::connect(&database_url).await.unwrap();

    let shared_data = web::Data::new(AppState {
        health_check_response: "Healthy".to_string(),
        visit_count: Mutex::new(0),
        db: db_pool,
    });

    let app = move || {
        App::new()
            .app_data(shared_data.clone())
            .configure(general_routes)
            .configure(courses_routes)
    };

    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}
