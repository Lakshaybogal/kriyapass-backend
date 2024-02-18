use crate::models::{Booking, NewBooking};
use crate::AppState;
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use serde_json::json;
use uuid::Uuid;

#[post("/book_ticket")]
pub async fn book_ticket(booking: Json<NewBooking>, pool: Data<AppState>) -> impl Responder {
    let booking = booking.into_inner();

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
        booking.user_id,
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

