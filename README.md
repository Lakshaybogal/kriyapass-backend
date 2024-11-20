# kriyapass-backend
use actix_cors::Cors;
use actix_web::{http::header, web, App, HttpServer};

fn is_valid_origin(origin: &str) -> bool {
    // List of allowed origins
    let allowed_origins = vec![
        "http://localhost:3000",
        "http://example.com",
        "http://another-origin.com",
    ];

    allowed_origins.contains(&origin)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin_fn(|origin, _req_head| is_valid_origin(origin))
                    .supports_credentials()
                    .allowed_methods(vec!["GET", "POST", "PATCH", "DELETE"])
                    .allowed_headers(vec![
                        header::CONTENT_TYPE,
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                    ])
            )
            .route("/", web::get().to(|| async { "Hello, world!" }))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
