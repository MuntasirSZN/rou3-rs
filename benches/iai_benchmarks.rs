use iai_callgrind::{
    EventKind, FlamegraphConfig, LibraryBenchmarkConfig, RegressionConfig, Tool, ValgrindTool,
    library_benchmark, library_benchmark_group, main as iai_main,
};
use rou3::{Router, add_route, find_all_routes, find_route};
use std::hint::black_box;

fn setup_large_router_for_lookups() -> Router<usize> {
    let router = Router::new();
    let size = 1_000;
    for i in 0..size {
        add_route(&router, "GET", &format!("/static/item/{}", i), i).unwrap();
        add_route(&router, "GET", &format!("/param/user{}/profile", i), i).unwrap();
        add_route(
            &router,
            "GET",
            &format!("/wildcard/files{}/docs/**:path", i),
            i,
        )
        .unwrap();
    }
    router
}

fn setup_api_style_router() -> Router<&'static str> {
    let router = Router::new();
    add_route(&router, "GET", "/api/v1/users", "list_users").unwrap();
    add_route(&router, "POST", "/api/v1/users", "create_user").unwrap();
    add_route(&router, "GET", "/api/v1/users/:userId", "get_user").unwrap();
    add_route(&router, "PUT", "/api/v1/users/:userId", "update_user").unwrap();
    add_route(&router, "DELETE", "/api/v1/users/:userId", "delete_user").unwrap();
    add_route(
        &router,
        "GET",
        "/api/v1/users/:userId/posts/:postId",
        "get_user_post",
    )
    .unwrap();
    add_route(&router, "GET", "/api/v1/files/**:filePath", "serve_file").unwrap();
    add_route(&router, "GET", "/api/v1/search/:query?", "search_optional").unwrap();
    router
}

fn setup_router_for_find_all() -> Router<usize> {
    let router = Router::new();
    let n_patterns = 200;
    for i in 0..n_patterns {
        add_route(&router, "GET", &format!("/api/collection{}/:id", i), i).unwrap();
        add_route(
            &router,
            "GET",
            &format!("/api/collection{}/specific/action", i),
            i + n_patterns,
        )
        .unwrap();
        if i % 10 == 0 {
            add_route(
                &router,
                "GET",
                &format!("/api/collection{}/archive/**:path", i),
                i + 2 * n_patterns,
            )
            .unwrap();
        }
    }
    let generic_param_id = n_patterns * 3;
    let generic_wildcard_id = n_patterns * 3 + 1;
    add_route(&router, "GET", "/api/:resource/:id", generic_param_id).unwrap();
    add_route(&router, "GET", "/api/**:wildcard_all", generic_wildcard_id).unwrap();
    router
}

// --- Benchmark Functions ---

#[library_benchmark]
pub fn bench_lookup_static_last_iai_fn() {
    let router = setup_large_router_for_lookups();
    black_box(find_route(&router, "GET", "/static/item/999", false).unwrap());
}

#[library_benchmark]
pub fn bench_lookup_param_last_iai_fn() {
    let router = setup_large_router_for_lookups();
    black_box(find_route(&router, "GET", "/param/user999/profile", true).unwrap());
}

#[library_benchmark]
pub fn bench_lookup_wildcard_last_iai_fn() {
    let router = setup_large_router_for_lookups();
    black_box(find_route(&router, "GET", "/wildcard/files999/docs/a/b/c.txt", true).unwrap());
}

#[library_benchmark]
pub fn bench_api_get_user_post_iai_fn() {
    let router = setup_api_style_router();
    black_box(
        find_route(
            &router,
            "GET",
            "/api/v1/users/user123abc/posts/post789xyz",
            true,
        )
        .unwrap(),
    );
}

#[library_benchmark]
pub fn bench_api_serve_file_wildcard_iai_fn() {
    let router = setup_api_style_router();
    black_box(find_route(&router, "GET", "/api/v1/files/docs/report.pdf", true).unwrap());
}

#[library_benchmark]
pub fn bench_api_search_optional_absent_iai_fn() {
    let router = setup_api_style_router();
    black_box(find_route(&router, "GET", "/api/v1/search/", true).unwrap());
}

#[library_benchmark]
pub fn bench_find_all_match_path_medium_router_iai_fn() {
    let router = setup_router_for_find_all();
    let path_to_match = "/api/collection50/123";
    black_box(find_all_routes(&router, "GET", path_to_match, true));
}

#[library_benchmark]
pub fn bench_find_all_match_wildcard_path_medium_router_iai_fn() {
    let router = setup_router_for_find_all();
    let path_to_match_wildcard = "/api/collection50/archive/some/long/path";
    black_box(find_all_routes(
        &router,
        "GET",
        path_to_match_wildcard,
        true,
    ));
}

#[library_benchmark]
pub fn bench_add_many_routes_iai_fn() {
    let router = Router::new();
    let num_routes_to_add = 500;
    for i in 0..num_routes_to_add {
        add_route(&router, "GET", &format!("/static/item/{}", i), i).unwrap();
        if i % 10 == 0 {
            add_route(&router, "GET", &format!("/param/user{}/:id", i), i).unwrap();
        }
    }
    black_box(router);
}

library_benchmark_group!(
    name = all_iai_benchmarks;
    benchmarks =
        bench_lookup_static_last_iai_fn,
        bench_lookup_param_last_iai_fn,
        bench_lookup_wildcard_last_iai_fn,
        bench_api_get_user_post_iai_fn,
        bench_api_serve_file_wildcard_iai_fn,
        bench_api_search_optional_absent_iai_fn,
        bench_find_all_match_path_medium_router_iai_fn,
        bench_find_all_match_wildcard_path_medium_router_iai_fn,
        bench_add_many_routes_iai_fn
);

iai_main!(
    config = LibraryBenchmarkConfig::default()
    .tool(Tool::new(ValgrindTool::DHAT))
    .tool(Tool::new(ValgrindTool::Massif))
    .tool(Tool::new(ValgrindTool::BBV))
    .tool(Tool::new(ValgrindTool::Memcheck))
    .tool(Tool::new(ValgrindTool::Helgrind))
    .tool(Tool::new(ValgrindTool::DRD))
    .flamegraph(FlamegraphConfig::default())
    .regression(
        RegressionConfig::default()
        .limits([(EventKind::Ir, 5.0)])
    );
    library_benchmark_groups = all_iai_benchmarks
);
