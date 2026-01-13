//! Geographic location lookup and database management.
//!
//! Provides GeoIP lookup functionality using MaxMind GeoLite2 databases
//! for determining client location based on IP address.

use std::collections::HashMap;
use std::net::IpAddr;
use std::path::Path;
use std::sync::Arc;

use parking_lot::RwLock;
use tracing::{debug, info};

/// Geographic location information for an IP address.
#[derive(Debug, Clone, Default)]
pub struct GeoLocation {
    /// ISO 3166-1 alpha-2 country code (e.g., "US", "DE", "JP")
    pub country_code: Option<String>,
    /// Continent code (AF, AN, AS, EU, NA, OC, SA)
    pub continent_code: Option<String>,
    /// Region/state code
    pub region_code: Option<String>,
    /// City name
    pub city: Option<String>,
    /// Latitude
    pub latitude: Option<f64>,
    /// Longitude
    pub longitude: Option<f64>,
    /// Autonomous System Number
    pub asn: Option<u32>,
    /// Autonomous System Organization
    pub as_org: Option<String>,
    /// Whether this is a known proxy/VPN/datacenter IP
    pub is_proxy: bool,
    /// Whether this is a known hosting provider
    pub is_hosting: bool,
}

impl GeoLocation {
    /// Calculate the haversine distance to another location in kilometers.
    pub fn distance_to(&self, other: &GeoLocation) -> Option<f64> {
        let lat1 = self.latitude?;
        let lon1 = self.longitude?;
        let lat2 = other.latitude?;
        let lon2 = other.longitude?;

        let r = 6371.0; // Earth's radius in km

        let d_lat = (lat2 - lat1).to_radians();
        let d_lon = (lon2 - lon1).to_radians();

        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + lat1_rad.cos() * lat2_rad.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().asin();

        Some(r * c)
    }

    /// Check if this location is in a specific continent.
    pub fn is_in_continent(&self, continent: &str) -> bool {
        self.continent_code
            .as_ref()
            .is_some_and(|c| c.eq_ignore_ascii_case(continent))
    }

    /// Check if this location is in a specific country.
    pub fn is_in_country(&self, country: &str) -> bool {
        self.country_code
            .as_ref()
            .is_some_and(|c| c.eq_ignore_ascii_case(country))
    }
}

/// Result of a geographic lookup.
#[derive(Debug, Clone)]
pub struct GeoLookupResult {
    /// The IP address that was looked up
    pub ip: IpAddr,
    /// The geographic location (if found)
    pub location: Option<GeoLocation>,
    /// Whether the result came from cache
    pub from_cache: bool,
}

/// Geographic database for IP lookups.
///
/// This implementation uses an in-memory cache with optional MaxMind database
/// support. In production, integrate with maxminddb crate for full functionality.
pub struct GeoDatabase {
    /// Cache of recent lookups
    cache: Arc<RwLock<HashMap<IpAddr, GeoLocation>>>,
    /// Maximum cache size
    max_cache_size: usize,
    /// Static country code mappings (for common ranges)
    static_mappings: HashMap<u32, String>,
    /// Continent mappings for countries
    country_to_continent: HashMap<String, String>,
}

