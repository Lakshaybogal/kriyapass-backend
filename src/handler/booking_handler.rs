use crate::models::{Booking, NewBooking};
use crate::{jwt_auth, AppState};
use actix_web::{
    delete, patch, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

#[post("/book_ticket")]
pub async fn book_ticket(
    booking: Json<NewBooking>,
    pool: Data<AppState>,
    jwt_guard: jwt_auth::JwtMiddleware,
) -> impl Responder {
    let booking = booking.into_inner();
    let user_id = jwt_guard.user.user_id;
    // Calculate total price if both quantity and price are present
    let total_price = booking.quantity * booking.price;
    let booking_id = Uuid::new_v4();

    let query_res = sqlx::query_as!(
        Booking,
        "INSERT INTO bookings (booking_id, event_name, ticket_id, user_id, quantity, total_price)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *",
        booking_id,
        booking.event_name,
        booking.ticket_id,
        user_id,
        booking.quantity,
        total_price
    )
    .fetch_one(&pool.db)
    .await;

    let response = match query_res {
        Ok(data) => {
            let update_res = sqlx::query!(
                "UPDATE tickets SET availability = availability - $1 WHERE ticket_id = $2",
                data.quantity,
                data.ticket_id
            )
            .execute(&pool.db)
            .await;

            match update_res {
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
    };

    response
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
