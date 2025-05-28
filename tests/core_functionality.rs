use rou3::{Router, RouterError, add_route, find_all_routes, find_route, remove_route};
use std::collections::{HashMap, HashSet};
use tracing::Level;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

fn setup_tracing_for_tests() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::TRACE.into()))
        .with_test_writer()
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();
}

fn convert_params_to_hashmap(
    params: Option<ahash::AHashMap<String, String>>,
) -> Option<HashMap<String, String>> {
    params.map(|ahashmap| ahashmap.into_iter().collect())
}

#[test]
fn test_static_routes() {
    setup_tracing_for_tests();
    let router = Router::new();
    add_route(&router, "GET", "/home", "home_data").unwrap();
    add_route(&router, "POST", "/submit", "submit_data").unwrap();
    add_route(&router, "", "/any_method", "any_method_data").unwrap();

    let matched_home = find_route(&router, "GET", "/home", false).unwrap();
    assert_eq!(matched_home.data, "home_data");
    assert!(matched_home.params.is_none());

    let matched_submit = find_route(&router, "POST", "/submit", false).unwrap();
    assert_eq!(matched_submit.data, "submit_data");
    assert!(matched_submit.params.is_none());

    let matched_any_get = find_route(&router, "GET", "/any_method", false).unwrap();
    assert_eq!(matched_any_get.data, "any_method_data");
    let matched_any_put = find_route(&router, "PUT", "/any_method", false).unwrap();
    assert_eq!(matched_any_put.data, "any_method_data");

    add_route(&router, "GET", "/about/", "about_data").unwrap();
    assert_eq!(
        find_route(&router, "GET", "/about", false).unwrap().data,
        "about_data"
    );
    assert_eq!(
        find_route(&router, "GET", "/about/", false).unwrap().data,
        "about_data"
    );

    match find_route(&router, "GET", "/nonexistent", false) {
        Err(RouterError::RouteNotFound {
            method,
            path: error_path,
        }) => {
            assert_eq!(method, "GET");
            assert_eq!(error_path, "/nonexistent");
        }
        _ => panic!("Expected RouteNotFound for /nonexistent"),
    }
}

#[test]
fn test_parameterized_routes() {
    setup_tracing_for_tests();
    let router = Router::new();
    add_route(&router, "GET", "/users/:id", "user_by_id").unwrap();
    add_route(
        &router,
        "GET",
        "/products/:category/:product_id",
        "product_detail",
    )
    .unwrap();
    add_route(&router, "GET", "/files/*", "unnamed_param").unwrap();
    add_route(&router, "GET", "/search/:query?", "search_query_optional").unwrap();

    let matched_user = find_route(&router, "GET", "/users/123", true).unwrap();
    assert_eq!(matched_user.data, "user_by_id");
    assert_eq!(
        convert_params_to_hashmap(matched_user.params),
        Some(HashMap::from([("id".to_string(), "123".to_string())]))
    );

    let matched_product = find_route(&router, "GET", "/products/electronics/tv-456", true).unwrap();
    assert_eq!(matched_product.data, "product_detail");
    assert_eq!(
        convert_params_to_hashmap(matched_product.params),
        Some(HashMap::from([
            ("category".to_string(), "electronics".to_string()),
            ("product_id".to_string(), "tv-456".to_string()),
        ]))
    );

    let matched_files = find_route(&router, "GET", "/files/report.pdf", true).unwrap();
    assert_eq!(matched_files.data, "unnamed_param");
    assert_eq!(
        convert_params_to_hashmap(matched_files.params),
        Some(HashMap::from([("_".to_string(), "report.pdf".to_string())]))
    );

    let matched_search_with_q = find_route(&router, "GET", "/search/rust-libs", true).unwrap();
    assert_eq!(matched_search_with_q.data, "search_query_optional");
    assert_eq!(
        convert_params_to_hashmap(matched_search_with_q.params),
        Some(HashMap::from([(
            "query".to_string(),
            "rust-libs".to_string()
        )]))
    );

    let matched_search_no_q = find_route(&router, "GET", "/search/", true).unwrap();
    assert_eq!(matched_search_no_q.data, "search_query_optional");
    assert!(
        matched_search_no_q
            .params
            .is_none_or(|p| p.is_empty() || p.get("query").is_none())
    );
}

