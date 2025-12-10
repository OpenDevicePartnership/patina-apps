//! UEFI shell app benchmark test for basic boot and runtime services.
//!
//! ## License
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!
#![cfg_attr(target_os = "uefi", no_std)]
#![cfg_attr(target_os = "uefi", no_main)]

cfg_if::cfg_if! {
    if #[cfg(all(target_os = "uefi"))] {
        use core::panic::PanicInfo;
        use uefi::prelude::*;
        use services_benchmark_test::bench_start;
        use r_efi::efi;
        use services_benchmark_test::BOOT_SERVICES;
        use log::LevelFilter;
        use patina::boot_services::protocol_handler::HandleSearchType;
        use patina::boot_services::BootServices;

        #[entry]
        fn main() -> Status {
            uefi::helpers::init().unwrap();
            log::info!("UEFI Services Benchmark Test Entry Point");

            let st = uefi::table::system_table_raw();
            if let Some(st_ptr) = st {
                let st = st_ptr.as_ptr();
                // SAFETY: `uefi` crate ensures that the system table pointer is valid after initialization.
                let system_table = unsafe { &*st };
                // SAFETY: `uefi` crate ensures that the boot services pointer is valid after initialization.
                let bs = unsafe { &*(system_table.boot_services as *const efi::BootServices) };
                BOOT_SERVICES.init(bs);
            }

            // Convert UEFI types to r-efi compatible types.
            let handle = uefi::boot::image_handle().as_ptr();

            bench_start(handle as r_efi::efi::Handle).unwrap_or_else(|e| {
                log::error!("Services Benchmark Test failed: {:?}", e);
            });

            Status::SUCCESS
        }

        #[panic_handler]
        fn panic(_info: &PanicInfo) -> ! {
            loop {}
        }
    } else {
        fn main() {}
    }
}
