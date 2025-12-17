//! Benchmarks for UEFI controller connection mechanism.
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use crate::alloc::{boxed::Box, vec};

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
    let controller_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol interface for controller", e))?;

    let driver_install = BOOT_SERVICES
        .install_protocol_interface(
            None,
            Box::new(efi::protocols::device_path::Protocol { r#type: 4, sub_type: 5, length: [0, 0] }),
        )
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol interface for driver", e))?;

    let image_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol2 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol interface for image", e))?;

    let binding = Box::new(efi::protocols::driver_binding::Protocol {
        version: 10,
        supported: mock_supported,
        start: mock_start,
        stop: mock_stop,
        driver_binding_handle: driver_install.0,
        image_handle: image_install.0,
    });

    let driver_binding = BOOT_SERVICES
        .install_protocol_interface(Some(driver_install.0), binding)
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol interface for driver binding", e))?;

    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        // SAFETY: All handles and pointers are valid (constructed by benchmark).
        unsafe {
            BOOT_SERVICES
                .connect_controller(controller_install.0, vec![driver_install.0], core::ptr::null_mut(), false)
                .map_err(|e| BenchError::BenchTest("Failed to connect controller", e))?;
        }
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
        BOOT_SERVICES
            .disconnect_controller(controller_install.0, None, None)
            .map_err(|e| BenchError::BenchCleanup("Failed to disconnect controller", e))?;
    }

    // Uninstall protocols to prevent side effects.
    BOOT_SERVICES
        .uninstall_protocol_interface(driver_binding.0, driver_binding.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol interface", e))?;
    BOOT_SERVICES
        .uninstall_protocol_interface(driver_install.0, driver_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol interface", e))?;
    BOOT_SERVICES
        .uninstall_protocol_interface(image_install.0, image_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol interface", e))?;
    BOOT_SERVICES
        .uninstall_protocol_interface(controller_install.0, controller_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol interface", e))?;

    Ok(stats)
}
