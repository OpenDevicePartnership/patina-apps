//! Benchmarks for memory operations.
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use mu_rust_helpers::perf_timer::{Arch, ArchFunctionality as _};
use patina::{
    base::UEFI_PAGE_SIZE,
    boot_services::{self, BootServices as _},
    efi_types::EfiMemoryType,
};
use r_efi::efi;
use rolling_stats::Stats;

use crate::{BOOT_SERVICES, error::BenchError};

/// Benchmarks page-level memory allocation.
pub(crate) fn bench_allocate_pages(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        // Use `BOOT_SERVICES_DATA` as it is commonly allocated during boot services/driver initialization.
        let pages = BOOT_SERVICES
            .allocate_pages(boot_services::allocation::AllocType::AnyPage, EfiMemoryType::BootServicesData, 1)
            .map_err(|e| BenchError::BenchTest("Failed to allocate pages", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        BOOT_SERVICES.free_pages(pages, 1).map_err(|e| BenchError::BenchCleanup("Failed to free pages", e))?;
    }
    Ok(stats)
}

/// Benchmarks pool memory allocation.
pub(crate) fn bench_allocate_pool(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        // Use `BOOT_SERVICES_DATA` as it is commonly allocated during boot services/driver initialization.
        let pool = BOOT_SERVICES
            .allocate_pool(EfiMemoryType::BootServicesData, UEFI_PAGE_SIZE / 4)
            .map_err(|e| BenchError::BenchTest("Failed to allocate pool", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        BOOT_SERVICES.free_pool(pool).map_err(|e| BenchError::BenchCleanup("Failed to free pool", e))?;
    }
    Ok(stats)
}

/// Benchmarks page memory deallocation.
pub(crate) fn bench_free_pages(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        // Use `BOOT_SERVICES_DATA` as it is commonly allocated during boot services/driver initialization.
        let pages = BOOT_SERVICES
            .allocate_pages(boot_services::allocation::AllocType::AnyPage, EfiMemoryType::BootServicesData, 1)
            .map_err(|e| BenchError::BenchSetup("Failed to allocate pages", e))?;

        let start = Arch::cpu_count();
        BOOT_SERVICES.free_pages(pages, 1).map_err(|e| BenchError::BenchTest("Failed to free pages", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}

/// Benchmarks pool memory deallocation.
pub(crate) fn bench_free_pool(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        // Use `BOOT_SERVICES_DATA` as it is commonly allocated during boot services/driver initialization.
        let pool = BOOT_SERVICES
            .allocate_pool(EfiMemoryType::BootServicesData, UEFI_PAGE_SIZE / 4)
            .map_err(|e| BenchError::BenchSetup("Failed to allocate pool", e))?;

        let start = Arch::cpu_count();
        BOOT_SERVICES.free_pool(pool).map_err(|e| BenchError::BenchTest("Failed to free pool", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}

/// Benchmarks memory copying performance.
pub(crate) fn bench_copy_mem(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let src: u64 = 5678;
    let mut dst: u64 = 1234;
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        BOOT_SERVICES.copy_mem::<u64>(&mut dst, &src);
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}

/// Benchmarks memory initialization performance.
pub(crate) fn bench_set_mem(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut dst: [u8; 128] = [0; 128];
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        BOOT_SERVICES.set_mem(&mut dst, 1);
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}

/// Benchmarks system memory map retrieval.
pub(crate) fn bench_get_memory_map(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        BOOT_SERVICES.get_memory_map().map_err(|e| BenchError::BenchTest("Failed to get memory map", e.0))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}
