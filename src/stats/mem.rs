use crate::{
    common::{get_default_byte, SysWrapper},
    renderer::core::Mouse,
};
use byte_unit::{AdjustedByte, Byte, UnitType};

pub struct MemStats {
    total_mem: AdjustedByte,
    mem_usage: AdjustedByte,
    available_mem: AdjustedByte,
    percentage_used: f64,
    min_mem_usage: AdjustedByte,
    max_mem_usage: AdjustedByte,
}

impl MemStats {
    pub fn new() -> Self {
        let default_byte: AdjustedByte = *get_default_byte();
        MemStats {
            total_mem: default_byte,
            mem_usage: default_byte,
            available_mem: default_byte,
            percentage_used: 1.00,
            min_mem_usage: default_byte,
            max_mem_usage: default_byte,
        }
    }

    pub fn update(&mut self, sys_obj: &mut SysWrapper) {
        sys_obj.sys.refresh_memory();

        self.total_mem =
            Byte::from_u64(sys_obj.sys.total_memory()).get_appropriate_unit(UnitType::Binary);
        self.mem_usage =
            Byte::from_u64(sys_obj.sys.used_memory()).get_appropriate_unit(UnitType::Binary);
        self.available_mem =
            Byte::from_u64(sys_obj.sys.available_memory()).get_appropriate_unit(UnitType::Binary);
        self.percentage_used =
            ((self.mem_usage.get_value() / self.total_mem.get_value()) * 100.00).round();

        self.max_mem_usage = self.max_mem_usage.max(self.mem_usage);

        if self.min_mem_usage.get_value() == 0.0 {
            self.min_mem_usage = self.mem_usage;
        } else {
            self.min_mem_usage = self.min_mem_usage.min(self.mem_usage);
        }
    }

    pub fn get_mouse(&mut self) -> Mouse {
        let title = String::from("Memory");
        let mut mouse = Mouse::new(title);
        mouse.add(format!(
            "Memory Usage:{:.2}{}/{:.2}{} {}% Max:{:.2}{} Min:{:.2}{}",
            self.mem_usage.get_value(),
            self.mem_usage.get_unit(),
            self.total_mem.get_value(),
            self.total_mem.get_unit(),
            self.percentage_used,
            self.max_mem_usage.get_value(),
            self.max_mem_usage.get_unit(),
            self.min_mem_usage.get_value(),
            self.min_mem_usage.get_unit()
        ));
        return mouse;
    }
}
