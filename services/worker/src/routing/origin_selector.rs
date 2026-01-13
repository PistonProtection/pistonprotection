//! Origin selection combining geographic routing and load balancing.
//!
//! This module provides the main interface for selecting the best origin
//! server based on client location, load balancing algorithms, and health status.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{debug, trace, warn};

use super::geo::{GeoDatabase, GeoLocation};
use super::load_balancer::{LoadBalancer, LoadBalancerAlgorithm, OriginInfo};

/// Strategy for geographic routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum GeoRoutingStrategy {
    /// Disabled - use load balancer only
    #[default]
    Disabled,
    /// Route to geographically closest origin
    Proximity,
    /// Route based on explicit region-to-origin mappings
    Mapping,
    /// Route based on measured latency
    Latency,
    /// Route to origin in same continent
    Continent,
}

/// Configuration for an origin's geographic preferences.
#[derive(Debug, Clone)]
pub struct OriginGeoConfig {
    /// Origin ID
    pub origin_id: String,
    /// Geographic location of this origin
    pub location: GeoLocation,
    /// Countries this origin should serve (ISO codes)
    pub preferred_countries: Vec<String>,
    /// Continents this origin should serve
    pub preferred_continents: Vec<String>,
    /// Priority for geo routing (lower = higher priority)
    pub geo_priority: u32,
}

/// Mapping from a region to preferred origins.
#[derive(Debug, Clone)]
pub struct RegionMapping {
    /// Region identifier (country code, continent code, or custom)
    pub region: String,
    /// Whether this is a country, continent, or custom region
    pub region_type: RegionType,
    /// Ordered list of origin IDs to try
    pub origin_ids: Vec<String>,
}

/// Type of region.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionType {
    Country,
    Continent,
    Custom,
}

/// Selected origin with routing information.
#[derive(Debug, Clone)]
pub struct SelectedOrigin {
    /// Selected origin ID
    pub origin_id: String,
    /// How the origin was selected
    pub selection_reason: SelectionReason,
    /// Client's geographic location (if available)
    pub client_location: Option<GeoLocation>,
    /// Distance to origin (if calculated)
    pub distance_km: Option<f64>,
}

/// Reason for origin selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionReason {
    /// Selected via geo routing (closest)
    GeoProximity,
    /// Selected via continent matching
    GeoContinent,
    /// Selected via explicit region mapping
    GeoMapping,
    /// Selected via latency-based routing
    GeoLatency,
    /// Selected via load balancer
    LoadBalancer,
    /// Fallback selection
    Fallback,
    /// Only one origin available
    SingleOrigin,
}

/// Origin selector combining geo routing and load balancing.
pub struct OriginSelector {
    /// Backend ID this selector is for
    backend_id: String,
    /// Geographic database for IP lookups
    geo_db: Arc<GeoDatabase>,
    /// Load balancer
    load_balancer: LoadBalancer,
    /// Geographic routing strategy
    geo_strategy: GeoRoutingStrategy,
    /// Per-origin geographic configuration
    origin_geo_configs: Arc<RwLock<HashMap<String, OriginGeoConfig>>>,
    /// Region to origin mappings
    region_mappings: Arc<RwLock<Vec<RegionMapping>>>,
    /// Fallback origin ID
    fallback_origin_id: Option<String>,
}

impl OriginSelector {
    /// Create a new origin selector.
    pub fn new(backend_id: impl Into<String>, geo_db: Arc<GeoDatabase>) -> Self {
        Self {
            backend_id: backend_id.into(),
            geo_db,
            load_balancer: LoadBalancer::new(LoadBalancerAlgorithm::default()),
            geo_strategy: GeoRoutingStrategy::default(),
            origin_geo_configs: Arc::new(RwLock::new(HashMap::new())),
            region_mappings: Arc::new(RwLock::new(Vec::new())),
            fallback_origin_id: None,
        }
    }

