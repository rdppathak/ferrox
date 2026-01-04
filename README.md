# ferrox

A macro-driven Rust framework for building REST APIs declaratively.

## Overview

Ferrox provides a declarative approach to building REST APIs in Rust. Using the `#[http_method]` attribute macro, you can annotate functions to automatically register them as HTTP endpoints. Routes are discovered at compile-time using the `inventory` crate and stored in a global registry for runtime lookup.

## Implementation

Ferrox leverages:
- **Procedural macros** for compile-time route registration
- **Inventory pattern** for automatic route discovery
- **Axum** as the underlying HTTP server framework
- **Generic JSON interface** where all handlers receive path parameters, query parameters, and request body as JSON values

Routes are stored in a global HashMap with `(method, path)` keys and executed dynamically at runtime.

## Usage

Add ferrox to your `Cargo.toml`:

```toml
[dependencies]
ferrox = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### Basic Example

```rust
use ferrox::{http_method, Server};
use ferrox::ApiResponse;
use serde_json::{json, Value};

#[http_method(GET, "/users")]
fn get_users(path: Value, query: Value, body: Value) -> Value {
    // Return JSON response - framework automatically converts to HTTP response
    json!(ApiResponse {
        success: true,
        data: Some(json!({"users": ["alice", "bob", "charlie"]})),
        message: "Users retrieved successfully".to_string()
    })
}

#[http_method(POST, "/users")]
fn create_user(path: Value, query: Value, body: Value) -> Value {
    // Access request body
    let user_data = body.as_object().unwrap();
    let name = user_data.get("name").unwrap().as_str().unwrap();

    json!(ApiResponse {
        success: true,
        data: Some(json!({"id": 123, "name": name})),
        message: "User created successfully".to_string()
    })
}

#[http_method(GET, "/users/:id")]
fn get_user(path: Value, query: Value, body: Value) -> Value {
    // Access path parameters
    let user_id = path.get("id").unwrap().as_str().unwrap();

    json!(ApiResponse {
        success: true,
        data: Some(json!({"id": user_id, "name": "alice"})),
        message: format!("User {} retrieved", user_id)
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = Server::new();
    server.start("127.0.0.1:3000").await?;
    Ok(())
}
```

Routes are automatically registered when the application starts. The framework handles JSON serialization/deserialization and converts responses to appropriate HTTP status codes.
