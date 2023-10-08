use actix_web::{web, HttpResponse};

use crate::{errors::AppError, models::Course, state::AppState};

pub async fn new_course(
    new_course: web::Json<Course>,
    app_state: web::Data<AppState>,
) -> Result<HttpResponse, AppError> {
    sqlx::query!(
        r#"
        insert into courses_c4 (tutor_id, course_name)
        Values ($1, $2)"#,
        new_course.tutor_id,
        new_course.course_name.clone(),
    )
    .execute(&app_state.db)
    .await?;

    Ok(HttpResponse::Created().json(""))
}

pub async fn get_courses_for_tutor(
    app_state: web::Data<AppState>,
    params: web::Path<i32>,
) -> Result<HttpResponse, AppError> {
    let tutor_id: i32 = params.into_inner();

    let course_rows = sqlx::query!(
        r#"
        select course_id, tutor_id, course_name, posted_time
        from courses_c4
        where tutor_id = $1"#,
        tutor_id
    )
    .map(|course_row| Course {
        course_id: Some(course_row.course_id),
        tutor_id: course_row.tutor_id,
        course_name: course_row.course_name.clone(),
        posted_time: Some(chrono::NaiveDateTime::from(course_row.posted_time.unwrap())),
    })
    .fetch_all(&app_state.db)
    .await?;

    if course_rows.len() > 0 {
        Ok(HttpResponse::Ok().json(course_rows))
    } else {
        Ok(HttpResponse::NotFound().json(""))
    }
}
pub async fn get_single_courses(
    app_state: web::Data<AppState>,
    params: web::Path<(i32, i32)>,
) -> Result<HttpResponse, AppError> {
    let (tutor_id, course_id) = params.into_inner();

    let course = sqlx::query!(
        r#"
        select course_id, tutor_id, course_name, posted_time
        from courses_c4
        where course_id = $1
        and tutor_id = $2
        limit 1
        "#,
        course_id,
        tutor_id
    )
    .fetch_one(&app_state.db)
    .await?;

    Ok(HttpResponse::Ok().json(Course {
        course_id: Some(course.course_id),
        tutor_id: course.tutor_id,
        course_name: course.course_name.clone(),
        posted_time: Some(chrono::NaiveDateTime::from(course.posted_time.unwrap())),
    }))
}
