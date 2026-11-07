//! Access to the LMU shared memory block. Windows opens the game's named
//! mappings directly; other platforms read a mirror file (see `unix`).

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
mod unix;
#[cfg(not(windows))]
pub use unix::*;