    /// Configure the load balancing algorithm.
    pub fn set_load_balancer_algorithm(&mut self, algorithm: LoadBalancerAlgorithm) {
        self.load_balancer = LoadBalancer::new(algorithm);
    }

    /// Configure the geographic routing strategy.
    pub fn set_geo_strategy(&mut self, strategy: GeoRoutingStrategy) {
        self.geo_strategy = strategy;
    }

    /// Set the fallback origin ID.
    pub fn set_fallback_origin(&mut self, origin_id: Option<String>) {
        self.fallback_origin_id = origin_id;
    }

    /// Update the list of available origins.
    pub fn update_origins(&self, origins: Vec<OriginInfo>) {
        self.load_balancer.update_origins(origins);
    }

    /// Update geographic configuration for an origin.
    pub fn update_origin_geo_config(&self, config: OriginGeoConfig) {
        let mut configs = self.origin_geo_configs.write();
        configs.insert(config.origin_id.clone(), config);
    }

    /// Update region mappings.
    pub fn update_region_mappings(&self, mappings: Vec<RegionMapping>) {
        let mut region_mappings = self.region_mappings.write();
        *region_mappings = mappings;
    }

    /// Update the health status of an origin.
    pub fn update_origin_health(&self, origin_id: &str, healthy: bool) {
        self.load_balancer.update_origin_health(origin_id, healthy);
    }

    /// Select the best origin for a client.
    pub fn select(&self, client_ip: IpAddr) -> Option<SelectedOrigin> {
        // Look up client location
        let geo_result = self.geo_db.lookup(client_ip);
        let client_location = geo_result.location.clone();

        trace!(
            backend = %self.backend_id,
            client_ip = %client_ip,
            country = ?client_location.as_ref().and_then(|l| l.country_code.clone()),
            "Selecting origin for client"
        );

        // Get all available origins
        let origins = self.load_balancer.get_origins();
        if origins.is_empty() {
            warn!(backend = %self.backend_id, "No origins available");
            return None;
        }

        // If only one origin, use it
        if origins.len() == 1 {
            return Some(SelectedOrigin {
                origin_id: origins[0].id.clone(),
                selection_reason: SelectionReason::SingleOrigin,
                client_location,
                distance_km: None,
            });
        }

        // Try geographic routing first
        if self.geo_strategy != GeoRoutingStrategy::Disabled {
            if let Some(selected) = self.select_geo(client_ip, &client_location, &origins) {
                return Some(selected);
            }
        }

        // Fall back to load balancer
        self.load_balancer.select(Some(client_ip)).map(|origin_id| {
            SelectedOrigin {
                origin_id,
                selection_reason: SelectionReason::LoadBalancer,
                client_location,
                distance_km: None,
            }
        })
    }

    /// Select origin using geographic routing.
    fn select_geo(
        &self,
        _client_ip: IpAddr,
        client_location: &Option<GeoLocation>,
        origins: &[OriginInfo],
    ) -> Option<SelectedOrigin> {
        let client_loc = client_location.as_ref()?;

        match self.geo_strategy {
            GeoRoutingStrategy::Disabled => None,
            GeoRoutingStrategy::Proximity => self.select_geo_proximity(client_loc, origins),
            GeoRoutingStrategy::Mapping => self.select_geo_mapping(client_loc, origins),
            GeoRoutingStrategy::Continent => self.select_geo_continent(client_loc, origins),
            GeoRoutingStrategy::Latency => {
                // Latency-based requires active probing, fall back to proximity
                self.select_geo_proximity(client_loc, origins)
            }
        }
    }

