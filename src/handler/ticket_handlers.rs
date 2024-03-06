use crate::{
    jwt_auth,
    models::{AppState, NewTicket, Ticket},
};
use actix_web::{
    delete, get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use serde_json::json;
use uuid::Uuid;

#[post("/create_ticket")]
async fn generate_ticket(
    ticket_data: Json<NewTicket>,
    pool: Data<AppState>,
    jwt_gaurd: jwt_auth::JwtMiddleware,
) -> impl Responder {
    if jwt_gaurd.user.user_id == ticket_data.user_id {
        let ticket_insert = sqlx::query_as!(
            Ticket,
            "INSERT INTO tickets (event_id ,ticket_id, event_name ,ticket_type, price, availability)
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
                    "status" : "fail",
                    "Error": err.to_string(),
                }))
            }
        }
    } else {
        HttpResponse::NonAuthoritativeInformation().json(json!({
            "status" : "fail",
            "message": "Unauthorize User",
        }))
    }
}
#[get("/get_ticket/{event_id}")]
async fn get_ticket(event_id: Path<Uuid>, pool: Data<AppState>) -> impl Responder {
    let event_id = event_id.into_inner();
    match sqlx::query_as!(
        Ticket,
        "
        SELECT * FROM tickets WHERE event_id = $1
        ",
        event_id
    )
    .fetch_all(&pool.db)
    .await
    {
        Ok(data) => HttpResponse::Ok().json(json!({
            "status" : "success",
            "data" : data
        })),
        Err(err) => HttpResponse::BadGateway().json(json!(
            {
                "status" : "fail",
                "error" : err.to_string()
            }
        )),
    }
}
#[delete("/delete/{ticket_id}")]
async fn delete_ticket(ticket_id: Path<Uuid>, pool: Data<AppState>) -> impl Responder {
    let ticket_id = ticket_id.into_inner();
    match sqlx::query!("DELETE FROM tickets where ticket_id = $1", ticket_id)
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
