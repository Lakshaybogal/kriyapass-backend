use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::Postgres, Pool};
use uuid::Uuid;
// Define the application state
pub struct AppState {
    pub db: Pool<Postgres>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub registration_date: Option<NaiveDateTime>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Event {
    pub event_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_name: String,
    pub event_date: NaiveDate,
    pub event_location: Option<String>,
    pub event_description: Option<String>,
    pub event_status: bool,
}
#[derive(Debug, Deserialize, Serialize)]
pub struct Ticket {
    pub ticket_id: Uuid,
    pub event_id: Option<Uuid>,
    pub event_name: Option<String>,
    pub ticket_type: Option<String>,
    pub price: i32,
    pub availability: Option<i32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Booking {
    pub booking_id: Uuid,
    pub user_id: Option<Uuid>,
    pub event_id: Option<Uuid>,
    pub event_name: Option<String>,
    pub ticket_id: Option<Uuid>,
    pub quantity: i32,
    pub total_price: i32,
    pub booking_date: Option<NaiveDateTime>,
    pub verified: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Login {
    pub email: String,
    pub password: String,
}

// Define the NewEvent struct for creating new events
#[derive(Debug, Deserialize, Serialize)]
pub struct NewEvent {
    pub event_name: String,
    pub event_date: String,
    pub event_location: String,
    pub event_description: String,
}

#[derive(Debug, Deserialize, serde::Serialize)]
pub struct EventWithTickets {
    pub event: Event,
    pub tickets: Vec<Ticket>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewTicket {
    pub event_id: Uuid,
    pub email: String,
    pub event_name: String,
    pub ticket_type: String,
    pub price: i32,
    pub availability: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewBooking {
    pub user_id: Uuid,
    pub event_name: String,
    pub ticket_id: Uuid,
    pub quantity: i32,
    pub price: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub phone_number: String,
}

// CREATE TABLE Users (
//     user_id SERIAL PRIMARY KEY,
//     username VARCHAR(255) NOT NULL,
//     email VARCHAR(255) NOT NULL,
//     password VARCHAR(255) NOT NULL,
//     first_name VARCHAR(100),
//     last_name VARCHAR(100),
//     phone_number VARCHAR(20),
//     registration_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP
// );

// CREATE TABLE Events (
//     event_id SERIAL PRIMARY KEY,
//     event_name VARCHAR(255) NOT NULL,
//     event_date DATE NOT NULL,
//     event_location VARCHAR(255) NOT NULL,
//     event_description TEXT,
//     event_status BOOLEAN DEFAULT FALSE
// );

// CREATE TABLE Tickets (
//     ticket_id SERIAL PRIMARY KEY,
//     event_id INT,
//     ticket_type VARCHAR(50) NOT NULL,
//     price DECIMAL(10, 2) NOT NULL,
//     availability INT NOT NULL,
//     FOREIGN KEY (event_id) REFERENCES Events(event_id)
// );

// CREATE TABLE Bookings (
//     booking_id SERIAL PRIMARY KEY,
//     user_id INT,
//     ticket_id INT,
//     quantity INT NOT NULL,
//     total_price DECIMAL(10, 2) NOT NULL,
//     booking_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
//     FOREIGN KEY (user_id) REFERENCES Users(user_id),
//     FOREIGN KEY (ticket_id) REFERENCES Tickets(ticket_id)
// );
