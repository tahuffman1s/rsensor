use common::SysWrapper;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use renderer::core::Rat;
use stats::mem::MemStats;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

mod common;
mod renderer;
mod stats;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sys: SysWrapper = SysWrapper::new();
    let mut memstats: MemStats = MemStats::new();
    let mut rat: Rat = Rat::new();
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    while running.load(Ordering::SeqCst) {
        // Poll for events
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Exit on 'q' key
                if key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL)
                {
                    break;
                }
            }
        }

        // Update UI on tick
        if last_tick.elapsed() >= tick_rate {
            memstats.update(&mut sys);
            rat.add(memstats.get_mouse());
            rat.draw()?;
            last_tick = Instant::now();
        }
    }
    rat.cleanup()?;

    Ok(())
}
