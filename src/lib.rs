//! # rou3
//!
//! rou3 is a lightweight and performant HTTP routing library for Rust.
//! It focuses on fast route matching, including support for static paths,
//! parameters (e.g., `/:id`), and wildcards (e.g., `/*` or `/**`).
//!
//! The primary goal is to provide a flexible routing solution that is easy to
//! integrate into various web frameworks or custom server implementations.
//! It uses a tree-based routing algorithm for efficient lookups.
//!
//! ## Features
//!
//! - Static, parameterized, and wildcard route matching.
//! - Method-based routing (GET, POST, etc.), including an "any" method.
//! - Route removal.
//! - Parameter extraction.
//! - Thread-safe router using `parking_lot::RwLock`.
//! - Efficient data structures (`AHashMap`, `IndexMap`) for performance.
//! - Structured error handling with `thiserror`.
//!
//! ## Example
//!
//! ```rust
//! use rou3::{Router, add_route, find_route, MatchedRoute, RouterError};
//!
//! // Create a new router instance.
//! let router = Router::new();
//!
//! // Add some routes.
//! add_route(&router, "GET", "/home", "Welcome Home!").expect("Failed to add /home");
//! add_route(&router, "GET", "/users/:id", "User Profile").expect("Failed to add /users/:id");
//! add_route(&router, "POST", "/users", "Create User").expect("Failed to add /users POST");
//! add_route(&router, "GET", "/files/**:filepath", "Serve File").expect("Failed to add /files/**:filepath");
//!
//! // Find a route.
//! // The `capture` argument (boolean) determines if parameters should be extracted.
//! match find_route(&router, "GET", "/users/123", true) {
//!     Ok(matched_route) => {
//!         assert_eq!(matched_route.data, "User Profile");
//!         if let Some(params) = matched_route.params {
//!             assert_eq!(params.get("id").unwrap(), "123");
//!         }
//!     }
//!     Err(e) => panic!("Expected to find route, but got error: {}", e),
//! }
//!
//! match find_route(&router, "GET", "/files/path/to/my/file.txt", true) {
//!     Ok(matched_route) => {
//!         assert_eq!(matched_route.data, "Serve File");
//!         if let Some(params) = matched_route.params {
//!             assert_eq!(params.get("filepath").unwrap(), "path/to/my/file.txt");
//!         }
//!     }
//!     Err(e) => panic!("Expected to find file route, but got error: {}", e),
//! }
//!
//! // If a route is not found, `find_route` returns `Err(RouterError::RouteNotFound)`.
//! match find_route(&router, "GET", "/nonexistent", false) {
//!     Err(RouterError::RouteNotFound { method, path }) => {
//!         assert_eq!(method, "GET");
//!         assert_eq!(path, "/nonexistent"); // Path normalization happens, so original might differ slightly
//!     }
//!     _ => panic!("Expected RouteNotFound error"),
//! }
//! ```

pub mod context;
pub mod error;
pub mod operations;
pub mod types;

pub use context::Router;
pub use error::RouterError;
pub use operations::add_route;
pub use operations::find_all_routes;
pub use operations::find_route;
pub use operations::remove_route;
pub use types::MatchedRoute;
