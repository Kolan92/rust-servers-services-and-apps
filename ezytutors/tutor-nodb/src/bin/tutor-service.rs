use actix_web::{web, App, HttpResponse, HttpServer};
use chrono::Utc;
use models::Course;
use state::AppState;
use std::{io, sync::Mutex};

#[path = "../state.rs"]
mod state;

#[path = "../routes.rs"]
mod routes;

#[path = "../handlers.rs"]
mod handlers;

#[path = "../models.rs"]
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

pub async fn new_course(
    new_course: web::Json<Course>,
    app_state: web::Data<AppState>,
) -> HttpResponse {
    let next_id = app_state
        .courses
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .map(|course| course.course_id.unwrap_or_default())
        .max_by(|a, b| a.cmp(b))
        .unwrap_or_default()
        + 1;

    let new_course = Course {
        course_id: Some(next_id),
        tutor_id: new_course.tutor_id,
        course_name: new_course.course_name.clone(),
        posted_time: Some(Utc::now().naive_utc()),
    };

    app_state.courses.lock().unwrap().push(new_course);

    HttpResponse::Created().json("")
}

pub async fn get_courses_for_tutor(
    app_state: web::Data<AppState>,
    params: web::Path<i32>,
) -> HttpResponse {
    let tutor_id: i32 = params.into_inner();

    let filtered_courses = app_state
        .courses
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .filter(|course| course.tutor_id == tutor_id)
        .collect::<Vec<Course>>();

    if filtered_courses.len() > 0 {
        HttpResponse::Ok().json(filtered_courses)
    } else {
        HttpResponse::NotFound().json("")
    }
}
pub async fn get_single_courses(
    app_state: web::Data<AppState>,
    params: web::Path<(i32, i32)>,
) -> HttpResponse {
    let (tutor_id, course_id) = params.into_inner();

    let filtered_courses = app_state
        .courses
        .lock()
        .unwrap()
        .clone()
        .into_iter()
        .find(|course| {
            course.tutor_id == tutor_id
                && if let Some(c_id) = course.course_id {
                    c_id == course_id
                } else {
                    false
                }
        });

    if let Some(course) = filtered_courses {
        HttpResponse::Ok().json(course)
    } else {
        HttpResponse::NotFound().json("")
    }
}

#[actix_rt::main]
async fn main() -> io::Result<()> {
    let shared_data = web::Data::new(AppState {
        health_check_response: "Healthy".to_string(),
        visit_count: Mutex::new(0),
        courses: Mutex::new(vec![]),
    });

    let app = move || {
        App::new()
            .app_data(shared_data.clone())
            .configure(general_routes)
            .configure(courses_routes)
    };

    HttpServer::new(app).bind("127.0.0.1:3000")?.run().await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::StatusCode;
    use std::sync::Mutex;

    #[actix_rt::test]
    async fn post_course_test() {
        let course = web::Json(Course {
            course_id: None,
            posted_time: None,
            tutor_id: 1,
            course_name: "test".to_string(),
        });

        let app_state: web::Data<AppState> = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![]),
        });
        let resp = new_course(course, app_state).await;
        assert_eq!(resp.status(), StatusCode::CREATED);
    }

    #[actix_rt::test]
    async fn get_tutor_courses_empty() {
        let app_state: web::Data<AppState> = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![]),
        });
        let tutor_id: web::Path<i32> = web::Path::from(1);
        let resp = get_courses_for_tutor(app_state, tutor_id).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn get_tutor_courses_with_some() {
        let app_state: web::Data<AppState> = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![]),
        });
        let course = web::Json(Course {
            course_id: None,
            posted_time: None,
            tutor_id: 1,
            course_name: "test".to_string(),
        });

        _ = new_course(course, app_state.clone()).await;

        let tutor_id: web::Path<i32> = web::Path::from(1);
        let resp = get_courses_for_tutor(app_state, tutor_id).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[actix_rt::test]
    async fn get_single_courses_empty() {
        let app_state: web::Data<AppState> = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![]),
        });
        let params: web::Path<(i32, i32)> = web::Path::from((1, 1));
        let resp = get_single_courses(app_state, params).await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[actix_rt::test]
    async fn get_single_courses_with_some() {
        let app_state: web::Data<AppState> = web::Data::new(AppState {
            health_check_response: "".to_string(),
            visit_count: Mutex::new(0),
            courses: Mutex::new(vec![]),
        });
        let course = web::Json(Course {
            course_id: None,
            posted_time: None,
            tutor_id: 1,
            course_name: "test".to_string(),
        });

        _ = new_course(course, app_state.clone()).await;

        let params: web::Path<(i32, i32)> = web::Path::from((1, 1));
        let resp = get_single_courses(app_state, params).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
