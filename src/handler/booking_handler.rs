use crate::models::{Booking, NewBooking};
use crate::{jwt_auth, AppState};
use actix_web::{
    delete, get, patch, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

#[post("/book_ticket")]
async fn book_ticket(
    booking: Json<NewBooking>,
    pool: Data<AppState>,
    jwt_guard: jwt_auth::JwtMiddleware,
) -> HttpResponse {
    let booking = booking.into_inner();
    let user_id = jwt_guard.user.user_id;

    // Parse price to an integer
    let price: i32 = booking.price.parse().unwrap_or(0);
    let quantity: i32 = booking.quantity.parse().unwrap_or(0);
    // Calculate total price if both quantity and price are present
    let total_price = quantity * price;
    let booking_id = Uuid::new_v4();

    let query_res = sqlx::query_as!(
        Booking,
        "INSERT INTO bookings (booking_id, event_name, ticket_id, user_id, quantity, total_price,verified)
        VALUES ($1, $2, $3, $4, $5, $6,$7)
        RETURNING *",
        booking_id,
        booking.event_name,
        booking.ticket_id,
        user_id,
        booking.quantity,
        total_price.to_string(),
        false
    )
    .fetch_one(&pool.db)
    .await;

    match query_res {
        Ok(data) => {
            match sqlx::query!(
                "UPDATE tickets SET availability = availability - $1 WHERE ticket_id = $2",
                quantity, // Use quantity directly for subtraction
                data.ticket_id
            )
            .execute(&pool.db)
            .await
            {
                Ok(_) => HttpResponse::Ok().json(json!({
                    "status": "success",
                    "data": data
                })),
                Err(err) => HttpResponse::InternalServerError().json(json!({
                    "status": "fail",
                    "error": format!("Failed to update ticket availability: {}", err)
                })),
            }
        }
        Err(err) => HttpResponse::BadRequest().json(json!({
            "status": "fail",
            "error": format!("Failed to book ticket: {}", err)
        })),
    }
}

#[get("/bookings")]
pub async fn get_bookings(
    pool: Data<AppState>,
    jwt_guard: jwt_auth::JwtMiddleware,
) -> impl Responder {
    let user_id = jwt_guard.user.user_id;
    match sqlx::query_as!(
        Booking,
        "Select * from bookings where user_id = $1",
        user_id
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(tickets) => HttpResponse::Ok().json(json!({
            "status" : "success",
            "data" : tickets,
        })),
        Err(err) => HttpResponse::InternalServerError().json(json!({
            "status" : "fail",
            "error" : err.to_string()
        })),
    }
}

#[patch("/booking_verification/{booking_id}")]
async fn ticket_verification(booking_id: Path<Uuid>, pool: Data<AppState>) -> impl Responder {
    let booking_id = booking_id.into_inner();

    let result = sqlx::query_as!(
        Booking,
        "UPDATE bookings SET verified = TRUE WHERE booking_id = $1 AND verified = FALSE RETURNING *",
        booking_id
    )
    .fetch_optional(&pool.db)
    .await;

    match result {
        Ok(Some(_)) => HttpResponse::Ok().json(json!({ "status": "success" })),
        Ok(None) => HttpResponse::AlreadyReported()
            .json(json!({ "error": "Already scanned or not valid ",
                "status": "fail" })),
        Err(err) => HttpResponse::InternalServerError()
            .json(json!({ "error": err.to_string(), "status": "fail" })),
    }
}

#[delete("/delete/{booking_id}")]
async fn delete_booking(booking_id: Path<Uuid>, pool: Data<AppState>) -> impl Responder {
    let booking_id = booking_id.into_inner();
    match sqlx::query!("DELETE FROM bookings where booking_id = $1", booking_id)
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
