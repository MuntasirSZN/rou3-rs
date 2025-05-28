use crate::{
    context::{Node, Router},
    error::RouterError,
    operations::util::{normalize, split_path},
    types::{MethodData, ParamEntry},
};

/// Parses path segments to identify and map named parameters, wildcards, and optional segments.
///
/// This function iterates over the segments of a route pattern. For each segment, it checks if it
/// represents a named parameter (e.g., `:id`), an unnamed parameter (`*`), a named wildcard (`**:name`),
/// or an unnamed wildcard (`**`). It also handles optional segments denoted by a trailing `?`.
///
/// The information extracted is stored as a vector of `ParamEntry` enums, which detail the type
/// of parameter, its name (if applicable), its index in the path segments, and whether it's optional.
///
/// # Arguments
/// * `segments`: A slice of string slices, where each element is a segment of the route path.
///
/// # Returns
/// * `Option<Vec<ParamEntry>>`: Returns `Some` with a vector of `ParamEntry` if the path contains
///   any dynamic segments (parameters or wildcards). Returns `None` if all segments are static.
pub(crate) fn build_param_entries_for_pattern_segments(
    segments: &[&str],
) -> Result<Option<Vec<ParamEntry>>, RouterError> {
    let mut params_map = Vec::new();
    let mut has_params = false;

    for (i, seg_str_ref) in segments.iter().enumerate() {
        let mut segment_str = *seg_str_ref;
        let is_segment_optional = segment_str.ends_with('?');
        if is_segment_optional {
            segment_str = &segment_str[..segment_str.len() - 1];
        }

        if segment_str.is_empty() && i < segments.len() - 1 {
            return Err(RouterError::InvalidSegment {
                segment: format!("'{segment_str}' at index {i}"),
                reason: "empty segments are not allowed unless at the very end".to_string(),
            });
        }

        if segment_str.starts_with("**") {
            has_params = true;
            let param_name = if let Some(stripped_name) = segment_str.strip_prefix("**:") {
                if stripped_name.is_empty() {
                    return Err(RouterError::InvalidSegment {
                        segment: segment_str.to_string(),
                        reason: "named wildcard must have a name".to_string(),
                    });
                }
                stripped_name.to_string()
            } else if segment_str == "**" {
                "_".to_string()
            } else {
                return Err(RouterError::InvalidSegment {
                    segment: segment_str.to_string(),
                    reason: "invalid wildcard format".to_string(),
                });
            };
            params_map.push(ParamEntry::Wildcard(i, param_name, is_segment_optional));
            if i < segments.len() - 1 {
                return Err(RouterError::InvalidSegment {
                    segment: segment_str.to_string(),
                    reason: "wildcard (**) must be the last segment".to_string(),
                });
            }
            break;
        } else if let Some(stripped_name) = segment_str.strip_prefix(':') {
            has_params = true;
            if stripped_name.is_empty() {
                return Err(RouterError::InvalidSegment {
                    segment: segment_str.to_string(),
                    reason: "named parameter must have a name".to_string(),
                });
            }
            params_map.push(ParamEntry::Index(
                i,
                stripped_name.to_string(),
                is_segment_optional,
            ));
        } else if segment_str == "*" {
            has_params = true;
            params_map.push(ParamEntry::Index(i, "_".to_string(), is_segment_optional));
        } else if segment_str.contains([':', '*'].as_ref()) {
            return Err(RouterError::InvalidSegment {
                segment: segment_str.to_string(),
                reason: "parameter/wildcard characters must appear at the start".to_string(),
            });
        }
    }

    if has_params {
        Ok(Some(params_map))
    } else {
        Ok(None)
    }
}

/// Creates a new `Node<T>` instance, boxed for heap allocation.
/// This is a helper function to reduce boilerplate when creating new nodes,
/// especially for insertion into `AHashMap` or `Option` fields within another `Node`.
fn new_node_boxed<T>() -> Box<Node<T>> {
    Box::new(Node::new())
}

/// Adds a route to the router.
///
/// This function parses the `path` string, normalizes it, and splits it into segments.
/// It then traverses the routing tree, creating nodes as necessary, until it reaches
/// the node corresponding to the final segment of the path. The `data` (handler)
/// is then associated with the specified `method` at this terminal node.
///
/// If the path is purely static (no parameters or wildcards), it's also added to a
/// separate `static_map` in the `Router` for potentially faster lookups.
///
/// # Arguments
/// * `router`: A reference to the `Router` instance.
/// * `method`: The HTTP method (e.g., "GET", "POST") for this route. An empty string `""`
///   can be used to specify a handler for any method not explicitly defined for this path.
/// * `path`: The path pattern for the route (e.g., "/users/:id").
/// * `data`: The data or handler to associate with this route. This data must be `Clone`.
///
/// # Returns
/// * `Result<(), RouterError>`: Returns `Ok(())` on successful addition. Returns an `Err`
///   of type `RouterError` if the path pattern is invalid (e.g., malformed parameter,
///   misplaced wildcard).
///
/// # Panics
/// This function may panic if acquiring write locks on the router's internal structures fails,
/// though this is typically indicative of a deeper issue like lock poisoning.
pub fn add_route<T: Clone>(
    router: &Router<T>,
    method: &str,
    path: &str,
    data: T,
) -> Result<(), RouterError> {
    let normalized_path_string = normalize(path);
    let segments: Vec<&str> = split_path(&normalized_path_string).collect();

    let params_map_for_route = build_param_entries_for_pattern_segments(&segments)?;

    if params_map_for_route.is_none() {
        let is_purely_static_check = !normalized_path_string.contains([':', '*']);
        if is_purely_static_check {
            let mut static_map_lock = router.static_map.write();
            static_map_lock
                .entry(normalized_path_string.clone())
                .or_default()
                .entry(method.to_string())
                .or_default()
                .push(MethodData::new(data.clone(), None));
        }
    }

    let mut current_node_mut_ref: &mut Node<T> = &mut router.root.write();

    for segment_str_ref in &segments {
        let segment_for_logic = *segment_str_ref;

        let temp_segment_for_type_check = segment_for_logic
            .strip_suffix('?')
            .unwrap_or(segment_for_logic);

        if temp_segment_for_type_check.starts_with("**") {
            current_node_mut_ref = &mut **current_node_mut_ref
                .wildcard_child
                .get_or_insert_with(new_node_boxed);
            break;
        } else if temp_segment_for_type_check.starts_with(':') || temp_segment_for_type_check == "*"
        {
            current_node_mut_ref = &mut **current_node_mut_ref
                .param_child
                .get_or_insert_with(new_node_boxed);
        } else {
            current_node_mut_ref = &mut **current_node_mut_ref
                .static_children
                .entry((*segment_str_ref).to_string())
                .or_insert_with(new_node_boxed);
        }
    }

    current_node_mut_ref
        .methods
        .entry(method.to_string())
        .or_default()
        .push(MethodData::new(data, params_map_for_route));

    Ok(())
}
