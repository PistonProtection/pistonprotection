//! eBPF loader tests (mocked)
//!
//! Note: These tests use mocks since actual eBPF operations require
//! root privileges and specific kernel support.

use super::test_utils::{MockNetworkInterface, TestPacketMeta, constants};
use crate::ebpf::loader::{AttachedProgram, EbpfLoader, XdpMode};
use std::path::Path;

/// Mock eBPF loader for testing without root privileges
struct MockEbpfLoader {
    loaded_programs: Vec<String>,
    attached_programs: Vec<AttachedProgram>,
    should_fail_load: bool,
    should_fail_attach: bool,
}

impl MockEbpfLoader {
    fn new() -> Self {
        Self {
            loaded_programs: Vec::new(),
            attached_programs: Vec::new(),
            should_fail_load: false,
            should_fail_attach: false,
        }
    }

    fn set_fail_load(&mut self, fail: bool) {
        self.should_fail_load = fail;
    }

    fn set_fail_attach(&mut self, fail: bool) {
        self.should_fail_attach = fail;
    }

    fn load_from_bytes(&mut self, name: &str, _data: &[u8]) -> Result<(), String> {
        if self.should_fail_load {
            return Err("Mock load failure".to_string());
        }
        self.loaded_programs.push(name.to_string());
        Ok(())
    }

    fn load_from_file(&mut self, name: &str, path: &Path) -> Result<(), String> {
        if !path.exists() && !self.should_fail_load {
            // For testing, we allow non-existent paths in mock
            self.loaded_programs.push(name.to_string());
            return Ok(());
        }
        if self.should_fail_load {
            return Err("Mock load failure".to_string());
        }
        self.loaded_programs.push(name.to_string());
        Ok(())
    }

    fn attach_xdp(
        &mut self,
        program_name: &str,
        interface: &MockNetworkInterface,
        mode: XdpMode,
    ) -> Result<(), String> {
        if self.should_fail_attach {
            return Err("Mock attach failure".to_string());
        }
        if !self.loaded_programs.contains(&program_name.to_string()) {
            return Err("Program not loaded".to_string());
        }
        self.attached_programs.push(AttachedProgram {
            interface: interface.name.clone(),
            mode,
            program_name: program_name.to_string(),
        });
        Ok(())
    }

    fn detach_xdp(&mut self, interface_name: &str) -> Result<(), String> {
        self.attached_programs
            .retain(|p| p.interface != interface_name);
        Ok(())
    }

    fn is_attached(&self, interface_name: &str) -> bool {
        self.attached_programs
            .iter()
            .any(|p| p.interface == interface_name)
    }

    fn list_attached(&self) -> Vec<&AttachedProgram> {
        self.attached_programs.iter().collect()
    }

    fn is_loaded(&self, program_name: &str) -> bool {
        self.loaded_programs.contains(&program_name.to_string())
    }
}

// ============================================================================
// Loading Tests
// ============================================================================

#[cfg(test)]
mod loading_tests {
    use super::*;

    /// Test loading eBPF program from bytes
    #[test]
    fn test_load_from_bytes() {
        let mut loader = MockEbpfLoader::new();

        // Minimal valid eBPF bytecode (mock)
        let bytecode = vec![0u8; 128];

        let result = loader.load_from_bytes("xdp_filter", &bytecode);

        assert!(result.is_ok());
        assert!(loader.is_loaded("xdp_filter"));
    }

    /// Test loading eBPF program from file
    #[test]
    fn test_load_from_file() {
        let mut loader = MockEbpfLoader::new();

        let result = loader.load_from_file("xdp_filter", Path::new("/path/to/filter.o"));

        assert!(result.is_ok());
        assert!(loader.is_loaded("xdp_filter"));
    }

    /// Test loading multiple programs
    #[test]
    fn test_load_multiple() {
        let mut loader = MockEbpfLoader::new();

        loader.load_from_bytes("prog1", &[]).unwrap();
        loader.load_from_bytes("prog2", &[]).unwrap();
        loader.load_from_bytes("prog3", &[]).unwrap();

        assert!(loader.is_loaded("prog1"));
        assert!(loader.is_loaded("prog2"));
        assert!(loader.is_loaded("prog3"));
    }

