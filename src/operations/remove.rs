use crate::{
    context::{Node, Router},
    error::RouterError,
    operations::util::{normalize, split_path},
};

/// Removes a route handler.
///
/// This function first normalizes the `path_pattern_to_remove` and determines its
/// parameter structure (if any). It then traverses the routing tree to find the node
/// corresponding to this pattern and removes the handler associated with the specified `method`.
///
/// If the removal of a handler leaves a node or a branch of the tree empty (i.e., no
/// other handlers or child nodes), this function will prune those empty nodes to keep
/// the routing tree compact.
///
/// If the removed route was purely static, it's also removed from the `router`'s
/// `static_map` optimization.
///
/// # Arguments
/// * `router`: A reference to the `Router` instance.
/// * `method`: The HTTP method of the route handler to remove.
/// * `path_pattern_to_remove`: The path pattern of the route handler to remove.
///   This must exactly match the pattern used when the route was added.
///
/// # Returns
/// * `Result<usize, RouterError>`:
///   - `Ok(usize)`: The number of handlers removed (typically 1 if a matching handler
///     was found and removed, or 0 if no such handler was found).
///   - `Err(RouterError)`: If the `path_pattern_to_remove` is invalid (e.g., malformed
///     parameter syntax), a `RouterError` is returned.
///
/// # Panics
/// This function may panic if acquiring write locks on the router's internal structures fails,
/// which usually indicates a deeper issue like lock poisoning.
pub fn remove_route<T>(
    router: &Router<T>,
    method: &str,
    path_pattern_to_remove: &str,
) -> Result<bool, RouterError> {
    let normalized_path_string = normalize(path_pattern_to_remove);
    let segments: Vec<&str> = split_path(&normalized_path_string).collect();

    let mut root_lock = router.root.write();
    let mut modified_in_trie = false;

    if segments.is_empty() {
        if let Some(handlers) = root_lock.methods.get_mut(method) {
            if !handlers.is_empty() {
                handlers.clear();
                modified_in_trie = true;
            }
        }
        if root_lock.methods.get(method).is_some_and(|h| h.is_empty()) {
            root_lock.methods.remove(method);
        }
    } else {
        modified_in_trie = recurse_remove(&mut *root_lock, method, &segments, 0);
    }

    let mut modified_in_static_map = false;
    if !normalized_path_string.contains([':', '*']) {
        let mut static_map_lock = router.static_map.write();
        if let Some(methods_for_path) = static_map_lock.get_mut(&normalized_path_string) {
            if methods_for_path.remove(method).is_some() {
                modified_in_static_map = true;
            }
            if methods_for_path.is_empty() {
                static_map_lock.shift_remove(&normalized_path_string);
            }
        }
    }

    Ok(modified_in_trie || modified_in_static_map)
}

/// Recursively traverses and removes handlers. Returns true if modification happened in the subtree.
fn recurse_remove<T>(
    current_node: &mut Node<T>,
    method: &str,
    pattern_segments: &[&str],
    idx: usize,
) -> bool {
    if idx >= pattern_segments.len() {
        let mut handler_removed_at_this_node = false;
        if let Some(handlers) = current_node.methods.get_mut(method) {
            if !handlers.is_empty() {
                handlers.clear();
                handler_removed_at_this_node = true;
            }
        }
        if current_node
            .methods
            .get(method)
            .is_some_and(|h| h.is_empty())
        {
            current_node.methods.remove(method);
        }
        return handler_removed_at_this_node;
    }

    let segment_str_of_pattern = pattern_segments[idx];
    let mut modified_in_child_branch = false;

    let temp_segment_for_type_check = segment_str_of_pattern
        .strip_suffix('?')
        .unwrap_or(segment_str_of_pattern);

    if temp_segment_for_type_check.starts_with("**") {
        if let Some(wc_child_box) = current_node.wildcard_child.as_mut() {
            if recurse_remove(wc_child_box, method, pattern_segments, idx + 1) {
                modified_in_child_branch = true;
                if wc_child_box.as_ref().is_empty_recursive() {
                    current_node.wildcard_child = None;
                }
            }
        }
    } else if temp_segment_for_type_check.starts_with(':') || temp_segment_for_type_check == "*" {
        if let Some(param_child_box) = current_node.param_child.as_mut() {
            if recurse_remove(param_child_box, method, pattern_segments, idx + 1) {
                modified_in_child_branch = true;
                if param_child_box.as_ref().is_empty_recursive() {
                    current_node.param_child = None;
                }
            }
        }
    } else if let Some(static_child_box) =
        current_node.static_children.get_mut(segment_str_of_pattern)
    {
        if recurse_remove(static_child_box, method, pattern_segments, idx + 1) {
            modified_in_child_branch = true;
            if static_child_box.as_ref().is_empty_recursive() {
                current_node.static_children.remove(segment_str_of_pattern);
            }
        }
    }
    modified_in_child_branch
}
