#![no_std]
#![allow(async_fn_in_trait)]    // doesn't use Dynamic Dispact or Send
 
/// provide hardware interface
mod interface;
pub use interface::EpdInterface;


mod drivers;
/// provide hardware drivers
pub use drivers::E0213A367::E0213A367;


/// Display rotation.
// #[derive(Copy, Clone, Debug)]
pub enum DisplayRotation {
    /// No rotation, normal display
    Rotate0,
    /// Rotate by 90 degrees clockwise
    Rotate90,
    /// Rotate by 180 degrees clockwise, upside down display
    Rotate180,
    /// Rotate 270 degrees clockwise
    Rotate270,
}


pub trait WaitUntilIdle {
    /// blocks until idle, or returns error upon timeout
    fn wait_until_idle(&mut self) ->
        Result<(), display_interface::DisplayError>;
}
/// Yielding 
pub trait AsyncWaitUntilIdle {
    /// yields until idle, or returns error upon timeout
    async fn wait_until_idle(&mut self) ->
        Result<(), display_interface::DisplayError>;
}



/// provide embedded_graphics support
#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async"),
    idents(
        flush(keep),
    )
)]
// pub trait EpdDrawTarget : embedded_graphics::draw_target::DrawTarget
pub trait EpdDrawTarget
{
    /// EPDs don't support drawable pixels, so after "drawing" a flush() must
    /// be issued to update the display
    async fn flush(&mut self) -> Result<(), display_interface::DisplayError>;
}