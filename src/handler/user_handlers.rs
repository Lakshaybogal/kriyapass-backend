use crate::jwt_auth; // Add missing token module
use crate::models::{AppState, Login, NewUser, User};
use crate::token::{generate_jwt_token, verify_jwt_token};
use actix_web::{
    cookie::time::Duration,
    cookie::Cookie,
    delete, get, post,
    web::{Data, Json},
    HttpRequest, HttpResponse, Responder,
};
use bcrypt::{hash, verify, DEFAULT_COST};
// use chrono::Duration;
use serde_json::json;
use std::env;
use uuid::Uuid;

// Handler for the add_user route
#[post("/register")]
pub async fn add_user(user: Json<NewUser>, pool: Data<AppState>) -> impl Responder {
    let user_data = user.into_inner();
    let password = hash(user_data.password, DEFAULT_COST).expect("Failed to hash password");

    let query_res = sqlx::query_as!(User, "INSERT INTO users (user_id ,username, email, password, first_name, last_name, phone_number) VALUES ($1, $2, $3, $4, $5, $6,$7) RETURNING *", Uuid::new_v4(), user_data.username, user_data.email, password, user_data.first_name, user_data.last_name, user_data.phone_number)
        .fetch_one(&pool.db)
        .await;

    match query_res {
        Ok(data) => {
            let access_token = match generate_jwt_token(
                data.user_id,
                &env::var("ACCESS_SECRET_KEY").unwrap(),
                1,
            ) {
                Ok(token) => token,
                Err(err) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({ "error": err.to_string() }));
                }
            };

            let refresh_token =
                match generate_jwt_token(data.user_id, &env::var("REFRESH_SECRET_KEY").unwrap(), 1)
                {
                    Ok(token) => token,
                    Err(err) => {
                        return HttpResponse::InternalServerError()
                            .json(json!({ "error": err.to_string() }));
                    }
                };

            let refresh_cookie =
                Cookie::build("refresh_token", refresh_token.token.clone().unwrap())
                    .path("/")
                    .http_only(true)
                    .same_site(actix_web::cookie::SameSite::None)
                    .secure(true)
                    .max_age(Duration::days(
                        env::var("REFRESH_TOKEN_AGE")
                            .unwrap()
                            .parse::<i64>()
                            .unwrap(),
                    ))
                    .finish();

            let access_cookie = Cookie::build("access_token", access_token.token.clone().unwrap())
                .path("/")
                .http_only(true)
                .secure(true)
                .same_site(actix_web::cookie::SameSite::None)
                .max_age(Duration::days(
                    env::var("ACCESS_TOKEN_AGE")
                        .unwrap()
                        .parse::<i64>()
                        .unwrap(),
                ))
                .finish();

            HttpResponse::Ok()
                .cookie(access_cookie)
                .cookie(refresh_cookie)
                .json(json!({ "status" : "success", "data" : data }))
        }

        Err(err) => {
            // Handle error from token generation
            HttpResponse::InternalServerError()
                .json(json!({ "status" : "fail", "error" : err.to_string() }))
        }
    }
}

// Handler for the get_user route
#[post("/login")]
async fn get_user(data: Json<Login>, pool: Data<AppState>) -> impl Responder {
    let login_data = data.into_inner();
    let user = match sqlx::query_as!(
        User,
        "SELECT * FROM users
        WHERE email = $1 ",
        login_data.email
    )
    .fetch_one(&pool.db)
    .await
    {
        Ok(user) => user,
        Err(err) => {
            return HttpResponse::InternalServerError()
                .json(json!({ "error": format!("Failed to fetch user: {}", err) }));
        }
    };

    // Verify the password
    let password_match = match verify(login_data.password, &user.password) {
        Ok(matched) => matched,
        Err(_) => {
            return HttpResponse::BadRequest()
                .json(json!({ "error": "Error occurred during verification" }))
        }
    };

    if password_match {
        // Generate JWT token
        let access_token =
            match generate_jwt_token(user.user_id, &env::var("ACCESS_SECRET_KEY").unwrap(), 1) {
                Ok(token) => token,
                Err(err) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({ "error": err.to_string() }))
                }
            };

        // Generate refresh token
        let refresh_token =
            match generate_jwt_token(user.user_id, &env::var("REFRESH_SECRET_KEY").unwrap(), 7) {
                Ok(token) => token,
                Err(err) => {
                    return HttpResponse::InternalServerError()
                        .json(json!({ "error": err.to_string() }))
                }
            };

        // Build access and refresh cookies
        let refresh_cookie = Cookie::build("refresh_token", refresh_token.token.clone().unwrap())
            .http_only(true)
            .secure(true)
            .same_site(actix_web::cookie::SameSite::None)
            .max_age(Duration::days(7))
            .finish();
        let access_cookie = Cookie::build("access_token", access_token.token.clone().unwrap())
            .http_only(true)
            .secure(true)
            .same_site(actix_web::cookie::SameSite::None)
            .max_age(Duration::days(1))
            .finish();

        // Return response with cookies and user data
        HttpResponse::Ok()
            .cookie(access_cookie)
            .cookie(refresh_cookie)
            .json(json!({
                "status": "success",
                "data": {
                    "username": user.username,
                    "email": user.email,
                    "first_name": user.first_name,
                    "last_name": user.last_name,
                    "phone_number": user.phone_number
                },
                "access_token": access_token.token.unwrap(),
            }))
    } else {
        // Password doesn't match
        HttpResponse::BadRequest().json(json!({ "error": "Wrong Password" }))
    }
}

