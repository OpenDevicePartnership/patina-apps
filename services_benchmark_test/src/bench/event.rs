//! Benchmarks for event handling.
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use crate::alloc::vec::Vec;

use core::{ffi::c_void, ptr};

use mu_rust_helpers::perf_timer::{Arch, ArchFunctionality as _};
use patina::boot_services::{BootServices, event::EventType, tpl::Tpl};
use r_efi::efi;
use rolling_stats::Stats;

use crate::{BOOT_SERVICES, error::BenchError};

/// Benchmarks checking the state of an already-signaled event (fast path).
pub(crate) fn bench_check_event_signaled(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    extern "efiapi" fn test_notify(_event: efi::Event, _context: *mut c_void) {}
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let event_handle = BOOT_SERVICES
            .create_event(EventType::NOTIFY_WAIT, Tpl::NOTIFY, Some(test_notify), ptr::null_mut())
            .map_err(|e| BenchError::BenchSetup("Failed to create event", e))?;
        // Signal the event to set it to the signaled state.
        BOOT_SERVICES.signal_event(event_handle).map_err(|e| BenchError::BenchSetup("Failed to signal event", e))?;

        let start = Arch::cpu_count();
        BOOT_SERVICES.check_event(event_handle).map_err(|e| BenchError::BenchTest("check_event failed", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        BOOT_SERVICES.close_event(event_handle).map_err(|e| BenchError::BenchCleanup("Failed to close event", e))?;
    }
    Ok(stats)
}

/// Benchmarks checking the state of an unsignaled event (slow path).
pub(crate) fn bench_check_event_unsignaled(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    extern "efiapi" fn test_notify(_event: efi::Event, _context: *mut c_void) {}
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let event_handle = BOOT_SERVICES
            .create_event(EventType::NOTIFY_WAIT, Tpl::NOTIFY, Some(test_notify), ptr::null_mut())
            .map_err(|e| BenchError::BenchSetup("Failed to create event", e))?;

        let start = Arch::cpu_count();
        if let Err(e) = BOOT_SERVICES.check_event(event_handle) {
            // In this case a NOT_READY error is acceptable since the event is unsignaled.
            if e != efi::Status::SUCCESS && e != efi::Status::NOT_READY {
                return Err(BenchError::BenchTest("check_event returned unexpected status", e));
            }
        }
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        BOOT_SERVICES.close_event(event_handle).map_err(|e| BenchError::BenchCleanup("Failed to close event", e))?;
    }
    Ok(stats)
}

/// Benchmarks event creation performance.
pub(crate) fn bench_create_event(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    extern "efiapi" fn test_notify(_event: efi::Event, _context: *mut c_void) {}
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        let event_handle = BOOT_SERVICES
            .create_event(EventType::NOTIFY_WAIT, Tpl::NOTIFY, Some(test_notify), ptr::null_mut())
            .map_err(|e| BenchError::BenchTest("Failed to create event", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        // Clean up the created event.
        BOOT_SERVICES.close_event(event_handle).map_err(|e| BenchError::BenchCleanup("Failed to close event", e))?;
    }
    Ok(stats)
}

/// Benchmarks event closing performance.
pub(crate) fn bench_close_event(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    extern "efiapi" fn test_notify(_event: efi::Event, _context: *mut c_void) {}
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let event_handle = BOOT_SERVICES
            .create_event(EventType::NOTIFY_WAIT, Tpl::NOTIFY, Some(test_notify), ptr::null_mut())
            .map_err(|e| BenchError::BenchSetup("Failed to create event", e))?;
        let start = Arch::cpu_count();
        BOOT_SERVICES.close_event(event_handle).map_err(|e| BenchError::BenchTest("Failed to close event", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}

/// Benchmarks individual event signaling.
pub(crate) fn bench_signal_event(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    extern "efiapi" fn test_notify(_event: efi::Event, _context: *mut c_void) {}
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let event_handle = BOOT_SERVICES
            .create_event(EventType::NOTIFY_WAIT, Tpl::NOTIFY, Some(test_notify), ptr::null_mut())
            .map_err(|e| BenchError::BenchSetup("Failed to create event", e))?;

        let start = Arch::cpu_count();
        BOOT_SERVICES.signal_event(event_handle).map_err(|e| BenchError::BenchTest("Failed to signal event", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        BOOT_SERVICES.close_event(event_handle).map_err(|e| BenchError::BenchCleanup("Failed to close event", e))?;
    }
    Ok(stats)
}

/// Tests signaling multiple events as a group.
pub(crate) fn bench_signal_event_group(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();

    // No-op notify function. We want to measure only the signaling overhead.
    extern "efiapi" fn test_notify(_event: efi::Event, _context: *mut c_void) {}

    // Use a mock GUID to avoid signalling real event groups.
    const BENCH_EVENT_GROUP: efi::Guid =
        efi::Guid::from_fields(0x12345678, 0x9abc, 0xdef0, 0x12, 0x34, &[0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0]);

    // The event group will increase in size with each iteration to test the impact of group size on signaling time.
    let mut event_grp = Vec::with_capacity(num_calls);
    for _ in 0..num_calls {
        let event_handle = BOOT_SERVICES
            .create_event_ex(
                EventType::NOTIFY_WAIT,
                Tpl::NOTIFY,
                Some(test_notify),
                ptr::null_mut(),
                &BENCH_EVENT_GROUP,
            )
            .map_err(|e| BenchError::BenchSetup("Failed to create event", e))?;
        event_grp.push(event_handle);

        let start = Arch::cpu_count();
        // Signals the most recently created event in the group.
        BOOT_SERVICES.signal_event(event_handle).map_err(|e| BenchError::BenchTest("Failed to signal event", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }

    // Clean up all created events.
    for event_handle in event_grp {
        BOOT_SERVICES.close_event(event_handle).map_err(|e| BenchError::BenchCleanup("Failed to close event", e))?;
    }

    Ok(stats)
}