impl GeoDatabase {
    /// Create a new GeoDatabase with default settings.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size: 100_000,
            static_mappings: HashMap::new(),
            country_to_continent: Self::build_continent_map(),
        }
    }

    /// Create a new GeoDatabase from a MaxMind database file.
    pub fn from_file<P: AsRef<Path>>(_path: P) -> Result<Self, GeoError> {
        // In production, use maxminddb crate:
        // let reader = maxminddb::Reader::open_readfile(path)?;
        info!("GeoDatabase initialized (using built-in mappings)");
        Ok(Self::new())
    }

    /// Look up the geographic location for an IP address.
    pub fn lookup(&self, ip: IpAddr) -> GeoLookupResult {
        // Check cache first
        {
            let cache = self.cache.read();
            if let Some(location) = cache.get(&ip) {
                return GeoLookupResult {
                    ip,
                    location: Some(location.clone()),
                    from_cache: true,
                };
            }
        }

        // Perform lookup
        let location = self.do_lookup(ip);

        // Cache the result
        if let Some(ref loc) = location {
            let mut cache = self.cache.write();
            if cache.len() >= self.max_cache_size {
                // Simple eviction: clear half the cache
                let to_remove: Vec<_> = cache.keys().take(self.max_cache_size / 2).cloned().collect();
                for key in to_remove {
                    cache.remove(&key);
                }
            }
            cache.insert(ip, loc.clone());
        }

        GeoLookupResult {
            ip,
            location,
            from_cache: false,
        }
    }

    /// Internal lookup implementation.
    fn do_lookup(&self, ip: IpAddr) -> Option<GeoLocation> {
        match ip {
            IpAddr::V4(ipv4) => {
                let octets = ipv4.octets();

                // Private/reserved ranges
                if octets[0] == 10
                    || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                    || (octets[0] == 192 && octets[1] == 168)
                    || octets[0] == 127
                {
                    return Some(GeoLocation {
                        country_code: Some("XX".to_string()),
                        continent_code: Some("XX".to_string()),
                        ..Default::default()
                    });
                }

                // Use first octet for rough geographic estimation (simplified)
                // In production, use MaxMind database
                let geo = self.estimate_location_v4(octets);
                Some(geo)
            }
            IpAddr::V6(ipv6) => {
                // For IPv6, extract embedded IPv4 if present, otherwise estimate
                if let Some(ipv4) = ipv6.to_ipv4_mapped() {
                    return self.do_lookup(IpAddr::V4(ipv4));
                }

                // Rough estimation based on prefix
                let segments = ipv6.segments();
                let geo = self.estimate_location_v6(segments);
                Some(geo)
            }
        }
    }

    /// Estimate location from IPv4 address (simplified).
    fn estimate_location_v4(&self, octets: [u8; 4]) -> GeoLocation {
        // This is a very simplified estimation based on IANA allocations
        // In production, use MaxMind GeoLite2 database
        let (country, continent) = match octets[0] {
            1..=2 => ("US", "NA"),     // ARIN (mostly US)
            3..=4 => ("US", "NA"),     // GE, Level3
            5..=7 => ("US", "NA"),     // ARIN
            8..=9 => ("US", "NA"),     // Level3
            11 => ("US", "NA"),        // DoD
            12..=15 => ("US", "NA"),   // AT&T
            16..=19 => ("US", "NA"),   // HP, Digital
            20..=23 => ("US", "NA"),   // CSC, MERIT
            24..=30 => ("US", "NA"),   // Various US
            31 => ("GB", "EU"),        // RIPE
            32..=56 => ("US", "NA"),   // Various ARIN
            57..=60 => ("EU", "EU"),   // Various EU
            61..=63 => ("AU", "OC"),   // APNIC (AU region)
            64..=76 => ("US", "NA"),   // ARIN
            77..=79 => ("EU", "EU"),   // RIPE
            80..=95 => ("EU", "EU"),   // RIPE
            96..=99 => ("US", "NA"),   // ARIN
            100..=111 => ("US", "NA"), // ARIN
            112..=126 => ("JP", "AS"), // APNIC (mostly JP/Asia)
            128..=159 => ("US", "NA"), // ARIN (mostly)
            160..=175 => ("US", "NA"), // ARIN
            176..=185 => ("EU", "EU"), // RIPE
            186..=187 => ("BR", "SA"), // LACNIC (Brazil)
            188..=191 => ("EU", "EU"), // RIPE
            192 => ("US", "NA"),       // Various
            193..=195 => ("EU", "EU"), // RIPE
            196 => ("ZA", "AF"),       // AFRINIC (South Africa)
            197 => ("AF", "AF"),       // AFRINIC
            198..=199 => ("US", "NA"), // ARIN
            200..=201 => ("BR", "SA"), // LACNIC
            202..=203 => ("AU", "OC"), // APNIC
            204..=215 => ("US", "NA"), // ARIN
            216..=223 => ("US", "NA"), // ARIN
            _ => ("XX", "XX"),         // Unknown/multicast
        };

        GeoLocation {
            country_code: Some(country.to_string()),
            continent_code: Some(continent.to_string()),
            ..Default::default()
        }
    }

    /// Estimate location from IPv6 address (simplified).
    fn estimate_location_v6(&self, segments: [u16; 8]) -> GeoLocation {
        // Very basic estimation based on first segment
        let (country, continent) = match segments[0] {
            0x2001 => ("US", "NA"),  // Various, including 6to4
            0x2400..=0x24ff => ("JP", "AS"),  // APNIC
            0x2600..=0x26ff => ("US", "NA"),  // ARIN
            0x2800..=0x28ff => ("BR", "SA"),  // LACNIC
            0x2a00..=0x2aff => ("EU", "EU"),  // RIPE
            0x2c00..=0x2cff => ("ZA", "AF"),  // AFRINIC
            _ => ("XX", "XX"),
        };

        GeoLocation {
            country_code: Some(country.to_string()),
            continent_code: Some(continent.to_string()),
            ..Default::default()
        }
    }

    /// Build country to continent mapping.
    fn build_continent_map() -> HashMap<String, String> {
        let mut map = HashMap::new();

        // North America
        for c in ["US", "CA", "MX", "GT", "CU", "HT", "DO", "HN", "NI", "CR", "PA", "JM", "TT", "BS", "BB", "LC", "VC", "GD", "AG", "DM", "KN", "BZ", "SV"] {
            map.insert(c.to_string(), "NA".to_string());
        }

        // South America
        for c in ["BR", "AR", "CL", "CO", "PE", "VE", "EC", "BO", "PY", "UY", "GY", "SR", "GF"] {
            map.insert(c.to_string(), "SA".to_string());
        }

        // Europe
        for c in ["GB", "DE", "FR", "IT", "ES", "PT", "NL", "BE", "AT", "CH", "PL", "CZ", "SK", "HU", "RO", "BG", "GR", "SE", "NO", "DK", "FI", "IE", "LT", "LV", "EE", "HR", "SI", "RS", "UA", "BY", "MD", "AL", "MK", "BA", "ME", "XK", "IS", "LU", "MT", "CY", "MC", "SM", "VA", "AD", "LI"] {
            map.insert(c.to_string(), "EU".to_string());
        }

        // Asia
        for c in ["CN", "JP", "KR", "IN", "ID", "TH", "VN", "PH", "MY", "SG", "TW", "HK", "MO", "BD", "PK", "LK", "NP", "MM", "KH", "LA", "BN", "MN", "KZ", "UZ", "TM", "TJ", "KG", "AF", "IR", "IQ", "SA", "AE", "QA", "KW", "BH", "OM", "YE", "JO", "LB", "SY", "IL", "PS", "TR", "AM", "GE", "AZ"] {
            map.insert(c.to_string(), "AS".to_string());
        }

        // Oceania
        for c in ["AU", "NZ", "FJ", "PG", "SB", "VU", "NC", "PF", "WS", "TO", "FM", "PW", "MH", "NR", "TV", "KI", "GU", "AS", "CK", "NU", "TK", "WF"] {
            map.insert(c.to_string(), "OC".to_string());
        }

        // Africa
        for c in ["ZA", "EG", "NG", "KE", "GH", "TZ", "UG", "ET", "MA", "DZ", "TN", "LY", "SD", "SS", "CD", "AO", "MZ", "ZW", "ZM", "MW", "RW", "BI", "BJ", "TG", "CI", "SN", "ML", "BF", "NE", "TD", "CM", "GA", "CG", "CF", "GQ", "ST", "CV", "MR", "GM", "GW", "SL", "LR", "NA", "BW", "SZ", "LS", "SC", "MU", "MG", "KM", "DJ", "ER", "SO"] {
            map.insert(c.to_string(), "AF".to_string());
        }

        map
    }

    /// Get the continent for a country code.
    pub fn get_continent(&self, country_code: &str) -> Option<&str> {
        self.country_to_continent
            .get(&country_code.to_uppercase())
            .map(|s| s.as_str())
    }

    /// Clear the lookup cache.
    pub fn clear_cache(&self) {
        let mut cache = self.cache.write();
        cache.clear();
        debug!("GeoDatabase cache cleared");
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read();
        (cache.len(), self.max_cache_size)
    }
}

