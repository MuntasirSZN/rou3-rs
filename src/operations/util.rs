use crate::types::ParamEntry;
use ahash::AHashMap;

/// Normalizes a path string by removing leading/trailing slashes and collapsing multiple internal slashes.
///
/// Multiple slashes (e.g., `//`) are effectively treated as one by the `split('/')`
/// method used in `split_path` after empty segments are filtered. This function
/// primarily ensures that paths like `/foo/`, `foo/`, and `/foo` are all treated
/// as `foo` in terms of their start and end, which simplifies segment processing.
/// An empty path or a path consisting only of slashes (e.g., `/`, `///`) becomes an empty string.
///
/// # Examples
/// ```rust
/// assert_eq!(rou3::operations::util::normalize("/foo/bar/"), "foo/bar");
/// assert_eq!(rou3::operations::util::normalize("foo/bar"), "foo/bar");
/// assert_eq!(rou3::operations::util::normalize("/"), "");
/// assert_eq!(rou3::operations::util::normalize(""), "");
/// ```
///
/// # Arguments
/// * `path`: The path string to normalize.
///
/// # Returns
/// * A string slice representing the normalized path. Returns an empty string for paths that are empty or consist only of slashes.
pub fn normalize(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segments.is_empty() {
        String::new()
    } else {
        segments.join("/")
    }
}

/// Splits a pre-normalized path string into its constituent segments.
/// The input `normalized_path` is expected to be the output of `normalize()`.
///
/// This function first calls `normalize` to ensure the path is in a standard form
/// (no leading/trailing slashes). Then, it splits the path by `/` and filters out
/// any empty segments that might result from multiple consecutive slashes (though
/// `normalize` should handle most of this).
///
/// # Example
/// ```rust
/// // Assuming split_path is publicly accessible or tested internally.
/// // let segments: Vec<&str> = rou3::operations::util::split_path("/foo//bar/").collect();
/// // assert_eq!(segments, vec!["foo", "bar"]);
/// // let root_segments: Vec<&str> = rou3::operations::util::split_path("/").collect();
/// // assert_eq!(root_segments, Vec::<&str>::new());
/// ```
///
/// # Arguments
/// * `path`: The path string to split.
///
/// # Returns
/// * An iterator over the string slices representing the path segments.
#[inline]
pub fn split_path(normalized_path: &str) -> impl Iterator<Item = &str> {
    normalized_path.split('/').filter(|s| !s.is_empty())
}

