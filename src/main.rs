use common::SysWrapper;
use renderer::core::Rat;
use stats::mem::MemStats;

mod common;
mod renderer;
mod stats;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sys: SysWrapper = SysWrapper::new();
    let mut memstats: MemStats = MemStats::new();
    let mut rat: Rat = Rat::new();
    loop {
        memstats.update(&mut sys);
        rat.add(memstats.get_mouse());
        let _draw = rat.draw();
    }
}
