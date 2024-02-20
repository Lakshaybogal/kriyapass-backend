use crate::models::{AppState, NewTicket, Ticket};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Responder,
};

use serde_json::json;
use uuid::Uuid;

#[post("/create_ticket")]
async fn generate_ticket(ticket_data: Json<NewTicket>, pool: Data<AppState>) -> impl Responder {
    let ticket_insert = sqlx::query_as!(
        Ticket,
        "INSERT INTO tickets (event_id ,ticket_id, event_name,ticket_type, price, availability)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING *",
        ticket_data.event_id,
        Uuid::new_v4(),
        ticket_data.event_name,
        ticket_data.ticket_type,
        ticket_data.price,
        ticket_data.availability
    )
    .fetch_one(&pool.db)
    .await;

    match ticket_insert {
        Ok(ticket) => HttpResponse::Ok().json(json!({
            "status" : "sucess",
            "data" : ticket
        })),
        Err(err) => {
            eprintln!("Failed to create ticket: {:?}", err);
            HttpResponse::InternalServerError().json(json!({
                "Error": err.to_string(),
            }))
        }
    }
}