#[get("/refresh")]
async fn refresh_access_token_handler(req: HttpRequest, pool: Data<AppState>) -> impl Responder {
    // Extract refresh token from Authorization header
    let refresh_token = match req.cookie("refresh_token") {
        Some(c) => c.value().to_string(),
        None => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": "Cookie not present"}));
        }
    };
    // Verify refresh token
    let refresh_token_details = match verify_jwt_token(
        &env::var("REFRESH_SECRET_KEY").unwrap(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::Forbidden()
                .json(serde_json::json!({"status": "fail", "message": format!("Failed to verify refresh token: {:?}", e)}));
        }
    };
    // Fetch user data
    let user = match sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE user_id = $1",
        refresh_token_details.user_id
    )
    .fetch_one(&pool.db)
    .await
    {
        Ok(user) => user,
        Err(err) => {
            return HttpResponse::InternalServerError().json(
                json!({ "status": "fail", "message": format!("Failed to fetch user: {:?}", err) }),
            );
        }
    };
    // Generate new access token
    let access_token = match generate_jwt_token(
        refresh_token_details.user_id,
        &env::var("ACCESS_SECRET_KEY").unwrap(),
        1,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            return HttpResponse::BadGateway()
                .json(serde_json::json!({"status": "fail", "message": format!("Failed to generate access token: {:?}", e)}));
        }
    };
    // Set new access token cookie
    let access_cookie = Cookie::build("access_token", access_token.token.clone().unwrap())
        .http_only(true)
        .secure(true)
        .same_site(actix_web::cookie::SameSite::None)
        .max_age(Duration::days(1))
        .finish();
    // Return response with new access token, refresh token, and user data
    HttpResponse::Ok()
        // .cookie(refresh_cookie)
        .cookie(access_cookie)
        .json(json!({
            "status": "success",
            "data": {
                "username": user.username,
                "email": user.email,
                "first_name": user.first_name,
                "last_name": user.last_name,
                "phone_number": user.phone_number
            },
            "access_token": access_token.token,
        }))
}

#[get("/logout")]
async fn logout() -> impl Responder {
    // Set new access token cookie
    let access_cookie = Cookie::build("access_token", "")
        .http_only(true)
        .secure(true)
        .same_site(actix_web::cookie::SameSite::None)
        .max_age(Duration::days(-1))
        .finish();
    let refresh_cookie = Cookie::build("refresh_token", "")
        .http_only(true)
        .secure(true)
        .same_site(actix_web::cookie::SameSite::None)
        .max_age(Duration::days(-1))
        .finish();
    // Return response with new access token, refresh token, and user data
    HttpResponse::Ok()
        .cookie(access_cookie)
        .cookie(refresh_cookie)
        .json(json!({
            "status": "success",
            "data": "Logout",
        }))
}
// Function to fetch user data by ID

#[delete("/delete")]
async fn delete_user(pool: Data<AppState>, jwt_guard: jwt_auth::JwtMiddleware) -> impl Responder {
    let user_id = jwt_guard.user.user_id;
    match sqlx::query!("DELETE FROM users where user_id = $1", user_id)
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
