// Re-export the macro for convenience
pub use ferrox_macros::http_method;

// Server-side runtime imports
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{delete, get, patch, post, put},
    Router,
};
use inventory;
use lazy_static::lazy_static;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceBuilder;

#[derive(Serialize, Clone)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
}

// Route registration via inventory with generic function interface
// Routes are automatically registered by macros - no naming scheme needed
pub struct RouteRegistration {
    pub method: &'static str,
    pub path: &'static str,
    pub handler_fn: fn() -> RouteHandler,
}

inventory::collect!(RouteRegistration);

// Global route registry
lazy_static! {
    pub static ref GLOBAL_ROUTE_REGISTRY: Mutex<HashMap<(String, String), RouteHandler>> = Mutex::new(HashMap::new());
}

// Generic handler interface - functions take JSON params and return JSON response
// The framework converts JSON responses to HTTP responses automatically
pub type RouteHandler = Arc<dyn Fn(serde_json::Value, serde_json::Value, serde_json::Value) -> serde_json::Value + Send + Sync>;

// Server struct - routes are automatically registered by http_method! macros
pub struct Server;

impl Server {
    pub fn new() -> Self {
        Self
    }

    pub async fn start(self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 REST API Server with Generic Route Handler Interface");
        println!("========================================================");
        println!("🔍 Routes automatically discovered from #[http_method] annotations");
        println!("📦 Generic interface: All functions wrapped in boxed closures");
        println!("📍 Routes stored as (method, path) -> closure in HashMap at startup!");
        println!();

        // Populate global registry from inventory-collected routes
        let mut global_registry = GLOBAL_ROUTE_REGISTRY.lock().unwrap();
        for registration in inventory::iter::<RouteRegistration> {
            global_registry.insert(
                (registration.method.to_string(), registration.path.to_string()),
                (registration.handler_fn)(),
            );

            let emoji = match registration.method {
                "GET" => "📋",
                "POST" => "➕",
                "PUT" => "✏️",
                "PATCH" => "📧",
                "DELETE" => "🗑️",
                "HEAD" => "❓",
                "OPTIONS" => "ℹ️",
                _ => "🔍",
            };
            println!("{} {} {} - Auto-registered from annotation", emoji, registration.method, registration.path);
        }

        // Build router - routes are looked up from HashMap at runtime
        let mut router = Router::new();

        // Handlers that lookup from the global registry

        // Dynamically register routes based on inventory-collected registrations
        for registration in inventory::iter::<RouteRegistration> {
            let method = registration.method;
            let path = registration.path;
            let handler = (registration.handler_fn)(); // Get the Arc<RouteHandler>

            // Create a generic handler that extracts path, query, and body parameters
            let generic_handler = move |
                Path(path_params): Path<HashMap<String, String>>,
                Query(query_params): Query<HashMap<String, String>>,
                body: Option<Json<serde_json::Value>>
            | async move {
                // Convert path parameters to JSON
                let mut path_json = serde_json::Map::new();
                for (key, value) in path_params {
                    path_json.insert(key, serde_json::Value::String(value));
                }
                let path_identifiers = serde_json::Value::Object(path_json);

                // Convert query parameters to JSON
                let mut query_json = serde_json::Map::new();
                for (key, value) in query_params {
                    query_json.insert(key, serde_json::Value::String(value));
                }
                let query_arguments = serde_json::Value::Object(query_json);

                // Body parameters
                let body_value = body.map(|Json(v)| v).unwrap_or(serde_json::Value::Null);

                // Call the handler with three separate arguments and convert JSON to HTTP response
                let json_result = handler(path_identifiers, query_arguments, body_value);
                axum::Json(json_result).into_response()
            };

            // Register the route based on HTTP method
            match method {
                "GET" => router = router.route(path, get(generic_handler)),
                "POST" => router = router.route(path, post(generic_handler)),
                "PUT" => router = router.route(path, put(generic_handler)),
                "PATCH" => router = router.route(path, patch(generic_handler)),
                "DELETE" => router = router.route(path, delete(generic_handler)),
                "HEAD" => router = router.route(path, axum::routing::head(generic_handler)),
                "OPTIONS" => router = router.route(path, axum::routing::options(generic_handler)),
                _ => { /* Log unsupported method */ }
            }
        }

        let app = router.fallback(not_found_handler).layer(ServiceBuilder::new());

        println!();

        let socket_addr: std::net::SocketAddr = addr.parse()?;
        println!("🌐 Server running at http://{}", socket_addr);
        println!("🔎 Each incoming request extracts (method, path) and looks up function from HashMap!");
        println!();

        let listener = tokio::net::TcpListener::bind(socket_addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn not_found_handler(uri: axum::http::Uri) -> (StatusCode, Json<ApiResponse<String>>) {
    (
        StatusCode::NOT_FOUND,
        Json(ApiResponse {
            success: false,
            data: None,
            message: format!("Route {} not found", uri.path()),
        }),
    )
}

// Server components are now available at the library root
