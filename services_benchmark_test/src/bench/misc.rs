use core::ffi::c_void;

use mu_rust_helpers::perf_timer::{Arch, ArchFunctionality as _};
use patina::boot_services::BootServices as _;
use r_efi::efi;
use rolling_stats::Stats;

use crate::{BOOT_SERVICES, bench::TEST_GUID1, error::BenchError};

/// Benchmarks checksum calculation performance.
pub(crate) fn bench_calculate_crc32(_handle: efi::Handle, num_calls: usize) -> Result<Stats<f64>, BenchError> {
    // The actual data values do not affect CRC32 performance, so use all zeroes for simplicity.
    let data: [u8; 128] = [0; 128];
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        let _crc =
            BOOT_SERVICES.calculate_crc_32(&data).map_err(|e| BenchError::BenchTest("Failed to calculate CRC32", e))?;
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}

/// Benchmarks installation of a configuration table.
pub(crate) fn bench_install_configuration_table(
    _handle: efi::Handle,
    num_calls: usize,
) -> Result<Stats<f64>, BenchError> {
    let table: u64 = 0xDEADBEEF;
    let mut stats: Stats<f64> = Stats::new();
    for _ in 0..num_calls {
        let start = Arch::cpu_count();
        unsafe {
            // We do not need to clean up the installed table on each iteration as
            // installing a table with a duplicate GUID simply overwrites the previous entry.
            BOOT_SERVICES
                .install_configuration_table(&TEST_GUID1, &table as *const u64 as *mut c_void)
                .map_err(|e| BenchError::BenchTest("Failed to install configuration table", e))?;
        }
        let end = Arch::cpu_count();
        stats.update((end - start) as f64);
    }
    Ok(stats)
}
