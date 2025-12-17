//! Services Benchmark Test Library
//!
//! This crate provides a set of benchmarks for measuring the performance of various UEFI services.
//! It is intended to be run in a UEFI environment to collect timing and call statistics for selected
//! UEFI service functions. The results are output in a markdown-formatted table for easy analysis.
//!
//! ## Usage
//!
//! Invoke the `bench_start` function from your UEFI application or test harness, passing the UEFI
//! image handle and system table. The library will execute a set of predefined benchmarks and print
//! the results to the UEFI console.
//!
//! ## Output
//!
//! The benchmark results include the name of each tested service, total cycles consumed, number of calls,
//! and average cycles per operation.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
#![cfg_attr(target_os = "uefi", no_std)]

/// Global instance of UEFI Boot Services.
pub static BOOT_SERVICES: StandardBootServices = StandardBootServices::new_uninit();

#[cfg(target_os = "uefi")]
extern crate alloc;

#[cfg(not(target_os = "uefi"))]
use std as alloc;

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use core::fmt::Write;
use mu_rust_helpers::perf_timer::{Arch, ArchFunctionality as _};
use rolling_stats::Stats;

use patina::boot_services::StandardBootServices;
use r_efi::efi;

use crate::{error::BenchError, measure::BENCH_FNS};

pub fn bench_start(handle: efi::Handle) -> Result<(), BenchError> {
    log::info!("Starting Services Benchmark Test...");

    let mut output_buf = String::new();

    write_headers(&mut output_buf)?;

    for (bf, num_calls) in BENCH_FNS {
        // Run a few warmup iterations. (10% of the benchmark iterations).
        (bf.func)(handle, num_calls / 10)?;

        let (bench_name, bench_func) = (bf.name, bf.func);
        let cycles_res = bench_func(handle, num_calls);
        match cycles_res {
            Ok(cycles_stats) => {
                // Calculate total time in milliseconds. Formula: ms = cycles / (cycles / s) * 1000.
                let total_time_ms = (cycles_stats.count as f64) / (Arch::perf_frequency() as f64) * 1000.0;
                write_result_row(&mut output_buf, bench_name, cycles_stats, total_time_ms, num_calls)?;
            }
            Err(e) => {
                log::error!("Benchmark {} failed: {:?}", bench_name, e);
                debug_assert!(false);
                // In case of failure write 0s and note failure.
                write_result_row(
                    &mut output_buf,
                    (bench_name.to_string() + " (Failed)").as_str(),
                    Stats::default(),
                    0.0,
                    0,
                )?;
            }
        }
    }

    log::info!("{}", output_buf);
    // SAFETY: `st` is a valid pointer to SystemTable provided by UEFI firmware in `efi_main`.
    unsafe { print_to_console(output_buf.as_str()) };

    Ok(())
}

// Writes the header rows for the fixed-width results markdown table.
pub fn write_headers(output_buf: &mut String) -> Result<(), BenchError> {
    // Column headers.
    writeln!(
        output_buf,
        "| {:<32} | {:>14} | {:>12} | {:>15} | {:>15} | {:>12} | {:>12} | {:>12} |",
        "Name",
        "Total cycles",
        "Total calls",
        "Cycles/op",
        "Total time (ms)",
        "Min cycles",
        "Max cycles",
        "SD [cycles]"
    )
    .map_err(|e| BenchError::WriteOutput("Write table header failed", e))?;
    // Column separators.
    writeln!(
        output_buf,
        "| {:-<32} | {:-<14} | {:-<12} | {:-<15} | {:-<15} | {:-<12} | {:-<12} | {:-<12} |",
        "-", "-", "-", "-", "-", "-", "-", "-"
    )
    .map_err(|e| BenchError::WriteOutput("Write table header failed", e))?;
    Ok(())
}

pub fn write_result_row(
    output_buf: &mut String,
    bench_name: &str,
    stats: Stats<f64>,
    total_time_ms: f64,
    num_calls: usize,
) -> Result<(), BenchError> {
    writeln!(
        output_buf,
        "| {:<32} | {:>14} | {:>12} | {:>15} | {:>15.3} | {:>12} | {:>12} | {:>12.2} |",
        bench_name,
        stats.count, // Format as usize for better readability. Partial cycles don't really matter.
        num_calls,
        stats.mean,
        total_time_ms,
        stats.min,
        stats.max,
        stats.std_dev as usize, // Format as usize for better readability. Partial cycles don't really matter.
    )
    .map_err(|e| BenchError::WriteOutput("Write table header failed", e))?;
    Ok(())
}

/// Print a message to the UEFI console output.
///
/// # Safety
/// The caller must ensure that the UEFI System Table pointer has been initialized.
pub unsafe fn print_to_console(message: &str) {
    let st = uefi::table::system_table_raw();
    if let Some(st_ptr) = st {
        let st = st_ptr.as_ptr();
        // SAFETY: The `uefi` crate guarantees that the System Table pointer is valid after initialization.
        let system_table = unsafe { &*st };
        let con_out = system_table.stdout;

        if con_out.is_null() {
            return;
        }

        // Convert the message to UTF-16 for UEFI console output.
        let mut utf16_buffer: Vec<u16> = message.encode_utf16().collect();
        utf16_buffer.push(0); // Null terminator.

        // Call the UEFI console output function.
        // SAFETY: If the safety conditions are met, the UEFI console output function will be valid.
        let output_string = unsafe { (*con_out).output_string };
        // SAFETY: If the safety conditions are met, the UEFI console output function will be valid.
        let _ = unsafe { output_string(con_out, utf16_buffer.as_ptr() as *mut u16) };
    }
}

mod bench;
mod error;
mod measure;
