use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use rou3::{Router as Rou3Router, add_route, find_route}; // Renamed to avoid conflict

macro_rules! routes {
    (literal) => {{
        routes!(finish => "match_p1", "match_p2", "match_p3", "match_p4")
    }};
    (colon) => {{
        routes!(finish => ":p1", ":p2", ":p3", ":p4")
    }};
    (rou3_colon_add) => {{
        routes!(finish => ":p1", ":p2", ":p3", ":p4")
    }};
    (brackets) => {{
        routes!(finish => "{p1}", "{p2}", "{p3}", "{p4}")
    }};
    (regex) => {{
        routes!(finish => "(.*)", "(.*)", "(.*)", "(.*)")
    }};
    (finish => $p1:expr, $p2:expr, $p3:expr, $p4:expr) => {{
        [
            // Static paths
            "/authorizations",
            "/events",
            "/feeds",
            "/notifications",
            "/user/starred",
            "/user/subscriptions",
            "/gists",
            "/issues",
            "/user/issues",
            "/emojis",
            "/gitignore/templates",
            "/meta",
            "/rate_limit",
            "/user/orgs",
            "/user/teams",
            "/repositories",
            "/search/repositories",
            "/search/code",
            "/search/issues",
            "/search/users",
            "/user",
            "/users",
            "/user/emails",
            "/user/followers",
            "/user/following",
            "/user/keys",

            // Parameterized paths
            concat!("/authorizations/", $p1),
            concat!("/notifications/threads/", $p1),
            concat!("/gists/", $p1),
            concat!("/gitignore/templates/", $p1),
            concat!("/orgs/", $p1),
            concat!("/teams/", $p1),
            concat!("/legacy/repos/search/", $p1),
            concat!("/legacy/user/search/", $p1),
            concat!("/legacy/user/email/", $p1),
            concat!("/users/", $p1),
            concat!("/user/keys/", $p1),

            concat!("/user/starred/", $p1, "/", $p2),
            concat!("/user/subscriptions/", $p1, "/", $p2),
            concat!("/orgs/", $p1, "/events"),
            concat!("/orgs/", $p1, "/issues"),
            concat!("/orgs/", $p1, "/members"),
            concat!("/orgs/", $p1, "/members/", $p2),
            concat!("/orgs/", $p1, "/public_members"),
            concat!("/orgs/", $p1, "/public_members/", $p2),
            concat!("/orgs/", $p1, "/repos"),
            concat!("/teams/", $p1, "/members"),
            concat!("/users/", $p1, "/received_events"),
            concat!("/users/", $p1, "/events"),
            concat!("/users/", $p1, "/gists"),
            concat!("/users/", $p1, "/orgs"),
            concat!("/users/", $p1, "/repos"),
            concat!("/users/", $p1, "/starred"),
            concat!("/users/", $p1, "/subscriptions"),
            concat!("/users/", $p1, "/followers"),
            concat!("/users/", $p1, "/following"),
            concat!("/users/", $p1, "/keys"),
            concat!("/users/", $p1, "/following/", $p2),

            concat!("/applications/", $p1, "/tokens/", $p2),
            concat!("/repos/", $p1, "/", $p2, "/events"),
            concat!("/networks/", $p1, "/", $p2, "/events"),
            concat!("/users/", $p1, "/events/orgs/", $p2),
            concat!("/repos/", $p1, "/", $p2, "/notifications"),
            concat!("/repos/", $p1, "/", $p2, "/stargazers"),
            concat!("/repos/", $p1, "/", $p2, "/subscribers"),
            concat!("/repos/", $p1, "/", $p2, "/subscription"),
            concat!("/repos/", $p1, "/", $p2, "/git/blobs/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/git/commits/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/git/refs"),
            concat!("/repos/", $p1, "/", $p2, "/git/tags/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/git/trees/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/issues"),
            concat!("/repos/", $p1, "/", $p2, "/issues/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/assignees"),
            concat!("/repos/", $p1, "/", $p2, "/assignees/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/labels"),
            concat!("/repos/", $p1, "/", $p2, "/labels/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/milestones"),
            concat!("/repos/", $p1, "/", $p2, "/milestones/", $p3),
            concat!("/teams/", $p1, "/repos/", $p2, "/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/pulls"),
            concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/readme"),
            concat!("/repos/", $p1, "/", $p2, "/tags"),
            concat!("/repos/", $p1, "/", $p2, "/branches"),
            concat!("/repos/", $p1, "/", $p2, "/branches/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/collaborators"),
            concat!("/repos/", $p1, "/", $p2, "/collaborators/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/comments"),
            concat!("/repos/", $p1, "/", $p2, "/commits"),
            concat!("/repos/", $p1, "/", $p2, "/commits/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/keys"),
            concat!("/repos/", $p1, "/", $p2, "/keys/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/downloads"),
            concat!("/repos/", $p1, "/", $p2, "/downloads/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/forks"),
            concat!("/repos/", $p1, "/", $p2, "/hooks"),
            concat!("/repos/", $p1, "/", $p2, "/hooks/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/releases"),
            concat!("/repos/", $p1, "/", $p2, "/releases/", $p3),
            concat!("/repos/", $p1, "/", $p2, "/stats/contributors"),
            concat!("/repos/", $p1, "/", $p2, "/stats/commit_activity"),
            concat!("/repos/", $p1, "/", $p2, "/stats/code_frequency"),
            concat!("/repos/", $p1, "/", $p2, "/stats/participation"),
            concat!("/repos/", $p1, "/", $p2, "/stats/punch_card"),
            concat!("/repos/", $p1, "/", $p2, "/statuses/", $p3),

            concat!("/repos/", $p1, "/", $p2, "/issues/", $p3, "/comments"),
            concat!("/repos/", $p1, "/", $p2, "/issues/", $p3, "/events"),
            concat!("/repos/", $p1, "/", $p2, "/issues/", $p3, "/labels"),
            concat!("/repos/", $p1, "/", $p2, "/milestones/", $p3, "/labels"),
            concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3, "/commits"),
            concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3, "/files"),
            concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3, "/merge"),
            concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3, "/comments"),
            concat!("/repos/", $p1, "/", $p2, "/commits/", $p3, "/comments"),
            concat!("/repos/", $p1, "/", $p2, "/releases/", $p3, "/assets"),
            concat!("/legacy/issues/search/", $p1, "/", $p2, "/", $p3, "/", $p4),
        ]
    }};
}

