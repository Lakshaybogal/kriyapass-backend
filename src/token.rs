use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub user_id: uuid::Uuid,
    // pub email: String,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: i64,
}

pub fn generate_jwt_token(
    user_id: uuid::Uuid,
    private_key: String,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let decoded_private_key = private_key;

    let now = chrono::Utc::now();
    let mut token_details = TokenDetails {
        user_id,
        expires_in: Some((now + chrono::Duration::minutes(60)).timestamp()),
        token: None,
    };

    let claims = TokenClaims {
        sub: token_details.user_id.to_string(),
        exp: token_details.expires_in.unwrap(),
    };

    let token = jsonwebtoken::encode(
        &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &jsonwebtoken::EncodingKey::from_secret(decoded_private_key.as_bytes()),
    )
    .unwrap();
    token_details.token = Some(token);
    Ok(token_details)
}

pub fn verify_jwt_token(
    public_key: String,
    token: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let decoded_public_key = public_key;
    let validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);

    let decoded = jsonwebtoken::decode::<TokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(decoded_public_key.as_bytes()),
        &validation,
    )?;

    let user_id = Uuid::parse_str(decoded.claims.sub.as_str()).unwrap();

    Ok(TokenDetails {
        token: None,
        user_id,
        expires_in: None,
    })
}
