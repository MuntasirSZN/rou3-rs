use crate::{
    context::{Node, Router},
    error::RouterError,
    operations::util::{extract_all_params, normalize, split_path},
    types::{MatchedRoute, MethodData, ParamEntry},
};

/// Finds a route matching the given HTTP method and path.
///
/// This function normalizes the input `path`, splits it into segments, and then
/// traverses the routing tree starting from the `router`'s root node. It attempts
/// to match each segment of the path against static children, parametric children,
/// or wildcard children of the current node in the tree.
///
/// If a match is found, it returns a `MatchedRoute` containing the data associated
/// with the route and, if `capture` is true, any extracted parameters.
///
/// # Arguments
/// * `router`: A reference to the `Router` instance.
/// * `method`: The HTTP method (e.g., "GET", "POST") to match.
/// * `path`: The request path to match against the router's patterns.
/// * `capture`: A boolean indicating whether path parameters should be extracted.
///   If `false`, the `params` field in the returned `MatchedRoute` will be `None`,
///   even if the matched route pattern contains parameters. This can be a
///   performance optimization if parameters are not needed.
///
/// # Returns
/// * `Result<MatchedRoute<T>, RouterError>`:
///   - `Ok(MatchedRoute<T>)` if a route is successfully found.
///   - `Err(RouterError::RouteNotFound)` if no route matches the given method and path.
///   - Other `RouterError` variants might occur if there's an issue with path processing,
///     though `RouteNotFound` is the most common error for this function.
///
/// # Panics
/// This function may panic if acquiring read locks on the router's internal structures fails,
/// which typically indicates a deeper issue like lock poisoning.
pub fn find_route<T: Clone + Eq>(
    router: &Router<T>,
    method: &str,
    path: &str,
    capture: bool,
) -> Result<MatchedRoute<T>, RouterError> {
    let normalized_path_string = normalize(path);

    if !normalized_path_string.contains([':', '*']) {
        let static_map_read_guard = router.static_map.read();
        if let Some(methods_for_path) = static_map_read_guard.get(&normalized_path_string) {
            if let Some(method_data_list) = methods_for_path
                .get(method)
                .or_else(|| methods_for_path.get(""))
            {
                if let Some(md) = method_data_list.first() {
                    if md.params_map.is_none() {
                        return Ok(MatchedRoute {
                            data: md.data.clone(),
                            params: None,
                        });
                    }
                }
            }
        }
    }

    let segments: Vec<&str> = split_path(&normalized_path_string).collect();
    let root_lock = router.root.read();

    match lookup_node_recursive(&*root_lock, method, &segments, 0) {
        Some(md) => {
            let params = if capture {
                extract_all_params(&segments, &md.params_map)
            } else {
                None
            };
            Ok(MatchedRoute {
                data: md.data.clone(),
                params,
            })
        }
        None => Err(RouterError::RouteNotFound {
            method: method.to_string(),
            path: path.to_string(),
        }),
    }
}

fn is_handler_for_optional_pattern<T>(md: &MethodData<T>) -> bool {
    md.params_map.as_ref().is_some_and(|pm| {
        pm.last().is_some_and(|p_entry| match p_entry {
            ParamEntry::Index(_, _, is_opt) => *is_opt,
            ParamEntry::Wildcard(_, _, is_opt) => *is_opt,
        })
    })
}

fn lookup_node_recursive<'a, T: Clone + Eq>(
    node: &'a Node<T>,
    method: &str,
    segments: &[&str],
    idx: usize,
) -> Option<&'a MethodData<T>> {
    // Base case: All segments of the input path have been consumed
    if idx == segments.len() {
        // 1. Check for a handler on the current node
        if let Some(handlers) = node.methods.get(method).or_else(|| node.methods.get("")) {
            if let Some(md) = handlers.first() {
                // Assuming first is highest precedence if multiple
                return Some(md);
            }
        }

        // 2. If no handler on current node, check if an optional parameter child can match "empty"
        if let Some(param_child_node) = &node.param_child {
            if let Some(handlers) = param_child_node
                .methods
                .get(method)
                .or_else(|| param_child_node.methods.get(""))
            {
                if handlers.iter().any(is_handler_for_optional_pattern) {
                    if let Some(md) = handlers.first() {
                        return Some(md);
                    }
                }
            }
        }

        // 3. If still no match, check if a wildcard child can match "empty"
        // A wildcard (e.g., /foo/**:name) inherently matches an empty sequence of segments.
        if let Some(wildcard_child_node) = &node.wildcard_child {
            if let Some(handlers) = wildcard_child_node
                .methods
                .get(method)
                .or_else(|| wildcard_child_node.methods.get(""))
            {
                // If there's any handler on the wildcard child, it implies it can match an empty suffix.
                if let Some(md) = handlers.first() {
                    return Some(md);
                }
            }
        }
        return None;
    }

    // Recursive step:
    let current_segment_value = segments[idx];

    // 1. Try static child match
    if let Some(static_child_node) = node.static_children.get(current_segment_value) {
        if let Some(found_md) = lookup_node_recursive(static_child_node, method, segments, idx + 1)
        {
            return Some(found_md);
        }
    }

    // 2. Try parametric child match
    if let Some(param_child_node) = &node.param_child {
        if let Some(found_md) = lookup_node_recursive(param_child_node, method, segments, idx + 1) {
            return Some(found_md);
        }
    }

    // 3. Try wildcard child match (consumes all remaining segments from this point)
    if let Some(wildcard_child_node) = &node.wildcard_child {
        if let Some(handlers) = wildcard_child_node
            .methods
            .get(method)
            .or_else(|| wildcard_child_node.methods.get(""))
        {
            if let Some(md) = handlers.first() {
                return Some(md);
            }
        }
    }
    None
}
