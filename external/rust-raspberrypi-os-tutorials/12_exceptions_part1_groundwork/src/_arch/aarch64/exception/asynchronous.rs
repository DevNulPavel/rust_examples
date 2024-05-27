// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright (c) 2018-2020 Andre Richter <andre.o.richter@gmail.com>

//! Architectural asynchronous exception handling.

use cortex_a::regs::*;

//--------------------------------------------------------------------------------------------------
// Private Definitions
//--------------------------------------------------------------------------------------------------

trait DaifField {
    fn daif_field() -> register::Field<u32, DAIF::Register>;
}

struct Debug;
struct SError;
struct IRQ;
struct FIQ;

//--------------------------------------------------------------------------------------------------
// Private Code
//--------------------------------------------------------------------------------------------------

impl DaifField for Debug {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::D
    }
}

impl DaifField for SError {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::A
    }
}

impl DaifField for IRQ {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::I
    }
}

impl DaifField for FIQ {
    fn daif_field() -> register::Field<u32, DAIF::Register> {
        DAIF::F
    }
}

fn is_masked<T: DaifField>() -> bool {
    DAIF.is_set(T::daif_field())
}

//--------------------------------------------------------------------------------------------------
// Public Code
//--------------------------------------------------------------------------------------------------

/// Print the AArch64 exceptions status.
#[rustfmt::skip]
pub fn print_state() {
    use crate::info;

    let to_mask_str = |x| -> _ {
        if x { "Masked" } else { "Unmasked" }
    };

    info!("      Debug:  {}", to_mask_str(is_masked::<Debug>()));
    info!("      SError: {}", to_mask_str(is_masked::<SError>()));
    info!("      IRQ:    {}", to_mask_str(is_masked::<IRQ>()));
    info!("      FIQ:    {}", to_mask_str(is_masked::<FIQ>()));
}
