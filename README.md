# rou3-rs

**A flexible and fast HTTP router for Rust, inspired by [rou3-js](https://github.com/h3js/rou3).**

[![Crates.io](https://img.shields.io/crates/v/rou3.svg)](https://crates.io/crates/rou3)
[![Docs.rs](https://docs.rs/rou3/badge.svg)](https://docs.rs/rou3)
[![Build Status](https://img.shields.io/github/actions/workflow/status/MuntasirSZN/rou3-rs/test.yml?branch=main)](https://github.com/MuntasirSZN/rou3-rs/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

`rou3-rs` provides a high-performance routing solution for Rust applications, drawing inspiration from the design principles of `rou3-js`. It uses a trie-based structure with efficient hash map lookups for dynamic path segments and an optimized map for purely static routes.

## Features

- **Versatile Route Matching:**
  - **Static routes:** e.g., `/home`, `/about/contact`
  - **Parameterized routes:** e.g., `/users/:id`, `/posts/:category/:slug`
  - **Wildcard routes:**
    - Single segment wildcard: `/files/*` (captures one segment)
    - Multi-segment (catch-all) wildcard: `/assets/**:filepath` (must be at the end)
  - **Optional parameters:** e.g., `/search/:query?` (matches `/search/` and `/search/term`)
- **Method-based Routing:** Supports standard HTTP methods (GET, POST, PUT, DELETE, etc.) and an "ANY" method (empty string `""`) to match any HTTP method.
- **Efficient:**
  - Trie structure with `AHashMap` for fast dynamic dispatch.
  - Dedicated `static_map` for instant lookups of purely static paths.
- **Dynamic Modification:** Add and remove routes at runtime.
- **`findAllRoutes`:** Retrieve all routes that match a given path, useful for middleware or complex dispatch logic.
- **Thread-Safe:** Core router operations are thread-safe using `parking_lot::RwLock`.
- **Clear Error Handling:** Provides a `RouterError` enum for robust error management.

## Installation

Add `rou3-rs` to your `Cargo.toml`:

```toml
[dependencies]
rou3 = "0.1.0"
```

## Usage

Here's a quick overview of how to use `rou3-rs`:

```rust
use rou3::{Router, add_route, find_route, findAllRoutes, MatchedRoute, RouterError};
use std::collections::HashMap; // For easily checking params

fn main() -> Result<(), RouterError> {
    // Create a new router. Let's say it stores string slices as data.
    let router: Router<&'static str> = Router::new();

    // 1. Add a static route
    add_route(&router, "GET", "/home", "Welcome Home!")?;

    // 2. Add a parameterized route
    add_route(&router, "GET", "/users/:userId", "User Profile")?;
    add_route(&router, "POST", "/users/:userId/message", "Send Message to User")?;

    // 3. Add a wildcard route
    add_route(&router, "GET", "/files/*", "Single File Wildcard")?; // Matches /files/report.pdf
    add_route(&router, "GET", "/assets/**:filepath", "Serve Asset")?; // Matches /assets/css/style.css

    // 4. Add a route with an optional parameter
    add_route(&router, "GET", "/search/:query?", "Search Page")?;

    // 5. Add a route for ANY HTTP method
    add_route(&router, "", "/any/path", "Matches any method")?;

    // --- Finding routes ---

    // Find the static route
    let home_route = find_route(&router, "GET", "/home", false)?; // capture_params = false
    assert_eq!(home_route.data, "Welcome Home!");
    assert!(home_route.params.is_none());

    // Find a parameterized route and capture parameters
    let user_route = find_route(&router, "GET", "/users/123", true)?; // capture_params = true
    assert_eq!(user_route.data, "User Profile");
    let expected_params = HashMap::from([("userId".to_string(), "123".to_string())]);
    assert_eq!(user_route.params.map(|p| p.into_iter().collect::<HashMap<_,_>>()), Some(expected_params));

    // Find a route matching the optional parameter (with value)
    let search_with_query = find_route(&router, "GET", "/search/rust-router", true)?;
    assert_eq!(search_with_query.data, "Search Page");
    assert_eq!(
        search_with_query.params.unwrap().get("query"),
        Some(&"rust-router".to_string())
    );

    // Find a route matching the optional parameter (without value)
    let search_without_query = find_route(&router, "GET", "/search/", true)?; // or /search
    assert_eq!(search_without_query.data, "Search Page");
    assert!(search_without_query.params.as_ref().map_or(true, |p| p.get("query").is_none() && p.is_empty()));


    // Find a route using the ANY method
    let any_method_route_get = find_route(&router, "GET", "/any/path", false)?;
    assert_eq!(any_method_route_get.data, "Matches any method");
    let any_method_route_post = find_route(&router, "POST", "/any/path", false)?;
    assert_eq!(any_method_route_post.data, "Matches any method");


    // --- Finding all matching routes ---
    add_route(&router, "GET", "/config", "Config Base")?;
    add_route(&router, "GET", "/config/:key", "Config Key Specific")?;
    add_route(&router, "GET", "/config/**:path", "Config Wildcard")?;

    let all_matches = findAllRoutes(&router, "GET", "/config/timeout", true);
    println!("Found {} matches for /config/timeout:", all_matches.len());
    for m in all_matches {
        println!("  - Data: {}, Params: {:?}", m.data, m.params);
    }
    // Expected output would show 3 matches: Config Key Specific, Config Wildcard, and potentially a root wildcard if one was added.

    Ok(())
}
```

## Benchmarks

`rou3-rs` is designed with performance in mind. It includes benchmarks comparing it against other popular Rust routers. For detailed results, please see the benchmark files in the `benches` directory and run `cargo bench`.

## Contributing

Contributions are welcome! Please feel free to submit issues, fork the repository, and create pull requests.

Ways to contribute:

- Reporting bugs
- Suggesting new features or enhancements
- Improving documentation
- Adding more test cases
- Optimizing performance

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgements

- The design and API are heavily inspired by [h3js/rou3](https://github.com/h3js/rou3).

