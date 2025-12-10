//! Errors that can occur during benchmarking.
//!
//! Copyright (c) Microsoft Corporation.
//!
//! SPDX-License-Identifier: Apache-2.0
//!

use core::fmt;

use r_efi::efi;

#[derive(Debug)]
pub enum BenchError {
    BenchSetup(&'static str, efi::Status),
    BenchTest(&'static str, efi::Status),
    BenchCleanup(&'static str, efi::Status),
    WriteOutput(&'static str, core::fmt::Error),
}

impl fmt::Display for BenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BenchError::BenchSetup(msg, status)
            | BenchError::BenchTest(msg, status)
            | BenchError::BenchCleanup(msg, status) => {
                write!(f, "{} with error {:?}", msg, status)
            }
            BenchError::WriteOutput(msg, err) => {
                write!(f, "{} with formatting error {:?}", msg, err)
            }
        }
    }
}
