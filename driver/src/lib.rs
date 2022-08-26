pub mod connection;
pub mod epdimage;
pub mod messages;
pub mod pybindings;

// Re-Exports
pub use connection::UsbConnection;
pub use epdimage::EpdImage;
pub use epdimage::EpdImageFormat;
pub use messages::DeviceMessage;
pub use messages::HostMessage;

use pyo3::prelude::*;

pub const EPD_WIDTH: u32 = 400;
pub const EPD_HEIGHT: u32 = 300;

#[derive(
    Debug, Clone, Copy, clap::ValueEnum, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[pyclass]
pub enum EpdPage {
    Overview = 0,
    AppScreen = 1,
    UserImage = 2,
}

impl TryFrom<EpdPage> for u8 {
    type Error = anyhow::Error;

    fn try_from(page: EpdPage) -> Result<Self, Self::Error> {
        num_traits::ToPrimitive::to_u8(&page)
            .ok_or_else(|| anyhow::anyhow!("u8 try_from::<Page>() for page {:?} failed.", page))
    }
}

impl TryFrom<u8> for EpdPage {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        num_traits::FromPrimitive::from_u8(value)
            .ok_or_else(|| anyhow::anyhow!("Page try_from::<u8>() for value {} failed.", value))
    }
}

#[derive(Debug, Clone, Copy)]
#[pyclass]
pub struct DeviceStatus {
    #[allow(unused)]
    #[pyo3(get, set)]
    current_epd_page: EpdPage,
}
