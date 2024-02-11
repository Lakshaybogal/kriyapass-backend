use crate::models::{AppState, Event, NewTicket, Ticket};
use actix_web::{
    post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use serde_json::json;
use uuid::Uuid;

#[post("/create_ticket/{event_id}")]
async fn generate_ticket(
    event_id: Path<i32>,
    ticket_data: Json<NewTicket>,
    pool: Data<AppState>,
) -> impl Responder {
    let event_id = event_id.into_inner();

    // Retrieve the event details from the database
    let event_query = sqlx::query_as!(Event, "SELECT * FROM events WHERE event_id = $1", event_id)
        .fetch_one(&pool.db)
        .await;

    match event_query {
        Ok(event) => {
            // Generate a UUID for the ticket
            let ticket_uuid = Uuid::new_v4();

            let ticket_insert = sqlx::query_as!(
                Ticket,
                "INSERT INTO tickets (event_id, ticket_uuid, event_name, ticket_type, price, availability)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING ticket_id,ticket_uuid,event_id,event_name,price,availability,ticket_type",
                event_id,
                ticket_uuid,
                event.event_name,
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
        Err(err) => {
            eprintln!("Failed to get event: {:?}", err);
            HttpResponse::NotFound().json(json!({
                "Error": "Event not found",
            }))
        }
    }
}
