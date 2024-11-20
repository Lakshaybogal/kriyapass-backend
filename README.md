To support multiple origins with the `actix-web` framework in Rust and its `Cors` middleware, you can't directly pass a list of origins to the `allowed_origin` method. Instead, you can use a custom validation function or a crate like [`actix-cors`](https://docs.rs/actix-cors/) to define a more flexible policy.

Here’s how you can handle multiple origins:

### 1. **Using `allowed_origin_fn`**
The `allowed_origin_fn` method allows you to provide a custom function to validate origins dynamically. Here’s an example:

```rust
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
```

### 2. **Using Wildcard with Constraints**
If your use case allows all subdomains of a domain or patterns, you might use a wildcard like `*.example.com`. However, this can be insecure if overused.

```rust
Cors::default()
    .allowed_origin("http://*.example.com")
    .supports_credentials();
```

### 3. **Manually Matching Origins**
For a more manual approach, you can inspect the `Origin` header in a middleware or request handler and conditionally respond based on the origin.

### Security Note
When using `supports_credentials`, avoid using a wildcard (`*`) for origins, as it conflicts with `Access-Control-Allow-Credentials`. Always specify origins explicitly or validate dynamically.