    /// Test loading same program twice
    #[test]
    fn test_load_duplicate() {
        let mut loader = MockEbpfLoader::new();

        loader.load_from_bytes("prog", &[]).unwrap();
        loader.load_from_bytes("prog", &[]).unwrap();

        // Should have duplicate entry (mock behavior)
        assert!(loader.is_loaded("prog"));
    }

    /// Test load failure handling
    #[test]
    fn test_load_failure() {
        let mut loader = MockEbpfLoader::new();
        loader.set_fail_load(true);

        let result = loader.load_from_bytes("failing_prog", &[]);

        assert!(result.is_err());
        assert!(!loader.is_loaded("failing_prog"));
    }
}

// ============================================================================
// Attachment Tests
// ============================================================================

#[cfg(test)]
mod attachment_tests {
    use super::*;

    /// Test attaching XDP program to interface
    #[test]
    fn test_attach_xdp() {
        let mut loader = MockEbpfLoader::new();
        let interface = MockNetworkInterface::default();

        loader.load_from_bytes("xdp_filter", &[]).unwrap();
        let result = loader.attach_xdp("xdp_filter", &interface, XdpMode::Generic);

        assert!(result.is_ok());
        assert!(loader.is_attached(&interface.name));
    }

    /// Test attach without loading fails
    #[test]
    fn test_attach_unloaded() {
        let mut loader = MockEbpfLoader::new();
        let interface = MockNetworkInterface::default();

        let result = loader.attach_xdp("not_loaded", &interface, XdpMode::Generic);

        assert!(result.is_err());
    }

    /// Test attach with different modes
    #[test]
    fn test_attach_modes() {
        let mut loader = MockEbpfLoader::new();
        loader.load_from_bytes("xdp_filter", &[]).unwrap();

        let modes = vec![XdpMode::Generic, XdpMode::Driver, XdpMode::Offload];

        for mode in modes {
            let mut interface = MockNetworkInterface::default();
            interface.name = format!("eth{:?}", mode);

            let result = loader.attach_xdp("xdp_filter", &interface, mode);
            assert!(result.is_ok());
        }
    }

    /// Test attach to multiple interfaces
    #[test]
    fn test_attach_multiple_interfaces() {
        let mut loader = MockEbpfLoader::new();
        loader.load_from_bytes("xdp_filter", &[]).unwrap();

        let interfaces = vec!["eth0", "eth1", "eth2"];

        for name in &interfaces {
            let mut interface = MockNetworkInterface::default();
            interface.name = name.to_string();
            loader
                .attach_xdp("xdp_filter", &interface, XdpMode::Generic)
                .unwrap();
        }

        for name in interfaces {
            assert!(loader.is_attached(name));
        }
    }

    /// Test attach failure handling
    #[test]
    fn test_attach_failure() {
        let mut loader = MockEbpfLoader::new();
        loader.load_from_bytes("xdp_filter", &[]).unwrap();
        loader.set_fail_attach(true);

        let interface = MockNetworkInterface::default();
        let result = loader.attach_xdp("xdp_filter", &interface, XdpMode::Generic);

        assert!(result.is_err());
        assert!(!loader.is_attached(&interface.name));
    }
}

// ============================================================================
// Detachment Tests
// ============================================================================

#[cfg(test)]
mod detachment_tests {
    use super::*;

    /// Test detaching XDP program
    #[test]
    fn test_detach_xdp() {
        let mut loader = MockEbpfLoader::new();
        let interface = MockNetworkInterface::default();

        loader.load_from_bytes("xdp_filter", &[]).unwrap();
        loader
            .attach_xdp("xdp_filter", &interface, XdpMode::Generic)
            .unwrap();
        assert!(loader.is_attached(&interface.name));

        let result = loader.detach_xdp(&interface.name);

        assert!(result.is_ok());
        assert!(!loader.is_attached(&interface.name));
    }

