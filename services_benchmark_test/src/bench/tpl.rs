//! Benchmarks for TPL manipulation operations.
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use mu_rust_helpers::perf_timer::{Arch, ArchFunctionality as _};
use patina::boot_services::{BootServices as _, tpl::Tpl};
use r_efi::efi::{self};
use rolling_stats::Stats;

use crate::{BOOT_SERVICES, error::BenchError};

const TPL_HIGH_LEVEL: Tpl = Tpl(31);

/// Benchmarks interrupt disable performance.
pub(crate) fn bench_raise_tpl(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        // Use TPL_HIGH_LEVEL to test impact of interrupts.
        let old_tpl = BOOT_SERVICES.raise_tpl(TPL_HIGH_LEVEL);
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);

        BOOT_SERVICES.restore_tpl(old_tpl);
    }

    Ok(stats)
}

/// Benchmarks interrupt enable performance.
pub(crate) fn bench_restore_tpl(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    let mut stats: Stats<f64> = Stats::new();
    let tpl_options = [Tpl::APPLICATION, Tpl::CALLBACK, Tpl::NOTIFY, TPL_HIGH_LEVEL];
    for i in 0..num_calls {
        // Rotate between different TPL levels to test all scenarios.
        let old_tpl = BOOT_SERVICES.raise_tpl(tpl_options[i % tpl_options.len()]);

        let start = Arch::cpu_count();
        BOOT_SERVICES.restore_tpl(old_tpl);
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }

    Ok(stats)
}
