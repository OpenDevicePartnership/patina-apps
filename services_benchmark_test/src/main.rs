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

        #[entry]
        fn main() -> Status {
            uefi::helpers::init().unwrap();

            // Convert UEFI types to r-efi compatible types
            let handle = uefi::boot::image_handle().as_ptr() as *mut core::ffi::c_void;

            bench_start(handle as r_efi::efi::Handle, st).unwrap_or_else(|e| {
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
