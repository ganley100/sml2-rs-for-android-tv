//! Reusable native SML2 runtime.
//!
//! `Session::advance` accepts explicit tick inputs and returns deferred effects.
//! Content loading, CLI/window hosting, and read-only rendering have separate owners.
//! Original diagnostic play and frozen replay adapters preserve their existing loops.

// Android builds drop the diagnostic CLI, leaving its helpers unused there.
#![cfg_attr(target_os = "android", allow(dead_code))]

pub mod audio_output;
pub mod content;
pub mod host;
pub mod persistence;
pub mod play_tape;
pub mod present;
pub mod render;
pub mod scene_scenery;
pub mod session;

#[cfg(not(target_os = "android"))]
mod cli;
pub mod divider_clock;
mod game;
mod image_output;
#[cfg(not(target_os = "android"))]
mod play;
mod replay;
#[cfg(not(target_os = "android"))]
mod viewer;

#[cfg(not(target_os = "android"))]
pub use cli::run_cli;
pub use session::{Boot, FrameEffects, FrameInput, Session};

/// Native GB screen; the HUD strip and parity checks are anchored to it.
pub const SCREEN_W: usize = 160;
pub const SCREEN_H: usize = 144;

#[cfg(test)]
mod tests;
