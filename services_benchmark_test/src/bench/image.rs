//! Benchmarks for image loading and execution.
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use patina::arch;
use patina::uefi::boot_services::BootServices;
use r_efi::efi;
use rolling_stats::Stats;

use crate::{BOOT_SERVICES, error::BenchError};

/// Benchmarks UEFI image execution performance through a no-op image that exits immediately.
///  As `start_image` and `exit` are difficult to bench individually, this benchmark combines them.
pub(crate) fn bench_start_image_and_exit(
    parent_handle: efi::Handle,
    num_calls: usize,
) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        // The image `NoopImage.efi` is a no-op image that exits immediately.
        let image_bytes = include_bytes!("../../resources/NoopImage.efi");
        let loaded_image_handle = BOOT_SERVICES
            .load_image(false, parent_handle, None, Some(image_bytes))
            .map_err(|e| BenchError::BenchSetup("Failed to load image", e))?;

        let start = arch::get_timer_value();
        // This also includes `exit` as the image exits immediately.
        BOOT_SERVICES
            .start_image(loaded_image_handle)
            .map_err(|e| BenchError::BenchTest("Failed to start image", e.0))?;
        let end = arch::get_timer_value();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}

/// Measures UEFI image loading performance using a no-op image.
pub(crate) fn bench_load_image(parent_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let image_bytes = include_bytes!("../../resources/NoopImage.efi");
        let start = arch::get_timer_value();
        let _loaded_image_handle = BOOT_SERVICES
            .load_image(false, parent_handle, None, Some(image_bytes))
            .map_err(|e| BenchError::BenchTest("Failed to load image", e))?;
        let end = arch::get_timer_value();
        stats.update((end - start) as f64);

        // Unload the image to avoid resource leaks.
        BOOT_SERVICES
            .unload_image(_loaded_image_handle)
            .map_err(|e| BenchError::BenchCleanup("Failed to unload image", e))?;
    }
    Ok(stats)
}
