//! Defines the core data structures for the router, primarily `Node` and `Router`.
//!
//! The `Node` represents a part of a route path in the routing tree. Each node
//! can have children (static, parametric, or wildcard) and can store handlers
//! associated with HTTP methods for the path segment it represents.
//!
//! The `Router` is the main entry point, holding the root of the routing tree
//! and a separate map for optimized lookups of purely static routes.

use crate::types::MethodData;
use ahash::AHashMap;
use indexmap::IndexMap;
use parking_lot::RwLock;

/// Represents a node in the routing tree.
#[derive(Debug, Clone)]
pub struct Node<T> {
    /// Stores handlers for HTTP methods. Key is method string (e.g., "GET", "" for ANY).
    pub methods: AHashMap<String, Vec<MethodData<T>>>,
    /// Children nodes for static path segments.
    pub static_children: AHashMap<String, Box<Node<T>>>,
    /// Child node for a parameterized path segment (e.g., `/:id`, `/*`).
    pub param_child: Option<Box<Node<T>>>,
    /// Child node for a wildcard path segment (e.g., `/**:filepath`).
    pub wildcard_child: Option<Box<Node<T>>>,
}

impl<T> Default for Node<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Node<T> {
    /// Constructs a new `Node` with empty method handlers and no children.
    pub fn new() -> Self {
        Self {
            methods: AHashMap::default(),
            static_children: AHashMap::default(),
            param_child: None,
            wildcard_child: None,
        }
    }

    /// Checks if this node is effectively empty (no handlers and no children).
    /// Used for pruning during route removal.
    pub fn is_empty_recursive(&self) -> bool {
        self.methods.is_empty()
            && self.static_children.is_empty()
            && self.param_child.is_none()
            && self.wildcard_child.is_none()
    }
}

/// Type alias for the value part of the static_map in the Router.
/// Represents a map from HTTP method strings to a list of method-specific data.
pub type StaticPathMethods<T> = AHashMap<String, Vec<MethodData<T>>>;

/// The main router structure.
#[derive(Debug)]
pub struct Router<T> {
    /// The root node of the routing tree.
    pub root: RwLock<Box<Node<T>>>,
    /// Optimized map for purely static routes.
    /// Key: normalized path string.
    /// Value: Map of method string to list of `MethodData`.
    pub static_map: RwLock<IndexMap<String, StaticPathMethods<T>>>,
}

impl<T: Clone> Default for Router<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Router<T> {
    /// Constructs a new `Router`.
    pub fn new() -> Self {
        Self {
            root: RwLock::new(Box::new(Node::new())),
            static_map: RwLock::new(IndexMap::default()),
        }
    }
}
