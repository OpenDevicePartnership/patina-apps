//! Benchmarks for protocol handling.
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use core::ffi::c_void;

use mu_rust_helpers::perf_timer::{Arch, ArchFunctionality as _};
use patina::boot_services::{BootServices, event::EventType, tpl::Tpl};
use r_efi::efi;
use rolling_stats::Stats;

use crate::{
    BOOT_SERVICES,
    bench::{TEST_GUID1, TestProtocol1},
    error::BenchError,
};

use crate::alloc::boxed::Box;

/// Benchmarks protocol installation performance.
pub(crate) fn bench_install_protocol_interface(
    _handle: efi::Handle,
    num_calls: usize,
) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        let protocol_install = BOOT_SERVICES
            .install_protocol_interface(None, Box::new(TestProtocol1 {}))
            .map_err(|e| BenchError::BenchTest("Failed to install protocol", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
        BOOT_SERVICES
            .uninstall_protocol_interface(protocol_install.0, protocol_install.1)
            .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol", e))?;
    }
    Ok(stats)
}

/// Benchmarks protocol opening performance.
/// This is the preferred method (over `handle_protocol`) for retrieving protocol interfaces in modern UEFI (2.0+).
pub(crate) fn bench_open_protocol(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    // Set up and install the protocol to be opened.
    let agent_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install agent protocol", e))?;
    let controller_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install controller protocol", e))?;
    let protocol_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol", e))?;
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        // SAFETY: The resulting interface reference is not used at all during the test.
        (unsafe {
            BOOT_SERVICES
                .open_protocol::<TestProtocol1>(
                    protocol_install.0,
                    agent_install.0,
                    controller_install.0,
                    efi::OPEN_PROTOCOL_BY_DRIVER,
                )
                .map_err(|e| BenchError::BenchTest("Failed to open protocol", e))
        })?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        BOOT_SERVICES
            .close_protocol(protocol_install.0, &TEST_GUID1, agent_install.0, controller_install.0)
            .map_err(|e| BenchError::BenchCleanup("Failed to close protocol", e))?;
    }

    // Uninstall mock protocols after benchmarking.
    BOOT_SERVICES
        .uninstall_protocol_interface(protocol_install.0, protocol_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol", e))?;
    BOOT_SERVICES
        .uninstall_protocol_interface(agent_install.0, agent_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall agent protocol", e))?;
    BOOT_SERVICES
        .uninstall_protocol_interface(controller_install.0, controller_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall controller protocol", e))?;

    Ok(stats)
}

/// Benchmarks protocol closing performance.
pub(crate) fn bench_close_protocol(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    // Set up and install the necessary protocol.
    let agent_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed install agent handle", e))?;
    let controller_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install controller handle.", e))?;
    let protocol_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol handle", e))?;
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        // SAFETY: The resulting interface reference is not used at all during the test.
        unsafe {
            BOOT_SERVICES
                .open_protocol::<TestProtocol1>(
                    protocol_install.0,
                    agent_install.0,
                    controller_install.0,
                    efi::OPEN_PROTOCOL_BY_DRIVER,
                )
                .map_err(|e| BenchError::BenchSetup("Failed to open protocol", e))?;
        }

        let start = Arch::cpu_count();
        BOOT_SERVICES
            .close_protocol(protocol_install.0, &TEST_GUID1, agent_install.0, controller_install.0)
            .map_err(|e| BenchError::BenchTest("Failed to close protocol", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }

    // Uninstall mock protocols after benchmarking.
    BOOT_SERVICES
        .uninstall_protocol_interface(protocol_install.0, protocol_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol", e))?;
    BOOT_SERVICES
        .uninstall_protocol_interface(agent_install.0, agent_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall agent protocol", e))?;
    BOOT_SERVICES
        .uninstall_protocol_interface(controller_install.0, controller_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall controller protocol", e))?;

    Ok(stats)
}

/// Benchmarks protocol handling performance.
/// This is a legacy method but is still included due to needing to support legacy UEFI (1.0).
pub(crate) fn bench_handle_protocol(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    // Set up and install the protocol to be accessed.
    let protocol_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install protocol", e))?;
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        // SAFETY: The resulting interface reference is not used at all during the test.
        (unsafe {
            BOOT_SERVICES
                .handle_protocol::<TestProtocol1>(protocol_install.0)
                .map_err(|e| BenchError::BenchTest("Failed to handle protocol", e))
        })?;

        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    // Uninstall mock protocol after benchmarking.
    BOOT_SERVICES
        .uninstall_protocol_interface(protocol_install.0, protocol_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol", e))?;
    Ok(stats)
}

/// Benchmarks device path resolution.
pub(crate) fn bench_locate_device_path(handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    // Find existing protocol handles to locate device path.
    // SAFETY: There is only one reference to the `loaded_image_protocol` interface.
    let loaded_image_protocol = unsafe {
        BOOT_SERVICES
            .handle_protocol::<efi::protocols::loaded_image::Protocol>(handle)
            .map_err(|e| BenchError::BenchSetup("Failed to get loaded image protocol.", e))?
    };
    // SAFETY: There is only one reference to the `device_path_protocol` interface.
    let device_path_protocol = unsafe {
        BOOT_SERVICES
            .handle_protocol::<efi::protocols::device_path::Protocol>(loaded_image_protocol.device_handle)
            .map_err(|e| BenchError::BenchSetup("Failed to device path protocol.", e))?
    };

    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let mut device_path_ptr = device_path_protocol as *mut efi::protocols::device_path::Protocol;
        let start = Arch::cpu_count();
        // SAFETY: The device path has been constructed above as a valid pointer.
        unsafe {
            BOOT_SERVICES
                .locate_device_path(&efi::protocols::device_path::PROTOCOL_GUID, &mut device_path_ptr as *mut _)
                .map_err(|e| BenchError::BenchTest("Failed to locate device path", e))
        }?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }

    Ok(stats)
}

/// Benchmarks protocol metadata retrieval.
pub(crate) fn bench_open_protocol_information(handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        let _info = BOOT_SERVICES
            .open_protocol_information(handle, &efi::protocols::loaded_image::PROTOCOL_GUID)
            .map_err(|e| BenchError::BenchTest("Failed to get open protocol information", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }

    Ok(stats)
}

/// Benchmarks handle protocol enumeration.
pub(crate) fn bench_protocols_per_handle(handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        let _protocols = BOOT_SERVICES
            .protocols_per_handle(handle)
            .map_err(|e| BenchError::BenchTest("Failed to get protocols per handle", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }

    Ok(stats)
}

/// Benchmarks protocol notification registration.
pub(crate) fn bench_register_protocol_notify(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    // Mock notify does nothing.
    extern "efiapi" fn mock_notify(_ptr: *mut c_void, _data: *mut i32) {}

    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let event = BOOT_SERVICES
            .create_event(EventType::NOTIFY_SIGNAL, Tpl::NOTIFY, Some(mock_notify), &mut 0 as *mut i32)
            .map_err(|e| BenchError::BenchSetup("Failed to create valid event", e))?;
        let start = Arch::cpu_count();
        BOOT_SERVICES
            .register_protocol_notify(&efi::protocols::loaded_image::PROTOCOL_GUID, event)
            .map_err(|e| BenchError::BenchTest("Failed to register protocol notify", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        BOOT_SERVICES.close_event(event).map_err(|e| BenchError::BenchCleanup("Failed to close event", e))?;
    }

    Ok(stats)
}

/// Benchmarks protocol update performance.
pub(crate) fn bench_reinstall_protocol_interface(
    _handle: efi::Handle,
    num_calls: usize,
) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let prev_interface = Box::new(TestProtocol1 {});
        let new_interface = Box::new(TestProtocol1 {});
        let protocol_install = BOOT_SERVICES
            .install_protocol_interface(None, prev_interface)
            .map_err(|e| BenchError::BenchSetup("Failed to install dummy protocol", e))?;

        let start = Arch::cpu_count();
        let reinstall = BOOT_SERVICES
            .reinstall_protocol_interface(protocol_install.0, protocol_install.1, new_interface)
            .map_err(|e| BenchError::BenchTest("Failed to reinstall protocol interface", e))?;

        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        // Cleanup: Uninstall the protocol after benchmarking. (It will be installed and reinstalled in the next iteration.)
        BOOT_SERVICES
            .uninstall_protocol_interface(protocol_install.0, reinstall.0)
            .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol interface", e))?;
    }

    Ok(stats)
}

/// Benchmarks protocol removal performance.
pub(crate) fn bench_uninstall_protocol_interface(
    _handle: efi::Handle,
    num_calls: usize,
) -> Result<Stats<f64>, BenchError> {
    let mut protocol_install = BOOT_SERVICES
        .install_protocol_interface(None, Box::new(TestProtocol1 {}))
        .map_err(|e| BenchError::BenchSetup("Failed to install dummy protocol", e))?;
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        BOOT_SERVICES
            .uninstall_protocol_interface(protocol_install.0, protocol_install.1)
            .map_err(|e| BenchError::BenchTest("Failed to uninstall protocol interface", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        // Reinstall for next iteration.
        protocol_install = BOOT_SERVICES
            .install_protocol_interface(None, Box::new(TestProtocol1 {}))
            .map_err(|e| BenchError::BenchCleanup("Failed to install a new dummy protocol", e))?;
    }

    // Installation from last iteration cleanup.
    BOOT_SERVICES
        .uninstall_protocol_interface(protocol_install.0, protocol_install.1)
        .map_err(|e| BenchError::BenchCleanup("Failed to uninstall protocol interface", e))?;

    Ok(stats)
}
