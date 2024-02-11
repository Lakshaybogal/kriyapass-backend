use std::collections::HashMap;

use crate::models::{AppState, Event, EventWithTickets, NewEvent, Ticket};
use actix_web::{
    get, post,
    web::{Data, Json, Path},
    HttpResponse, Responder,
};

use serde_json::json;

// Handler for the create_user route
#[post("/add_event")]
async fn create_event(event_data: Json<NewEvent>, pool: Data<AppState>) -> impl Responder {
    let event = event_data.into_inner();

    // Execute the SQL query to insert a new event into the database
    let query_res = sqlx::query_as!(
        Event,
        "INSERT INTO events (event_name, event_date, event_location, event_description)
         VALUES ($1, $2, $3, $4)
         RETURNING *",
        event.event_name,
        event.event_date,
        event.event_location,
        event.event_description
    )
    .fetch_one(&pool.db)
    .await;

    // Handle the query result
    match query_res {
        Ok(event) => {
            HttpResponse::Ok().json(json!(
                {
                    "status" : "success",
                    "Data" : event
                }
            ))
        }
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

#[get("/get_event/{event_id}")]
async fn get_event(event_id: Path<i32>, pool: Data<AppState>) -> impl Responder {
    let event_id = event_id.into_inner();
    let event_data = sqlx::query!(
        "SELECT
            t.ticket_id AS ticket_id, 
            t.ticket_uuid AS ticket_uuid,
            t.ticket_type AS ticket_type,
            t.price AS price,
            t.availability AS availability,
            e.event_id AS event_id,
            e.event_name AS event_name,
            e.event_date AS event_date,
            e.event_location AS event_location,
            e.event_description AS event_description,
            e.event_status AS event_status
        FROM
            tickets AS t
        LEFT JOIN
            events AS e ON t.event_id = e.event_id 
        WHERE
        e.event_id = $1
        ",
        event_id
    )
    .fetch_all(&pool.db)
    .await;

    match event_data {
        Ok(events_ticket_data) => {
            let mut events_map: HashMap<i32, EventWithTickets> = HashMap::new();

            for data in events_ticket_data {
                // Create or update the event entry in the map
                let event_tic = events_map.entry(event_id).or_insert(EventWithTickets {
                    event: Event {
                        event_id,
                        event_name: data.event_name.clone(),
                        event_description: data.event_description,
                        event_location: Some(data.event_location),
                        event_date: data.event_date,
                        event_status: data.event_status,
                    },
                    tickets: Vec::new(),
                });

                // Add the ticket to the event's tickets list
                event_tic.tickets.push(Ticket {
                    ticket_id: data.ticket_id,
                    ticket_uuid: data.ticket_uuid,
                    ticket_type: Some(data.ticket_type),
                    event_id: Some(data.event_id),
                    event_name: Some(data.event_name),
                    price: data.price,
                    availability: Some(data.availability),
                });
            }

            let events: Vec<EventWithTickets> = events_map
                .into_iter()
                .map(|(_, event_tic)| event_tic)
                .collect();

            HttpResponse::Ok().json(json!({
                "status": "success",
                "data": events
            }))
        }
        Err(err) => HttpResponse::NotFound().json(json!({
            "error" : "Event Not Found",
            "system_error" : err.to_string()
        })),
    }
}

#[get("/get_events")]
async fn get_events(pool: Data<AppState>) -> impl Responder {
    // Query the database to get events with associated tickets
    let event_data = sqlx::query!(
        "
        SELECT
            t.ticket_id AS ticket_id, 
            t.ticket_uuid AS ticket_uuid,
            t.ticket_type AS ticket_type,
            t.price AS price,
            t.availability AS availability,
            e.event_id AS event_id,
            e.event_name AS event_name,
            e.event_date AS event_date,
            e.event_location AS event_location,
            e.event_description AS event_description,
            e.event_status AS event_status
        FROM
            tickets AS t
        LEFT JOIN
            events AS e ON t.event_id = e.event_id
        ",
    )
    .fetch_all(&pool.db)
    .await;

    match event_data {
        Ok(events_ticket_data) => {
            let mut events_map: HashMap<i32, EventWithTickets> = HashMap::new();

            for data in events_ticket_data {
                let event_id = data.event_id.unwrap_or_default();

                // Create or update the event entry in the map
                let event_tic = events_map.entry(event_id).or_insert(EventWithTickets {
                    event: Event {
                        event_id,
                        event_name: data.event_name.clone().unwrap_or_default(),
                        event_description: data.event_description,
                        event_location: data.event_location,
                        event_date: data.event_date.unwrap_or_default(),
                        event_status: data.event_status,
                    },
                    tickets: Vec::new(),
                });

                // Add the ticket to the event's tickets list
                event_tic.tickets.push(Ticket {
                    ticket_id: data.ticket_id,
                    ticket_uuid: data.ticket_uuid,
                    ticket_type: Some(data.ticket_type),
                    event_id: data.event_id,
                    event_name: data.event_name,
                    price: data.price,
                    availability: Some(data.availability),
                });
            }

            let events: Vec<EventWithTickets> = events_map
                .into_iter()
                .map(|(_, event_tic)| event_tic)
                .collect();

            HttpResponse::Ok().json(json!({
                "status": "success",
                "data": events
            }))
        }
        Err(err) => HttpResponse::NotFound().json(json!({
            "error" : "Event Not Found",
            "system_error" : err.to_string()
        })),
    }
}
