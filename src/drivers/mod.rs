/// Trait for EpdDrivers to support EpdDrawTarget
#[maybe_async_cfg::maybe(
    sync(keep_self, cfg(not(feature = "async"))),
    async(keep_self, feature = "async")
)]
pub trait EpdDriver {
    fn dimensions(&self) -> embedded_graphics::geometry::Size;

    /// initialize the chip post power-on
    async fn init(
        &mut self,
    ) -> Result<(), display_interface::DisplayError>;

    async fn refresh(
        &mut self,
        frame_buffer: &fixedbitset::FixedBitSet,
    ) -> Result<(), display_interface::DisplayError>;
}

#[allow(non_snake_case)]
mod _E0213A367;
pub use _E0213A367::E0213A367 as E0213A367;

// #[allow(non_snake_case)]
// mod _DEPG0213BNS800;
// pub use _DEPG0213BNS800::DEPG0213BNS800 as DEPG0213BNS80;

// #[allow(non_snake_case)]
// mod _LCMEN2R13EFC;
// pub use _LCMEN2R13EFC::LCMEN2R13EFC as LCMEN2R13EFC;