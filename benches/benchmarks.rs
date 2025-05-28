use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rou3::{Router, add_route, find_all_routes, find_route};
use std::hint::black_box;

fn bench_build_router_with_various_routes(c: &mut Criterion) {
    let mut group = c.benchmark_group("build_router");
    for &size in &[100usize, 1_000, 5_000] {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &n| {
            b.iter(|| {
                let r = Router::new();
                for i in 0..n {
                    add_route(&r, "GET", &format!("/static/{}", i), i).unwrap();
                    if i % 10 == 0 {
                        add_route(&r, "GET", &format!("/param/{}/:id", i), i).unwrap();
                    }
                    if i % 50 == 0 {
                        add_route(&r, "GET", &format!("/wildcard/{}/item/**:rest", i), i).unwrap();
                    }
                }
                black_box(r);
            });
        });
    }
    group.finish();
}

fn bench_lookup_routes(c: &mut Criterion) {
    let mut group = c.benchmark_group("lookup_routes");
    let size = 5_000;
    let router = Router::new();
    for i in 0..size {
        add_route(&router, "GET", &format!("/static/{}", i), i).unwrap();
        add_route(&router, "GET", &format!("/user/:id{}", i), i).unwrap();
        add_route(&router, "GET", &format!("/files/{}/docs/**:path", i), i).unwrap();
    }

    group.bench_function("lookup_static_last", |b| {
        b.iter(|| {
            black_box(find_route(&router, "GET", "/static/4999", false).unwrap());
        })
    });

    group.bench_function("lookup_param_last", |b| {
        b.iter(|| {
            // The path needs to match one of the :id{} patterns
            black_box(find_route(&router, "GET", "/user/somevalue4999", true).unwrap());
        })
    });

    group.bench_function("lookup_wildcard_last", |b| {
        b.iter(|| {
            black_box(find_route(&router, "GET", "/files/4999/docs/a/b/c.txt", true).unwrap());
        })
    });
    group.finish();
}

fn bench_find_all_matching_routes(c: &mut Criterion) {
    let mut group = c.benchmark_group("find_all_matching_routes");
    let router = Router::new();
    let n_patterns = 1_000;
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
    // Change string data to usize to match the router's inferred type T=usize
    let generic_param_id = n_patterns * 3;
    let generic_wildcard_id = n_patterns * 3 + 1;
    add_route(&router, "GET", "/api/:resource/:id", generic_param_id).unwrap();
    add_route(&router, "GET", "/api/**:wildcard_all", generic_wildcard_id).unwrap();

    let path_to_match = "/api/collection50/123";
    group.bench_function("match_path_medium_router", |b| {
        b.iter(|| {
            black_box(find_all_routes(&router, "GET", path_to_match, true));
        })
    });

    let path_to_match_wildcard = "/api/collection50/archive/some/long/path";
    group.bench_function("match_wildcard_path_medium_router", |b| {
        b.iter(|| {
            black_box(find_all_routes(
                &router,
                "GET",
                path_to_match_wildcard,
                true,
            ));
        })
    });
    group.finish();
}

fn bench_api_style_lookups(c: &mut Criterion) {
    let mut group = c.benchmark_group("api_style_lookups");
    // For this benchmark, let's use &'static str for T
    let router = Router::<&'static str>::new();

    add_route(&router, "GET", "/api/v1/users", "list_users").unwrap();
    add_route(&router, "POST", "/api/v1/users", "create_user").unwrap();
    add_route(&router, "GET", "/api/v1/users/:userId", "get_user").unwrap();
    add_route(&router, "PUT", "/api/v1/users/:userId", "update_user").unwrap();
    add_route(&router, "DELETE", "/api/v1/users/:userId", "delete_user").unwrap();
    add_route(
        &router,
        "GET",
        "/api/v1/users/:userId/posts",
        "list_user_posts",
    )
    .unwrap();
    add_route(
        &router,
        "POST",
        "/api/v1/users/:userId/posts",
        "create_user_post",
    )
    .unwrap();
    add_route(
        &router,
        "GET",
        "/api/v1/users/:userId/posts/:postId",
        "get_user_post",
    )
    .unwrap();
    add_route(
        &router,
        "PUT",
        "/api/v1/users/:userId/posts/:postId",
        "update_user_post",
    )
    .unwrap();
    add_route(
        &router,
        "DELETE",
        "/api/v1/users/:userId/posts/:postId",
        "delete_user_post",
    )
    .unwrap();
    add_route(&router, "GET", "/api/v1/files/**:filePath", "serve_file").unwrap();
    add_route(&router, "GET", "/api/v1/search/:query?", "search_optional").unwrap();

    group.bench_function("get_user_specific", |b| {
        b.iter(|| black_box(find_route(&router, "GET", "/api/v1/users/user123abc", true).unwrap()))
    });
    group.bench_function("get_post_specific", |b| {
        b.iter(|| {
            black_box(
                find_route(
                    &router,
                    "GET",
                    "/api/v1/users/user123abc/posts/post789xyz",
                    true,
                )
                .unwrap(),
            )
        })
    });
    group.bench_function("serve_file_wildcard", |b| {
        b.iter(|| {
            black_box(find_route(&router, "GET", "/api/v1/files/docs/report.pdf", true).unwrap())
        })
    });
    group.bench_function("search_with_optional_present", |b| {
        b.iter(|| black_box(find_route(&router, "GET", "/api/v1/search/keyword", true).unwrap()))
    });
    group.bench_function("search_with_optional_absent", |b| {
        b.iter(|| black_box(find_route(&router, "GET", "/api/v1/search/", true).unwrap()))
    });

    let path_to_match_user_post = "/api/v1/users/user123abc/posts/post789xyz";
    group.bench_function("find_all_for_user_post", |b| {
        b.iter(|| {
            black_box(find_all_routes(
                &router,
                "GET",
                path_to_match_user_post,
                true,
            ))
        })
    });
    group.finish();
}

criterion_group!(
    benches,
    bench_build_router_with_various_routes,
    bench_lookup_routes,
    bench_find_all_matching_routes,
    bench_api_style_lookups
);
criterion_main!(benches);
