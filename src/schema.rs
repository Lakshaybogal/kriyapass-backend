// @generated automatically by Diesel CLI.

diesel::table! {
    bookings (booking_id) {
        booking_id -> Int4,
        user_id -> Nullable<Int4>,
        ticket_id -> Nullable<Int4>,
        quantity -> Int4,
        total_price -> Numeric,
        booking_date -> Nullable<Timestamp>,
    }
}

diesel::table! {
    events (event_id) {
        event_id -> Int4,
        #[max_length = 255]
        event_name -> Varchar,
        event_date -> Date,
        #[max_length = 255]
        event_location -> Varchar,
        event_description -> Nullable<Text>,
        event_status -> Nullable<Bool>,
    }
}

diesel::table! {
    tickets (ticket_id) {
        ticket_id -> Int4,
        event_id -> Nullable<Int4>,
        #[max_length = 50]
        ticket_type -> Varchar,
        price -> Numeric,
        availability -> Int4,
    }
}

diesel::table! {
    users (user_id) {
        user_id -> Int4,
        #[max_length = 255]
        username -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        #[max_length = 100]
        first_name -> Nullable<Varchar>,
        #[max_length = 100]
        last_name -> Nullable<Varchar>,
        #[max_length = 20]
        phone_number -> Nullable<Varchar>,
        registration_date -> Nullable<Timestamp>,
    }
}

diesel::joinable!(bookings -> tickets (ticket_id));
diesel::joinable!(bookings -> users (user_id));
diesel::joinable!(tickets -> events (event_id));

diesel::allow_tables_to_appear_in_same_query!(
    bookings,
    events,
    tickets,
    users,
);
