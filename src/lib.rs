#![no_std]
#![allow(async_fn_in_trait)] // doesn't use Dynamic Dispact or Send

/// provide hardware interface
mod interface;
pub use interface::EpdInterface;

/// provide hardware drivers
pub mod drivers;

/// provide embedded graphics DrawTarget
mod graphics;
pub use graphics::{DisplayRotation, EpdDrawTarget};
