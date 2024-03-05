use core::fmt;
use std::{
    env,
    future::{ready, Ready},
};

use actix_web::error::{ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{web, FromRequest, HttpRequest};
use futures::executor::block_on;
use serde::{Deserialize, Serialize};

use crate::models::AppState;
use crate::models::User;
use crate::token;

#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtMiddleware {
    pub user: User,
}

impl FromRequest for JwtMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let data = req.app_data::<web::Data<AppState>>().unwrap();

        let access_token = match req.cookie("access_token") {
            Some(c) => c.value().to_string(),
            None => {
                let json_error = ErrorResponse {
                    status: "fail".to_string(),
                    message: format!("{:?}", "cookie not found"),
                };
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let access_token_details = match token::verify_jwt_token(
            env::var("ACCESS_SECRET_KEY").unwrap().to_string(),
            &access_token,
        ) {
            Ok(token_details) => token_details,
            Err(e) => {
                let json_error = ErrorResponse {
                    status: "fail".to_string(),
                    message: format!("{:?}", e),
                };
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        let user_id = uuid::Uuid::parse_str(&access_token_details.user_id.to_string()).unwrap();

        let user_exists_result = async move {
            let query_result =
                sqlx::query_as!(User, "SELECT * FROM users WHERE user_id = $1", user_id)
                    .fetch_optional(&data.db)
                    .await;

            match query_result {
                Ok(Some(user)) => Ok(user),
                Ok(None) => {
                    let json_error = ErrorResponse {
                        status: "fail".to_string(),
                        message: "the user belonging to this token no logger exists".to_string(),
                    };
                    Err(ErrorUnauthorized(json_error))
                }
                Err(_) => {
                    let json_error = ErrorResponse {
                        status: "error".to_string(),
                        message: "Faled to check user existence".to_string(),
                    };
                    Err(ErrorInternalServerError(json_error))
                }
            }
        };

        match block_on(user_exists_result) {
            Ok(user) => ready(Ok(JwtMiddleware { user })),
            Err(error) => ready(Err(error)),
        }
    }
}