fn compare_routers(c: &mut Criterion) {
    let mut group = c.benchmark_group("Compare Routers");

    let lookup_paths = routes!(literal).to_vec();

    // --- rou3 ---
    let rou3_router = Rou3Router::new();
    for route_pattern in routes!(rou3_colon_add) {
        add_route(&rou3_router, "GET", route_pattern, true).expect("rou3 add failed");
    }
    group.bench_function("rou3", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let result =
                    black_box(find_route(&rou3_router, "GET", path_to_lookup, true).unwrap());
                assert!(result.data);
            }
        });
    });

    // --- matchit ---
    let mut matchit_router = matchit::Router::new();
    for route_pattern in routes!(brackets) {
        matchit_router.insert(route_pattern, true).unwrap();
    }
    group.bench_function("matchit", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let result = black_box(matchit_router.at(path_to_lookup).unwrap());
                assert!(*result.value);
            }
        });
    });

    // --- wayfind ---
    let mut wayfind_router = wayfind::Router::new();
    for route_pattern in routes!(brackets) {
        wayfind_router.insert(route_pattern, true).unwrap();
    }
    group.bench_function("wayfind", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let result = black_box(wayfind_router.search(path_to_lookup).unwrap());
                assert!(*result.data);
            }
        });
    });

    // --- path-tree ---
    let mut path_tree_router = path_tree::PathTree::new();
    for route_pattern in routes!(colon) {
        let _ = path_tree_router.insert(route_pattern, true);
    }
    group.bench_function("path-tree", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let result = black_box(path_tree_router.find(path_to_lookup).unwrap());
                assert!(*result.0);
            }
        });
    });

    // --- gonzales ---
    // gonzales takes a Vec or slice of path strings.
    let gonzales_route_patterns = routes!(brackets).to_vec();
    let gonzales_router = gonzales::RouterBuilder::new().build(&gonzales_route_patterns);
    group.bench_function("gonzales", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let result = black_box(gonzales_router.route(path_to_lookup).unwrap());
                // The original assertion checked if the index was valid against the input `registered` list.
                // Here, `gonzales_route_patterns` is that list.
                assert!(gonzales_route_patterns.get(result.get_index()).is_some());
            }
        });
    });

    // --- actix-router ---
    let mut actix_builder = actix_router::Router::<bool>::build();
    for route_pattern in routes!(brackets) {
        actix_builder.path(route_pattern, true);
    }
    let actix_router_finished = actix_builder.finish();
    group.bench_function("actix", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let mut path_obj = actix_router::Path::new(*path_to_lookup);
                let result = black_box(actix_router_finished.recognize(&mut path_obj).unwrap());
                assert!(*result.0);
            }
        });
    });

    // --- regex ---
    let regex_patterns_for_set: Vec<String> = routes!(colon)
        .iter()
        .map(|s| {
            // Using colon for route structure
            let route_like_pattern = s
                .replace(":p1", "([^/]+)")
                .replace(":p2", "([^/]+)")
                .replace(":p3", "([^/]+)")
                .replace(":p4", "([^/]+)");
            format!("^{}$", route_like_pattern)
        })
        .collect::<Vec<String>>();
    let regex_set = regex::RegexSet::new(regex_patterns_for_set).unwrap();
    group.bench_function("regex", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let result = black_box(regex_set.matches(path_to_lookup));
                assert!(result.matched_any());
            }
        });
    });

    // --- route-recognizer ---
    let mut route_recognizer_router = route_recognizer::Router::new();
    for route_pattern in routes!(colon) {
        route_recognizer_router.add(route_pattern, true);
    }
    group.bench_function("route-recognizer", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let result = black_box(route_recognizer_router.recognize(path_to_lookup).unwrap());
                assert!(**result.handler());
            }
        });
    });

    // --- routefinder ---
    let mut routefinder_router = routefinder::Router::new();
    for route_pattern in routes!(colon) {
        routefinder_router.add(route_pattern, true).unwrap();
    }
    group.bench_function("routefinder", |b| {
        b.iter(|| {
            for path_to_lookup in black_box(&lookup_paths) {
                let result = black_box(routefinder_router.best_match(path_to_lookup).unwrap());
                assert!(*result.handler());
            }
        });
    });

    group.finish();
}

criterion_group!(benches, compare_routers);
criterion_main!(benches);