impl Default for GeoDatabase {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during geo operations.
#[derive(Debug, thiserror::Error)]
pub enum GeoError {
    #[error("Database file not found: {0}")]
    DatabaseNotFound(String),
    #[error("Failed to read database: {0}")]
    ReadError(String),
    #[error("Invalid database format: {0}")]
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_geo_lookup_private_ip() {
        let db = GeoDatabase::new();
        let result = db.lookup(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)));
        assert!(result.location.is_some());
        let loc = result.location.unwrap();
        assert_eq!(loc.country_code, Some("XX".to_string()));
    }

    #[test]
    fn test_geo_lookup_public_ip() {
        let db = GeoDatabase::new();
        let result = db.lookup(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)));
        assert!(result.location.is_some());
        let loc = result.location.unwrap();
        assert_eq!(loc.country_code, Some("US".to_string()));
        assert_eq!(loc.continent_code, Some("NA".to_string()));
    }

    #[test]
    fn test_distance_calculation() {
        let london = GeoLocation {
            latitude: Some(51.5074),
            longitude: Some(-0.1278),
            ..Default::default()
        };
        let new_york = GeoLocation {
            latitude: Some(40.7128),
            longitude: Some(-74.0060),
            ..Default::default()
        };

        let distance = london.distance_to(&new_york).unwrap();
        // Should be approximately 5570 km
        assert!(distance > 5500.0 && distance < 5700.0);
    }

    #[test]
    fn test_cache() {
        let db = GeoDatabase::new();
        let ip = IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4));

        let result1 = db.lookup(ip);
        assert!(!result1.from_cache);

        let result2 = db.lookup(ip);
        assert!(result2.from_cache);
    }

    #[test]
    fn test_continent_mapping() {
        let db = GeoDatabase::new();
        assert_eq!(db.get_continent("US"), Some("NA"));
        assert_eq!(db.get_continent("DE"), Some("EU"));
        assert_eq!(db.get_continent("JP"), Some("AS"));
        assert_eq!(db.get_continent("AU"), Some("OC"));
        assert_eq!(db.get_continent("BR"), Some("SA"));
        assert_eq!(db.get_continent("ZA"), Some("AF"));
    }
}
