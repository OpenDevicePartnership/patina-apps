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
//! Copyright (c) Microsoft Corporation. All rights reserved.
//!
//! SPDX-License-Identifier: BSD-2-Clause-Patent
#![cfg_attr(target_os = "uefi", no_std)]

extern crate alloc;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::Write;
use mu_rust_helpers::perf_timer::{Arch, ArchFunctionality as _};

use patina::boot_services::StandardBootServices;
use r_efi::{efi, system};

use crate::{error::BenchError, measure::BENCH_FNS};
use alloc::string::String;

/// Global instance of UEFI Boot Services.
pub static BOOT_SERVICES: StandardBootServices = StandardBootServices::new_uninit();

pub fn bench_start(handle: efi::Handle, st: *const system::SystemTable) -> Result<(), BenchError> {
    log::info!("Starting Services Benchmark Test...");

    let mut output_buf = String::new();

    // Writes fixed-width markdown table.
    // Column headers.
    writeln!(
        &mut output_buf,
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
    .map_err(|e| BenchError::WriteFailure("Write table header failed", e))?;
    // Column seperators.
    writeln!(
        &mut output_buf,
        "| {:-<32} | {:-<14} | {:-<12} | {:-<15} | {:-<15} | {:-<12} | {:-<12} | {:-<12} |",
        "-", "-", "-", "-", "-", "-", "-", "-"
    )
    .map_err(|e| BenchError::WriteFailure("Write table header failed", e))?;

    for (bf, num_calls) in BENCH_FNS {
        // Run a few warmup iterations. (10% of the benchmark iterations).
        (bf.func)(handle, num_calls / 10)?;

        let (bench_name, bench_func) = (bf.name, bf.func);
        let cycles_res = bench_func(handle, num_calls);
        match cycles_res {
            Ok(cycles) => {
                // Calculate total time in milliseconds. Formula: ms = cycles / (cycles / s) * 1000.
                let total_time_ms =
                    (cycles.count as f64) / (Arch::perf_frequency() as f64) * 1000.0;
                writeln!(
                    &mut output_buf,
                    "| {:<32} | {:>14} | {:>12} | {:>15} | {:>15.3} | {:>12} | {:>12} | {:>12.2} |",
                    bench_name,
                    cycles.count as usize, // Format as usize for better readability. Partial cycles don't really matter.
                    num_calls,
                    cycles.mean,
                    total_time_ms,
                    cycles.min,
                    cycles.max,
                    cycles.std_dev as usize, // Format as usize for better readability. Partial cycles don't really matter.
                )
                .map_err(|e| BenchError::WriteFailure("Write table data failed", e))?;
            }
            Err(e) => {
                log::info!("Benchmark {} failed: {:?}", bench_name, e);
                debug_assert!(false);
                // In case of failure write 0s and note failure.
                writeln!(
                    &mut output_buf,
                    "| {:<32} | {:>14} | {:>12} | {:>15} | {:>15.3} | {:>12} | {:>12} | {:>12.2} |",
                    bench_name.to_string() + " (Failed)",
                    0,
                    0,
                    0,
                    0.0,
                    0,
                    0,
                    0
                )
                .map_err(|e| BenchError::WriteFailure("Write table header failed", e))?;
            }
        }
    }

    log::info!("{}", output_buf);
    // SAFETY: `st` is a valid pointer to SystemTable provided by UEFI firmware in `efi_main`.
    unsafe { print_to_console(&output_buf.as_str()) };

    Ok(())
}

/// Print a message to the UEFI console output.
pub unsafe fn print_to_console(message: &str) {
    let st = uefi::table::system_table_raw();
    if let Some(st_ptr) = st {
        let st = st_ptr.as_ptr();
        let system_table = unsafe { &*st };
        let con_out = system_table.stdout;

        if con_out.is_null() {
            return;
        }

        // Convert the message to UTF-16 for UEFI console output.
        let mut utf16_buffer: Vec<u16> = message.encode_utf16().collect();
        utf16_buffer.push(0); // Null terminator.

        // Call the UEFI console output function.
        let output_string = unsafe { (*con_out).output_string };
        let _ = unsafe { output_string(con_out, utf16_buffer.as_ptr() as *mut u16) };
    }
}

mod bench;
mod error;
mod measure;
