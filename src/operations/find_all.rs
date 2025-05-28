use crate::{
    context::{Node, Router},
    operations::util::{extract_all_params, normalize, split_path},
    types::{MatchedRoute, MethodData, ParamEntry},
};
use std::collections::HashSet;

fn is_last_param_optional_for_find_all<T>(md: &MethodData<T>) -> bool {
    md.params_map.as_ref().is_some_and(|pm| {
        pm.last().is_some_and(|p_entry| match p_entry {
            ParamEntry::Index(_, _, is_opt) => *is_opt,
            ParamEntry::Wildcard(_, _, is_opt) => *is_opt,
        })
    })
}

/// Finds all routes registered in the router that match a given HTTP method.
///
/// This function traverses the entire routing tree and collects all routes.
/// The behavior regarding the `method_filter` is as follows:
/// - If `method_filter` is a specific method (e.g., "GET"), it returns routes
///   registered with that specific method OR routes registered with an empty method
///   string `""` (which signifies an "ANY" method handler).
/// - If `method_filter` is an empty string `""`, it *only* returns routes that
///   were explicitly registered with an empty method string `""`.
///
/// To avoid duplicates if the same data payload `T` is registered for multiple
/// route patterns (which is possible), this function uses a `HashSet` to track
/// seen data payloads, ensuring each unique `T` is returned only once.
/// The `params` field in the returned `MatchedRoute` instances will be `None`,
/// as `find_all_routes` does not perform path matching or parameter extraction.
///
/// The order of routes returned is influenced by the traversal strategy:
/// it first iterates over static children (sorted alphabetically by segment name),
/// then the parametric child, and finally the wildcard child.
///
/// # Arguments
/// * `router`: A reference to the `Router` instance.
/// * `method_filter`: The HTTP method to filter routes by. See behavior description above.
///
/// # Returns
/// * `Vec<MatchedRoute<T>>`: A vector of `MatchedRoute` instances. The `data` field
///   contains the user-provided data, and `params` is always `None`.
///   `T` must implement `Clone + Eq + std::hash::Hash` for deduplication.
///
/// # Panics
/// This function may panic if acquiring read locks on the router's internal structures fails.
pub fn find_all_routes<T: Clone + Eq + std::hash::Hash>(
    router: &Router<T>,
    method: &str,
    path: &str,
    capture_params: bool,
) -> Vec<MatchedRoute<T>> {
    let normalized_path_string = normalize(path);
    let segments: Vec<&str> = split_path(&normalized_path_string).collect();

    let mut collected_method_data_refs: Vec<&MethodData<T>> = Vec::new();
    let root_lock = router.root.read();

    find_all_recursive_ordered(
        &*root_lock,
        method,
        &segments,
        0,
        &mut collected_method_data_refs,
    );

    let mut results = Vec::new();
    let mut seen_t_values = HashSet::new();

    for md_ref in collected_method_data_refs {
        if seen_t_values.insert(md_ref.data.clone()) {
            // Deduplicate by T value
            let params = if capture_params {
                extract_all_params(&segments, &md_ref.params_map)
            } else {
                None
            };
            results.push(MatchedRoute {
                data: md_ref.data.clone(),
                params,
            });
        }
    }
    results
}

fn find_all_recursive_ordered<'a, T: Clone + Eq + std::hash::Hash>(
    node: &'a Node<T>,
    method: &str,
    segments: &[&str],
    idx: usize,
    matches: &mut Vec<&'a MethodData<T>>,
) {
    // 1. Wildcard child of current node (matches remaining segments from this point)
    if let Some(wildcard_child_node) = &node.wildcard_child {
        if let Some(handlers) = wildcard_child_node
            .methods
            .get(method)
            .or_else(|| wildcard_child_node.methods.get(""))
        {
            matches.extend(handlers.iter());
        }
    }

    let current_segment_val = if idx < segments.len() {
        Some(segments[idx])
    } else {
        None
    };

    // 2. Parametric child
    if let Some(param_child_node) = &node.param_child {
        if current_segment_val.is_some() {
            find_all_recursive_ordered(param_child_node, method, segments, idx + 1, matches);
        }
        if idx == segments.len() {
            // Path ends here, check if param child can match optionally
            if let Some(handlers) = param_child_node
                .methods
                .get(method)
                .or_else(|| param_child_node.methods.get(""))
            {
                if handlers.iter().any(is_last_param_optional_for_find_all) {
                    // Check if any handler on param child is for an optional pattern
                    matches.extend(handlers.iter());
                }
            }
        }
    }

    // 3. Static child for current segment
    if let Some(segment_val) = current_segment_val {
        if let Some(static_child_node) = node.static_children.get(segment_val) {
            find_all_recursive_ordered(static_child_node, method, segments, idx + 1, matches);
        }
    }

    // 4. Current node methods if path ends here
    if idx == segments.len() {
        if let Some(handlers) = node.methods.get(method).or_else(|| node.methods.get("")) {
            matches.extend(handlers.iter());
        }
    }
}