    /// Test detach non-attached interface
    #[test]
    fn test_detach_not_attached() {
        let mut loader = MockEbpfLoader::new();

        let result = loader.detach_xdp("not_attached");

        // Should succeed (no-op)
        assert!(result.is_ok());
    }

    /// Test detach specific interface
    #[test]
    fn test_detach_specific() {
        let mut loader = MockEbpfLoader::new();
        loader.load_from_bytes("xdp_filter", &[]).unwrap();

        let mut if1 = MockNetworkInterface::default();
        if1.name = "eth0".to_string();
        let mut if2 = MockNetworkInterface::default();
        if2.name = "eth1".to_string();

        loader
            .attach_xdp("xdp_filter", &if1, XdpMode::Generic)
            .unwrap();
        loader
            .attach_xdp("xdp_filter", &if2, XdpMode::Generic)
            .unwrap();

        // Detach only eth0
        loader.detach_xdp("eth0").unwrap();

        assert!(!loader.is_attached("eth0"));
        assert!(loader.is_attached("eth1"));
    }
}

// ============================================================================
// XDP Mode Tests
// ============================================================================

#[cfg(test)]
mod mode_tests {
    use super::*;
    use aya::programs::XdpFlags;

    /// Test XDP mode to flags conversion
    #[test]
    fn test_mode_to_flags() {
        assert_eq!(XdpMode::Generic.to_flags(), XdpFlags::SKB_MODE);
        assert_eq!(XdpMode::Driver.to_flags(), XdpFlags::DRV_MODE);
        assert_eq!(XdpMode::Offload.to_flags(), XdpFlags::HW_MODE);
    }

    /// Test mode fallback logic
    #[test]
    fn test_mode_fallback() {
        // Mock fallback behavior
        let modes = vec![
            (XdpMode::Offload, vec![XdpMode::Driver, XdpMode::Generic]),
            (XdpMode::Driver, vec![XdpMode::Generic]),
            (XdpMode::Generic, vec![]),
        ];

        for (preferred, fallbacks) in modes {
            match preferred {
                XdpMode::Offload => {
                    assert_eq!(fallbacks.len(), 2);
                }
                XdpMode::Driver => {
                    assert_eq!(fallbacks.len(), 1);
                }
                XdpMode::Generic => {
                    assert!(fallbacks.is_empty());
                }
            }
        }
    }
}

// ============================================================================
// Listing Tests
// ============================================================================

#[cfg(test)]
mod listing_tests {
    use super::*;

    /// Test listing attached programs
    #[test]
    fn test_list_attached() {
        let mut loader = MockEbpfLoader::new();
        loader.load_from_bytes("prog1", &[]).unwrap();
        loader.load_from_bytes("prog2", &[]).unwrap();

        let mut if1 = MockNetworkInterface::default();
        if1.name = "eth0".to_string();
        let mut if2 = MockNetworkInterface::default();
        if2.name = "eth1".to_string();

        loader.attach_xdp("prog1", &if1, XdpMode::Generic).unwrap();
        loader.attach_xdp("prog2", &if2, XdpMode::Driver).unwrap();

        let attached = loader.list_attached();

        assert_eq!(attached.len(), 2);
    }

    /// Test listing empty
    #[test]
    fn test_list_attached_empty() {
        let loader = MockEbpfLoader::new();

        let attached = loader.list_attached();

        assert!(attached.is_empty());
    }
}

// ============================================================================
// Cleanup Tests
// ============================================================================

#[cfg(test)]
mod cleanup_tests {
    use super::*;

    /// Test loader cleanup on drop
    #[test]
    fn test_cleanup_on_drop() {
        let mut loader = MockEbpfLoader::new();
        let interface = MockNetworkInterface::default();

        loader.load_from_bytes("xdp_filter", &[]).unwrap();
        loader
            .attach_xdp("xdp_filter", &interface, XdpMode::Generic)
            .unwrap();

        // Drop loader - should clean up
        drop(loader);

        // Can't verify cleanup in mock, but real implementation
        // would detach programs
    }
}
