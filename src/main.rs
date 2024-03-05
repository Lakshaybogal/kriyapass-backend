// Import necessary dependencies
use actix_cors::Cors;
use actix_rt::{spawn, time::interval};
use actix_web::{
    http::header,
    middleware::Logger,
    web::{self, scope, Data},
    App, HttpResponse, HttpServer, Responder,
};
use tokio::time::Duration; // Add missing imports
                           // Import module
mod database;
mod handler;
mod models;
mod jwt_auth;
mod token;
use crate::database::connect_database;
use crate::models::AppState;
use handler::{
    booking_handler::{book_ticket, delete_booking, ticket_verification},
    event_handlers::{
        check_and_update_events, create_event, delete_event, get_event, get_event_by_user,
        get_events,
    },
    ticket_handlers::{delete_ticket, generate_ticket},
    user_handlers::{add_user, delete_user, get_user, refresh_access_token_handler},
};

// Main function
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "actix_web=info");
    }
    env_logger::init();
    let pool = connect_database().await;
    let poolclone = pool.clone();
    spawn(async move {
        let mut interval = interval(Duration::from_secs(60 * 60 * 12));
        loop {
            interval.tick().await;
            check_and_update_events(Data::new(AppState {
                db: poolclone.clone(),
            }))
            .await;
        }
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .supports_credentials()
            .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
            .allowed_headers(vec![
                header::CONTENT_TYPE,
                header::AUTHORIZATION,
                header::ACCEPT,
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
            ]);
        // Capture pool by reference
        App::new()
            .app_data(Data::new(AppState { db: pool.clone() }))
            .wrap(cors)
            .route("/", web::get().to(greet))
            .service(
                scope("/users")
                    .service(get_user)
                    .service(add_user)
                    .service(refresh_access_token_handler)
                    .service(delete_user),
            )
            .service(
                scope("/tickets")
                    .service(generate_ticket)
                    .service(delete_ticket),
            )
            .service(
                scope("/events")
                    .service(create_event)
                    .service(get_event)
                    .service(get_event_by_user)
                    .service(get_events)
                    .service(delete_event),
            )
            .service(
                scope("/bookings")
                    .service(book_ticket)
                    .service(ticket_verification)
                    .service(delete_booking),
            )
           
            .wrap(Logger::default())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

// Handler for the root route
async fn greet() -> impl Responder {
    HttpResponse::Ok().body("Jai Mata Di")
}
