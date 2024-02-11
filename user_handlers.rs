use crate::models::{AppState, Login, NewUser, User};
use actix_web::{
    get, post,
    web::{Data, Json},
    HttpResponse, Responder,
};
use bcrypt::{hash, verify, DEFAULT_COST};
use serde_json::json;
// Handler for the add_user route
#[post("/add_user")]
pub async fn add_user(user: Json<NewUser>, pool: Data<AppState>) -> impl Responder {
    let user_data = user.into_inner();
    // Execute the SQL query to insert a new user into the database
    let password = hash(user_data.password, DEFAULT_COST).expect("Failed to hash password");
    let query_res = sqlx::query_as!(
        User,
        "INSERT INTO users (username, email, password, first_name, last_name, phone_number)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING *",
        user_data.username,
        user_data.email,
        password,
        user_data.first_name,
        user_data.last_name,
        user_data.phone_number
    )
    .fetch_one(&pool.db)
    .await;

    // Handle the query result
    match query_res {
        Ok(data) => {
            let res = json!({
                "status" : "success",
                "Data" :  data,
            });
            HttpResponse::Ok().json(res)
        }
        Err(err) => {
            eprintln!("Failed to add user: {:?}", err);
            HttpResponse::InternalServerError().json(json!({
                "Error": err.to_string(),
            }))
        }
    }
}

// Handler for the get_user route
#[get("/get_user")]
async fn get_user(data: Json<Login>, pool: Data<AppState>) -> impl Responder {
    let login_data = data.into_inner();
    let query_res = sqlx::query_as!(
        User,
        "SELECT * FROM users
        WHERE email = $1 ",
        login_data.email
    )
    .fetch_one(&pool.db)
    .await;

    match query_res {
        Ok(data) => {
            let password_match = verify(login_data.password, &data.password);
            match password_match {
                Ok(matched) => {
                    if matched {
                        HttpResponse::Ok().json(json!({
                            "status" : "success",
                            "data" : data
                        }))
                    } else {
                        HttpResponse::BadRequest().json(json!({
                            "error" : "Wrong Password",
                        }))
                    }
                }
                Err(_) => HttpResponse::BadRequest().json(json!({
                    "error" : "Error occurred during verification",
                })),
            }
        }
        Err(err) => HttpResponse::BadRequest().json(json!({
            "error" : err.to_string(),
        })),
    }
}
