use crate::{
    jwt_auth,
    models::{AppState, Event, NewEvent},
};
use actix_web::{
    delete, get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use chrono::{NaiveDate, Utc};
use serde_json::json;
use uuid::Uuid;

// Handler for the create_user route
#[post("/events/add_event")]
async fn create_event(
    jwt_guard: jwt_auth::JwtMiddleware,
    event_data: Json<NewEvent>,
    pool: Data<AppState>,
) -> impl Responder {
    let event = event_data.into_inner();
    let event_date = NaiveDate::parse_from_str(&event.event_date, "%Y-%m-%d")
        .expect("Failed to parse event_date");
    // Execute the SQL query to insert a new event into the database
    let query_res = sqlx::query_as!(
        Event,
        "INSERT INTO events (event_id ,user_id ,event_name, event_date, event_location, event_description,event_status)
         VALUES ($1, $2, $3, $4, $5,$6,$7)
         RETURNING *",
        Uuid::new_v4(),
        jwt_guard.user.user_id,
        event.event_name,
        event_date,
        event.event_location,
        event.event_description,
        false,
    )
    .fetch_one(&pool.db)
    .await;

    // Handle the query result
    match query_res {
        Ok(event) => HttpResponse::Ok().json(json!(
            {
                "status" : "success",
                "data" : event
            }
        )),
        Err(err) => {
            // Log the error
            eprintln!("Failed to create event: {:?}", err);
            // Return internal server error response
            HttpResponse::InternalServerError().json(json!({
                "Error": err.to_string(),
            }))
        }
    }
}

#[get("/event/{event_id}")]
async fn get_event(event_id: Path<Uuid>, pool: Data<AppState>) -> impl Responder {
    let event_id = event_id.into_inner();
    // let user_id = token_details.user_id;
    let event_data = sqlx::query_as!(
        Event,
        "
        SELECT
           *
        FROM
            events
        WHERE
            event_id = $1
        ",
        event_id
    )
    .fetch_one(&pool.db)
    .await;

    match event_data {
        Ok(events) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": events
        })),
        Err(err) => HttpResponse::NotFound().json(json!({
            "error" : "Event Not Found",
            "system_error" : err.to_string()
        })),
    }
}

#[get("/userevents")]
async fn get_event_by_user(
    jwt_guard: jwt_auth::JwtMiddleware,
    pool: Data<AppState>,
) -> impl Responder {
    let user_id = jwt_guard.user.user_id;
    // let user_id = token_details.user_id;
    let event_data = sqlx::query_as!(
        Event,
        "
        SELECT
           *
        FROM
            events
        WHERE
            user_id = $1
        ",
        user_id
    )
    .fetch_all(&pool.db)
    .await;

    match event_data {
        Ok(events) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": events
        })),
        Err(err) => HttpResponse::NotFound().json(json!({
            "error" : "Event Not Found",
            "system_error" : err.to_string()
        })),
    }
}

#[get("/events")]
async fn get_events(pool: Data<AppState>) -> impl Responder {
    // Query the database to get events with associated tickets
    let event_data = sqlx::query_as!(
        Event,
        "
        SELECT
           *
        FROM
            events
        ",
    )
    .fetch_all(&pool.db)
    .await;

    match event_data {
        Ok(events) => HttpResponse::Ok().json(json!({
            "status": "success",
            "data": events
        })),
        Err(err) => HttpResponse::NotFound().json(json!({
            "error" : "Event Not Found",
            "system_error" : err.to_string()
        })),
    }
}

#[delete("/delete/{event_id}")]
async fn delete_event(event_id: Path<Uuid>, pool: Data<AppState>) -> impl Responder {
    let event_id = event_id.into_inner();
    match sqlx::query!("DELETE FROM events where event_id = $1", event_id)
        .execute(&pool.db)
        .await
    {
        Ok(_) => HttpResponse::Ok().json(json!({
            "status" : "success"
        })),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status" : "fail",
            "error" : err.to_string()
        })),
    }
}

pub async fn check_and_update_events(pool: Data<AppState>) {
    println!("Running scheduled task...");
    let current_date = Utc::now().naive_utc().date();

    // Execute SQL query to mark events as true where event_date <= current_date
    let query_res = sqlx::query!(
        "UPDATE events SET event_status = TRUE WHERE event_date <= $1",
        current_date
    )
    .execute(&pool.db)
    .await;

    match query_res {
        Ok(_) => {
            println!("Events marked");
        }
        Err(err) => {
            eprintln!("Failed to mark events: {:?}", err);
        }
    }
}