#[test]
fn test_wildcard_routes() {
    setup_tracing_for_tests();
    let router = Router::new();
    add_route(&router, "GET", "/assets/**:filepath", "serve_asset_named").unwrap();
    add_route(&router, "GET", "/data/**", "serve_data_unnamed").unwrap();

    let matched_asset = find_route(&router, "GET", "/assets/css/style.css", true).unwrap();
    assert_eq!(matched_asset.data, "serve_asset_named");
    assert_eq!(
        convert_params_to_hashmap(matched_asset.params),
        Some(HashMap::from([(
            "filepath".to_string(),
            "css/style.css".to_string()
        )]))
    );

    let matched_asset_empty = find_route(&router, "GET", "/assets/", true).unwrap();
    assert_eq!(matched_asset_empty.data, "serve_asset_named");
    assert_eq!(
        convert_params_to_hashmap(matched_asset_empty.params),
        Some(HashMap::from([("filepath".to_string(), "".to_string())])),
        "Wildcard matching empty path part should yield empty string for param"
    );

    let matched_data = find_route(&router, "GET", "/data/images/pic.jpg", true).unwrap();
    assert_eq!(matched_data.data, "serve_data_unnamed");
    assert_eq!(
        convert_params_to_hashmap(matched_data.params),
        Some(HashMap::from([(
            "_".to_string(),
            "images/pic.jpg".to_string()
        )]))
    );
}

#[test]
fn test_route_priority() {
    setup_tracing_for_tests();
    let router = Router::new();
    add_route(&router, "GET", "/prio/static", "prio_static").unwrap();
    add_route(&router, "GET", "/prio/:param", "prio_param").unwrap();
    add_route(&router, "GET", "/prio/**:wildcard_val", "prio_wildcard").unwrap();

    assert_eq!(
        find_route(&router, "GET", "/prio/static", false)
            .unwrap()
            .data,
        "prio_static"
    );

    let matched_param = find_route(&router, "GET", "/prio/avalue", true).unwrap();
    assert_eq!(matched_param.data, "prio_param");
    assert_eq!(
        convert_params_to_hashmap(matched_param.params)
            .unwrap()
            .get("param")
            .unwrap(),
        "avalue"
    );

    let matched_wildcard = find_route(&router, "GET", "/prio/avalue/another", true).unwrap();
    assert_eq!(matched_wildcard.data, "prio_wildcard");
    assert_eq!(
        convert_params_to_hashmap(matched_wildcard.params)
            .unwrap()
            .get("wildcard_val")
            .unwrap(),
        "avalue/another"
    );
}

#[test]
fn test_remove_route() {
    setup_tracing_for_tests();
    let router = Router::new();
    add_route(&router, "GET", "/temp/route1", "temp_data1").unwrap();
    add_route(&router, "GET", "/temp/:id", "temp_data_id").unwrap();

    assert_eq!(
        find_route(&router, "GET", "/temp/route1", false)
            .unwrap()
            .data,
        "temp_data1"
    );
    assert!(remove_route(&router, "GET", "/temp/route1").unwrap());

    // After removing static /temp/route1, /temp/route1 should now match /temp/:id
    let matched_after_remove = find_route(&router, "GET", "/temp/route1", true).unwrap();
    assert_eq!(
        matched_after_remove.data, "temp_data_id",
        "Path /temp/route1 should match /temp/:id after static is removed"
    );
    assert_eq!(
        convert_params_to_hashmap(matched_after_remove.params)
            .unwrap()
            .get("id")
            .unwrap(),
        "route1"
    );

    assert!(!remove_route(&router, "GET", "/nonexistent").unwrap());

    assert_eq!(
        find_route(&router, "GET", "/temp/123", true).unwrap().data,
        "temp_data_id"
    );
    assert!(remove_route(&router, "GET", "/temp/:id").unwrap());
    assert!(
        find_route(&router, "GET", "/temp/123", false).is_err(),
        "Path /temp/123 should not be found after /temp/:id is removed"
    );
    assert!(
        find_route(&router, "GET", "/temp/route1", false).is_err(),
        "Path /temp/route1 should also not be found"
    );
}

