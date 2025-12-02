#[cfg(target_os = "uefi")]
use alloc::{boxed::Box, vec};

#[cfg(not(target_os = "uefi"))]
use std::{boxed::Box, vec};

use mu_rust_helpers::perf_timer::{Arch, ArchFunctionality as _};
use patina::boot_services::BootServices;
use r_efi::efi;
use rolling_stats::Stats;

use crate::{
    BOOT_SERVICES,
    bench::{TestProtocol1, TestProtocol2},
    error::BenchError,
};

/// Benchmarks the UEFI driver model's controller connection mechanism.
pub(crate) fn bench_connect_controller(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    /// Mock driver binding protocols definitions.
    extern "efiapi" fn mock_supported(
        _this: *mut efi::protocols::driver_binding::Protocol,
        _controller_handle: efi::Handle,
        _remaining_device_path: *mut efi::protocols::device_path::Protocol,
    ) -> efi::Status {
        efi::Status::SUCCESS
    }

    extern "efiapi" fn mock_start(
        _this: *mut efi::protocols::driver_binding::Protocol,
        _controller_handle: efi::Handle,
        _remaining_device_path: *mut efi::protocols::device_path::Protocol,
    ) -> efi::Status {
        efi::Status::SUCCESS
    }

    extern "efiapi" fn mock_stop(
        _this: *mut efi::protocols::driver_binding::Protocol,
        _controller_handle: efi::Handle,
        _num_children: usize,
        _child_handle_buffer: *mut efi::Handle,
    ) -> efi::Status {
        efi::Status::SUCCESS
    }

    // Setup controller, driver, and image handles with test protocols.
    let controller_handle = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol interface for controller", e))?
        .0;

    let driver_handle = BOOT_SERVICES
        .install_protocol_interface(
            None,
            Box::new(efi::protocols::device_path::Protocol { r#type: 4, sub_type: 5, length: [0, 0] }),
        )
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol interface for driver", e))?
        .0;

    let image_handle = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol2 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol interface for image", e))?
        .0;

    let binding = Box::new(efi::protocols::driver_binding::Protocol {
        version: 10,
        supported: mock_supported,
        start: mock_start,
        stop: mock_stop,
        driver_binding_handle: driver_handle,
        image_handle,
    });

    let driver_binding_key = BOOT_SERVICES
        .install_protocol_interface(Some(driver_handle), binding)
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol interface for driver binding", e))?
        .1;

    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        // SAFETY: All handles and pointers are valid (constructed by benchmark).
        unsafe {
            BOOT_SERVICES
                .connect_controller(controller_handle, vec![driver_handle], core::ptr::null_mut(), false)
                .map_err(|e| BenchError::BenchTest("Failed to connect controller", e))?;
        }
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
        BOOT_SERVICES
            .disconnect_controller(controller_handle, None, None)
            .map_err(|e| BenchError::BenchCleanup("Failed to disconnect controller", e))?;
    }

    // Uninstall protocols to prevent side effects.
    BOOT_SERVICES
        .uninstall_protocol_interface(driver_handle, driver_binding_key)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol interface", e))?;

    Ok(stats)
}