    /// Select origin based on geographic proximity.
    fn select_geo_proximity(
        &self,
        client_loc: &GeoLocation,
        origins: &[OriginInfo],
    ) -> Option<SelectedOrigin> {
        let configs = self.origin_geo_configs.read();

        // Calculate distance to each origin
        let mut distances: Vec<(&OriginInfo, Option<f64>)> = origins
            .iter()
            .filter(|o| o.enabled && o.healthy)
            .map(|origin| {
                let distance = configs
                    .get(&origin.id)
                    .and_then(|config| client_loc.distance_to(&config.location));
                (origin, distance)
            })
            .collect();

        // Sort by distance (None = unknown = last)
        distances.sort_by(|(_, d1), (_, d2)| {
            match (d1, d2) {
                (Some(a), Some(b)) => a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => std::cmp::Ordering::Equal,
            }
        });

        distances.first().map(|(origin, distance)| {
            debug!(
                backend = %self.backend_id,
                origin = %origin.id,
                distance_km = ?distance,
                "Selected origin by proximity"
            );
            SelectedOrigin {
                origin_id: origin.id.clone(),
                selection_reason: SelectionReason::GeoProximity,
                client_location: Some(client_loc.clone()),
                distance_km: *distance,
            }
        })
    }

    /// Select origin based on continent matching.
    fn select_geo_continent(
        &self,
        client_loc: &GeoLocation,
        origins: &[OriginInfo],
    ) -> Option<SelectedOrigin> {
        let client_continent = client_loc.continent_code.as_ref()?;
        let configs = self.origin_geo_configs.read();

        // Find origins in the same continent
        let mut matching: Vec<&OriginInfo> = origins
            .iter()
            .filter(|o| o.enabled && o.healthy)
            .filter(|o| {
                configs
                    .get(&o.id)
                    .map(|c| {
                        c.preferred_continents.contains(client_continent)
                            || c.location
                                .continent_code
                                .as_ref()
                                .is_some_and(|cc| cc == client_continent)
                    })
                    .unwrap_or(false)
            })
            .collect();

        // Sort by geo priority
        matching.sort_by_key(|o| configs.get(&o.id).map(|c| c.geo_priority).unwrap_or(u32::MAX));

        matching.first().map(|origin| {
            debug!(
                backend = %self.backend_id,
                origin = %origin.id,
                continent = %client_continent,
                "Selected origin by continent"
            );
            SelectedOrigin {
                origin_id: origin.id.clone(),
                selection_reason: SelectionReason::GeoContinent,
                client_location: Some(client_loc.clone()),
                distance_km: None,
            }
        })
    }

    /// Select origin based on explicit region mappings.
    fn select_geo_mapping(
        &self,
        client_loc: &GeoLocation,
        origins: &[OriginInfo],
    ) -> Option<SelectedOrigin> {
        let mappings = self.region_mappings.read();
        let healthy_origins: std::collections::HashSet<_> = origins
            .iter()
            .filter(|o| o.enabled && o.healthy)
            .map(|o| o.id.as_str())
            .collect();

        // Try country mapping first
        if let Some(country) = &client_loc.country_code {
            for mapping in mappings.iter() {
                if mapping.region_type == RegionType::Country
                    && mapping.region.eq_ignore_ascii_case(country)
                {
                    // Find first healthy origin from the mapping
                    for origin_id in &mapping.origin_ids {
                        if healthy_origins.contains(origin_id.as_str()) {
                            debug!(
                                backend = %self.backend_id,
                                origin = %origin_id,
                                country = %country,
                                "Selected origin by country mapping"
                            );
                            return Some(SelectedOrigin {
                                origin_id: origin_id.clone(),
                                selection_reason: SelectionReason::GeoMapping,
                                client_location: Some(client_loc.clone()),
                                distance_km: None,
                            });
                        }
                    }
                }
            }
        }

        // Try continent mapping
        if let Some(continent) = &client_loc.continent_code {
            for mapping in mappings.iter() {
                if mapping.region_type == RegionType::Continent
                    && mapping.region.eq_ignore_ascii_case(continent)
                {
                    for origin_id in &mapping.origin_ids {
                        if healthy_origins.contains(origin_id.as_str()) {
                            debug!(
                                backend = %self.backend_id,
                                origin = %origin_id,
                                continent = %continent,
                                "Selected origin by continent mapping"
                            );
                            return Some(SelectedOrigin {
                                origin_id: origin_id.clone(),
                                selection_reason: SelectionReason::GeoMapping,
                                client_location: Some(client_loc.clone()),
                                distance_km: None,
                            });
                        }
                    }
                }
            }
        }

        None
    }