#[test]
fn test_find_all_routes_behavior() {
    setup_tracing_for_tests();
    let router = Router::<&'static str>::new();

    add_route(&router, "GET", "/config", "config_base").unwrap();
    add_route(&router, "GET", "/config/:key", "config_key_specific").unwrap();
    add_route(&router, "GET", "/config/:key/value", "config_key_value").unwrap();
    add_route(&router, "GET", "/config/**:path", "config_wildcard").unwrap();
    add_route(&router, "GET", "/**:root_wild", "root_wild_data").unwrap();

    let matches1 = find_all_routes(&router, "GET", "/config/timeout", true);
    let data1_set: HashSet<_> = matches1.iter().map(|m| m.data).collect();
    assert_eq!(
        data1_set.len(),
        3,
        "Matches for /config/timeout: {:?}",
        data1_set
    );
    assert!(data1_set.contains(&"config_key_specific"));
    assert!(data1_set.contains(&"config_wildcard"));
    assert!(data1_set.contains(&"root_wild_data"));

    let matches2 = find_all_routes(&router, "GET", "/config/user/name", true);
    let data2_set: HashSet<_> = matches2.iter().map(|m| m.data).collect();
    assert_eq!(
        data2_set.len(),
        2,
        "Matches for /config/user/name: {:?}",
        data2_set
    );
    assert!(data2_set.contains(&"config_wildcard"));
    assert!(data2_set.contains(&"root_wild_data"));
    for m in matches2 {
        if m.data == "config_wildcard" {
            assert_eq!(
                convert_params_to_hashmap(m.params),
                Some(HashMap::from([(
                    "path".to_string(),
                    "user/name".to_string()
                )]))
            );
        }
    }

    let matches3 = find_all_routes(&router, "GET", "/config", true);
    let data3_set: HashSet<_> = matches3.iter().map(|m| m.data).collect();
    assert_eq!(data3_set.len(), 3, "Matches for /config: {:?}", data3_set);
    assert!(data3_set.contains(&"config_base"));
    assert!(data3_set.contains(&"config_wildcard"));
    assert!(data3_set.contains(&"root_wild_data"));
    for m in matches3 {
        if m.data == "config_wildcard" {
            assert_eq!(
                convert_params_to_hashmap(m.params),
                Some(HashMap::from([("path".to_string(), "".to_string())]))
            );
        }
    }
}

#[test]
fn test_invalid_patterns_add_route() {
    setup_tracing_for_tests();
    let router = Router::<&str>::new();
    assert!(matches!(
        add_route(&router, "GET", "/path/:", "data"),
        Err(RouterError::InvalidSegment { segment, .. }) if segment == ":"
    ));
    assert!(matches!(
        add_route(&router, "GET", "/path/**:", "data"),
        Err(RouterError::InvalidSegment { segment, .. }) if segment == "**:"
    ));
    assert!(matches!(
        add_route(&router, "GET", "/path/**:name/extra", "data"),
        Err(RouterError::InvalidSegment { segment, reason,.. }) if segment == "**:name" && reason.contains("wildcard (**) must be the last segment")
    ));
    assert!(matches!(
        add_route(&router, "GET", "/path/**/extra", "data"),
        Err(RouterError::InvalidSegment { segment, reason,.. }) if segment == "**" && reason.contains("wildcard (**) must be the last segment")
    ));
}

#[test]
fn test_optional_trailing_param_find_route() {
    setup_tracing_for_tests();
    let router = Router::new();
    add_route(&router, "GET", "/api/items/:id?", "optional_item_id").unwrap();

    let matched_with_id = find_route(&router, "GET", "/api/items/123", true).unwrap();
    assert_eq!(matched_with_id.data, "optional_item_id");
    assert_eq!(
        convert_params_to_hashmap(matched_with_id.params),
        Some(HashMap::from([("id".to_string(), "123".to_string())]))
    );

    let matched_without_id = find_route(&router, "GET", "/api/items/", true).unwrap();
    assert_eq!(matched_without_id.data, "optional_item_id");
    assert!(
        matched_without_id
            .params
            .is_none_or(|p| p.get("id").is_none() && p.is_empty())
    );
}

#[test]
fn test_optional_trailing_wildcard_find_route() {
    setup_tracing_for_tests();
    let router = Router::new();
    add_route(&router, "GET", "/api/files/**:path?", "optional_files_path").unwrap();

    let matched_with_path = find_route(&router, "GET", "/api/files/docs/report.pdf", true).unwrap();
    assert_eq!(matched_with_path.data, "optional_files_path");
    assert_eq!(
        convert_params_to_hashmap(matched_with_path.params),
        Some(HashMap::from([(
            "path".to_string(),
            "docs/report.pdf".to_string()
        )]))
    );

    let matched_without_path = find_route(&router, "GET", "/api/files/", true).unwrap();
    assert_eq!(matched_without_path.data, "optional_files_path");
    assert_eq!(
        convert_params_to_hashmap(matched_without_path.params),
        Some(HashMap::from([("path".to_string(), "".to_string())])),
        "Optional wildcard matching empty should give empty string for param"
    );
}
