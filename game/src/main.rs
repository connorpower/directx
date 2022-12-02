// The feature flag `stdio` can be used to conditionally disable the windows
// subsystem which allows program output to be sent to the console which
// launched the app. Useful mostly for debugging.
#![cfg_attr(not(feature = "stdio"), windows_subsystem = "windows")]

mod game;
mod resources;
mod trace;

use crate::game::Game;
use ::tracing::{error, info};
use ::win32::proc::{self, ComLibraryHandle};

pub fn main() {
    proc::enable_heap_protection();
    trace::configure();

    info!(
        version = env!("CARGO_PKG_VERSION"),
        bin = env!("CARGO_BIN_NAME"),
        "Starting"
    );

    {
        let _com_handle = ComLibraryHandle::acquire();
        if let Err(e) = Game::new().run() {
            error!(error = %e);
        }
    }

    info!(
        version = env!("CARGO_PKG_VERSION"),
        bin = env!("CARGO_BIN_NAME"),
        "Terminating"
    );
}
