use byte_unit::{AdjustedByte, Byte, UnitType};
use std::sync::OnceLock;
use sysinfo::System;

static DEFAULT_BYTE: OnceLock<AdjustedByte> = OnceLock::new();
//static SYSTEM: OnceLock<Mutex<System>> = OnceLock::new();

pub fn get_default_byte() -> &'static AdjustedByte {
    DEFAULT_BYTE.get_or_init(|| Byte::from_u64(0).get_appropriate_unit(UnitType::Binary))
}

pub struct SysWrapper {
    pub sys: System,
}

impl SysWrapper {
    pub fn new() -> Self {
        SysWrapper {
            sys: System::new_all(),
        }
    }
}

//pub fn get_system() -> &'static Mutex<System> {
//    SYSTEM.get_or_init(|| Mutex::new(System::new_all()))
//}
