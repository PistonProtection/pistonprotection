//! Geographic and load-balanced routing for origin selection.
//!
//! This module provides GeoDNS-like functionality for selecting the best origin
//! server based on client location, health status, and load balancing algorithms.

pub mod geo;
pub mod load_balancer;
pub mod origin_selector;

pub use geo::{GeoDatabase, GeoLocation, GeoLookupResult};
pub use load_balancer::{LoadBalancer, LoadBalancerAlgorithm};
pub use origin_selector::{OriginSelector, SelectedOrigin};