/// Extracts parameters from path segments based on a list of `ParamEntry` definitions.
pub(crate) fn extract_all_params(
    path_segments: &[&str],
    param_entries_opt: &Option<Vec<ParamEntry>>,
) -> Option<AHashMap<String, String>> {
    let entries = param_entries_opt.as_ref()?;
    if entries.is_empty() {
        return None;
    }

    let mut extracted_params = AHashMap::new();
    for entry in entries {
        match entry {
            ParamEntry::Index(segment_idx, param_name, is_optional) => {
                if *segment_idx < path_segments.len() {
                    let value = path_segments[*segment_idx].to_string();
                    extracted_params.insert(param_name.clone(), value);
                } else if *is_optional {
                    // Optional parameter not present, do not add to map
                }
            }
            ParamEntry::Wildcard(start_idx, param_name, _is_optional) => {
                // A wildcard captures segments from start_idx to the end.
                // If start_idx is at or beyond the number of segments, it captures an empty string.
                let value = if *start_idx < path_segments.len() {
                    path_segments[*start_idx..].join("/")
                } else {
                    String::new() // Capture empty if wildcard starts at/after end of segments
                };
                extracted_params.insert(param_name.clone(), value);
            }
        }
    }

    if extracted_params.is_empty() {
        None
    } else {
        Some(extracted_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ParamEntry;

    #[test]
    fn test_normalize_paths() {
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("/"), "");
        assert_eq!(normalize("path"), "path");
        assert_eq!(normalize("/path"), "path");
        assert_eq!(normalize("path/"), "path");
        assert_eq!(normalize("/path/"), "path");
        assert_eq!(normalize("path/to/resource"), "path/to/resource");
        assert_eq!(normalize("//path//to//resource//"), "path/to/resource");
        assert_eq!(normalize("foo//bar"), "foo/bar");
        assert_eq!(normalize("///foo///bar///"), "foo/bar");
        assert_eq!(normalize("foo/bar///"), "foo/bar");
    }

    #[test]
    fn test_split_paths() {
        // Test with pre-normalized paths as `split_path` expects normalized input
        assert_eq!(split_path("").collect::<Vec<&str>>(), Vec::<&str>::new());
        assert_eq!(split_path("path").collect::<Vec<&str>>(), vec!["path"]);
        assert_eq!(
            split_path("path/to/resource").collect::<Vec<&str>>(),
            vec!["path", "to", "resource"]
        );
    }

    #[test]
    fn test_extract_all_params_basic() {
        let segments = vec!["users", "123", "posts"];
        let param_entries = Some(vec![
            ParamEntry::Index(1, "userId".to_string(), false),
            ParamEntry::Index(2, "type".to_string(), false),
        ]);
        let params = extract_all_params(&segments, &param_entries).unwrap();
        assert_eq!(params.get("userId").unwrap(), "123");
        assert_eq!(params.get("type").unwrap(), "posts");

        let param_entries_wildcard = Some(vec![ParamEntry::Wildcard(1, "rest".to_string(), false)]);
        let params_wild = extract_all_params(&segments, &param_entries_wildcard).unwrap();
        assert_eq!(params_wild.get("rest").unwrap(), "123/posts");
    }

    #[test]
    fn test_extract_all_params_optional() {
        let segments_full = vec!["search", "rust"];
        let param_entries_opt = Some(vec![
            ParamEntry::Index(0, "verb".to_string(), false),
            ParamEntry::Index(1, "query".to_string(), true),
        ]);
        let params_full = extract_all_params(&segments_full, &param_entries_opt).unwrap();
        assert_eq!(params_full.get("verb").unwrap(), "search");
        assert_eq!(params_full.get("query").unwrap(), "rust");

        let segments_partial = vec!["search"];
        let params_partial = extract_all_params(&segments_partial, &param_entries_opt).unwrap();
        assert_eq!(params_partial.get("verb").unwrap(), "search");
        assert!(params_partial.get("query").is_none());

        let param_entries_only_opt = Some(vec![ParamEntry::Index(0, "maybe".to_string(), true)]);
        let segments_empty: Vec<&str> = vec![];
        let params_none = extract_all_params(&segments_empty, &param_entries_only_opt);
        assert!(params_none.is_none());

        let segments_present = vec!["value"];
        let params_opt_present =
            extract_all_params(&segments_present, &param_entries_only_opt).unwrap();
        assert_eq!(params_opt_present.get("maybe").unwrap(), "value");
    }

    #[test]
    fn test_extract_wildcard_empty() {
        // Path is /files, so segments = ["files"]
        // Wildcard is /**:path at index 1 (after "files")
        let segments: Vec<&str> = vec!["files"];
        let param_entries = Some(vec![ParamEntry::Wildcard(1, "path".to_string(), true)]);
        let params = extract_all_params(&segments, &param_entries).unwrap();
        assert_eq!(
            params.get("path").unwrap(),
            "",
            "Wildcard starting after all segments should capture empty string"
        );

        // Path is /, so segments = []
        // Wildcard is /**:all at index 0
        let segments_root_wild: Vec<&str> = vec![];
        let param_entries_root_wild = Some(vec![ParamEntry::Wildcard(0, "all".to_string(), true)]);
        let params_root =
            extract_all_params(&segments_root_wild, &param_entries_root_wild).unwrap();
        assert_eq!(
            params_root.get("all").unwrap(),
            "",
            "Wildcard at root for empty path should capture empty string"
        );
    }
}
