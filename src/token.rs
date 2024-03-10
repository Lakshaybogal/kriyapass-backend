use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenDetails {
    pub token: Option<String>,
    pub user_id: Uuid,
    pub expires_in: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub exp: i64,
}

pub fn generate_jwt_token(
    user_id: Uuid,
    private_key: &str,
    time: i64,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let now = chrono::Utc::now();
    let expires_in = (now + chrono::Duration::minutes(60 * time * 24)).timestamp();

    let claims = TokenClaims {
        sub: user_id.to_string(),
        exp: expires_in,
    };

    let token = encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(private_key.as_bytes()),
    )?;

    Ok(TokenDetails {
        token: Some(token),
        user_id,
        expires_in: Some(expires_in),
    })
}

pub fn verify_jwt_token(
    public_key: &str,
    token: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);

    let decoded = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(public_key.as_bytes()),
        &validation,
    )?;

    let user_id = Uuid::parse_str(&decoded.claims.sub).unwrap();

    Ok(TokenDetails {
        token: None,
        user_id,
        expires_in: None,
    })
}