    /// Get the backend ID.
    pub fn backend_id(&self) -> &str {
        &self.backend_id
    }

    /// Get current strategy.
    pub fn geo_strategy(&self) -> GeoRoutingStrategy {
        self.geo_strategy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn create_selector() -> OriginSelector {
        let geo_db = Arc::new(GeoDatabase::new());
        OriginSelector::new("test-backend", geo_db)
    }

    #[test]
    fn test_single_origin() {
        let selector = create_selector();
        selector.update_origins(vec![OriginInfo::new("origin-1")]);

        let result = selector.select(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.origin_id, "origin-1");
        assert_eq!(selected.selection_reason, SelectionReason::SingleOrigin);
    }

    #[test]
    fn test_load_balancer_fallback() {
        let selector = create_selector();
        selector.update_origins(vec![
            OriginInfo::new("origin-1"),
            OriginInfo::new("origin-2"),
        ]);

        let result = selector.select(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.selection_reason, SelectionReason::LoadBalancer);
    }

    #[test]
    fn test_geo_continent_routing() {
        let geo_db = Arc::new(GeoDatabase::new());
        let mut selector = OriginSelector::new("test-backend", geo_db);
        selector.set_geo_strategy(GeoRoutingStrategy::Continent);

        selector.update_origins(vec![
            OriginInfo::new("us-origin"),
            OriginInfo::new("eu-origin"),
        ]);

        // Configure US origin for NA continent
        selector.update_origin_geo_config(OriginGeoConfig {
            origin_id: "us-origin".to_string(),
            location: GeoLocation {
                country_code: Some("US".to_string()),
                continent_code: Some("NA".to_string()),
                latitude: Some(37.7749),
                longitude: Some(-122.4194),
                ..Default::default()
            },
            preferred_countries: vec!["US".to_string(), "CA".to_string()],
            preferred_continents: vec!["NA".to_string()],
            geo_priority: 0,
        });

        // Configure EU origin for EU continent
        selector.update_origin_geo_config(OriginGeoConfig {
            origin_id: "eu-origin".to_string(),
            location: GeoLocation {
                country_code: Some("DE".to_string()),
                continent_code: Some("EU".to_string()),
                latitude: Some(52.5200),
                longitude: Some(13.4050),
                ..Default::default()
            },
            preferred_countries: vec!["DE".to_string(), "FR".to_string()],
            preferred_continents: vec!["EU".to_string()],
            geo_priority: 0,
        });

        // US IP should get US origin
        let result = selector.select(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.origin_id, "us-origin");
        assert_eq!(selected.selection_reason, SelectionReason::GeoContinent);
    }

    #[test]
    fn test_region_mapping() {
        let geo_db = Arc::new(GeoDatabase::new());
        let mut selector = OriginSelector::new("test-backend", geo_db);
        selector.set_geo_strategy(GeoRoutingStrategy::Mapping);

        selector.update_origins(vec![
            OriginInfo::new("origin-1"),
            OriginInfo::new("origin-2"),
            OriginInfo::new("origin-3"),
        ]);

        // Map US to origin-1
        selector.update_region_mappings(vec![
            RegionMapping {
                region: "US".to_string(),
                region_type: RegionType::Country,
                origin_ids: vec!["origin-1".to_string()],
            },
            RegionMapping {
                region: "NA".to_string(),
                region_type: RegionType::Continent,
                origin_ids: vec!["origin-2".to_string()],
            },
        ]);

        // US IP should match country mapping first
        let result = selector.select(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(result.is_some());
        let selected = result.unwrap();
        assert_eq!(selected.origin_id, "origin-1");
        assert_eq!(selected.selection_reason, SelectionReason::GeoMapping);
    }

    #[test]
    fn test_no_origins() {
        let selector = create_selector();
        let result = selector.select(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(result.is_none());
    }
}
