#![no_std]

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
    /// wait on the N_BUSY pin
    fn wait_until_idle(&mut self) ->
        Result<(), display_interface::DisplayError>;
}



// /// provide embedded_graphics support
// #[maybe_async_cfg::maybe(
//     sync(keep_self, cfg(not(feature = "async"))),
//     async(keep_self, feature = "async"),
//     idents(
//         // AsyncWriteOnlyDataCommand(async, sync = "WriteOnlyDataCommand"),
//         init(keep),
//         flush(keep),
//     )
// )]
// pub trait EpdDrawTarget
// {
//     /// configure the hardware for graphics
//     async fn init(&mut self) -> Result<(), display_interface::DisplayError>;

//     async fn flush(&mut self) -> Result<(), display_interface::DisplayError>;
// }