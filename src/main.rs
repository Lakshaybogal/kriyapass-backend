// Import necessary dependencies
use actix_web::{
    web::{self, scope, Data},
    App, HttpResponse, HttpServer, Responder,
};

// Import module
mod database;
mod handler;
mod models;
use crate::database::connect_database;
use crate::models::AppState;
use handler::{
    booking_handler::book_ticket,
    event_handlers::{create_event, get_event, get_event_by_user, get_events, mark_done},
    ticket_handlers::generate_ticket,
    user_handlers::{add_user, get_user},
};

// Main function
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = connect_database().await;
   
    // Start the HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .route("/", web::get().to(greet))
            .service(add_user)
            .service(get_user)
            .service(create_event)
            .service(generate_ticket)
            .service(
                scope("/get_event")
                    .service(get_event)
                    .service(get_event_by_user),
            )
            .service(get_events)
            .service(book_ticket)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await

}

// Handler for the root route
async fn greet() -> impl Responder {
    HttpResponse::Ok().body("Jai Mata Di")
}
