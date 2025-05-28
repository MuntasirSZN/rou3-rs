//! Contains type definitions used throughout the rou3 router.
//!
//! This module defines structures for storing method-specific data,
//! parameter information, and the result of a route match.

use ahash::AHashMap;

/// Stores the data associated with a specific HTTP method on a route,
/// along with information about any parameters defined in the route's path.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MethodData<T> {
    /// The actual data or handler associated with this route and method.
    pub data: T,
    /// An optional list of parameter entries derived from the route pattern.
    /// `None` if the route has no parameters. Otherwise, `Some(Vec<ParamEntry>)`
    /// detailing how to extract parameters from a matched path.
    pub params_map: Option<Vec<ParamEntry>>,
}

impl<T: Clone> MethodData<T> {
    /// Constructs new `MethodData`.
    pub fn new(data: T, params_map: Option<Vec<ParamEntry>>) -> Self {
        Self { data, params_map }
    }
}

/// Describes a parameter captured from a route's path pattern.
///
/// Parameters can be simple placeholders (e.g., `/:id`), unnamed placeholders (`/*`),
/// or wildcards that capture multiple segments (`/**:name`). Optionality is
/// typically denoted by a `?` suffix in the path pattern (e.g., `/:id?`).
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ParamEntry {
    /// A parameter at a specific segment index.
    /// `usize` is the segment index in the path.
    /// `String` is the name of the parameter.
    /// `bool` indicates if the parameter segment is optional.
    Index(usize, String, bool),
    /// A wildcard parameter that captures all segments from a starting index.
    /// `usize` is the starting segment index.
    /// `String` is the name of the parameter.
    /// `bool` indicates if the wildcard itself is optional.
    Wildcard(usize, String, bool),
}

/// Represents a successfully matched route.
///
/// It contains the data associated with the route and an optional map of
/// extracted parameters if the route was dynamic and parameter capture was requested.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MatchedRoute<T: Eq> {
    /// The data or handler associated with the matched route.
    pub data: T,
    /// An optional map of extracted parameters.
    /// Keys are parameter names (e.g., "id"), and values are the captured strings from the path.
    /// This is `None` if no parameters were captured or if capture was disabled.
    pub params: Option<AHashMap<String, String>>,
}
