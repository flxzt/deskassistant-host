pub mod epdimage;
pub mod pybindings;

// Re-Exports
pub use epdimage::EpdImage;

use anyhow::Context;
use pyo3::prelude::*;

pub const USB_DEVICE_VID: u16 = 1155;
pub const USB_DEVICE_PID: u16 = 0x456;

pub const USB_HID_REPORT_ID1_LEN: usize = 8 + 1;
pub const USB_HID_REPORT_ID2_LEN: usize = 8 + 1;
pub const USB_HID_REPORT_ID3_LEN: usize = 63 + 1;

pub const EPD_WIDTH: u32 = 400;
pub const EPD_HEIGHT: u32 = 300;

#[derive(
    Debug, Clone, Copy, clap::ValueEnum, num_derive::FromPrimitive, num_derive::ToPrimitive,
)]
#[pyclass]
pub enum EpdPage {
    First = 0,
    Second = 1,
    Third = 2,
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

type HidReport = Vec<u8>;

#[derive(Debug, Clone)]
pub enum HostMessage {
    RequestDeviceStatus,
    SwitchPage(EpdPage),
    UpdateUserImage {
        format: epdimage::EpdImageFormat,
    },
    UpdateUserImageComplete,
    RefreshDisplay,
    /// Data Transfer has Report ID 3
    DataTransfer([u8; USB_HID_REPORT_ID3_LEN - 1]),
}

impl HostMessage {
    pub fn into_hid_report(self) -> HidReport {
        match self {
            HostMessage::RequestDeviceStatus => {
                vec![
                    0x02, // Report ID
                    0x00, // Host message type
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ]
            }
            HostMessage::SwitchPage(page) => {
                vec![
                    0x02, // Report ID
                    0x01, // Host message type
                    page.try_into().unwrap(),
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                    0x00,
                ]
            }
            HostMessage::UpdateUserImage { format } => {
                vec![
                    0x02, // Report ID
                    0x02, // Host message type
                    (format.width & 0xff) as u8,
                    ((format.width >> 8) & 0xff) as u8,
                    (format.height & 0xff) as u8,
                    ((format.height >> 8) & 0xff) as u8,
                    0x00,
                    0x00,
                    0x00,
                ]
            }
            HostMessage::UpdateUserImageComplete => {
                vec![
                    0x02, // Report ID
                    0x03, // Host message type
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ]
            }
            HostMessage::RefreshDisplay => {
                vec![
                    0x02, // Report ID
                    0x04, // Host message type
                    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                ]
            }
            HostMessage::DataTransfer(data) => {
                let mut report = vec![
                    0x03, // Report ID
                ];
                report.extend(data);

                report
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceMessage {
    DeviceStatus(DeviceStatus),
}

impl DeviceMessage {
    /// Must include the report ID as first byte
    pub fn from_hid_report_data(
        data: &[u8; USB_HID_REPORT_ID1_LEN],
    ) -> anyhow::Result<Self> {
        if data[0] != 0x01 {
            return Err(anyhow::anyhow!(
                "Could not extract DeviceMessage from data, invalid report ID `{}`.",
                data[0]
            ));
        }

        match data[1] {
            0x00 => Ok(Self::DeviceStatus(DeviceStatus {
                current_epd_page: EpdPage::try_from(data[2])?,
            })),
            _ => Err(anyhow::anyhow!(
                "Could not extract DeviceMessage from data, invalid message variant"
            )),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[pyclass]
pub struct DeviceStatus {
    #[allow(unused)]
    #[pyo3(get, set)]
    current_epd_page: EpdPage,
}

#[pyclass]
pub struct UsbConnection {
    #[allow(unused)]
    api: hidapi::HidApi,
    device: hidapi::HidDevice,
}

impl UsbConnection {
    pub fn init() -> anyhow::Result<Self> {
        let vid = USB_DEVICE_VID;
        let pid = USB_DEVICE_PID;

        let api = hidapi::HidApi::new().context("creating hid api failed.")?;
        let device = api.open(vid, pid).context(format!(
            "open device with VID `{:#06x}`, PID `{:#06x}` failed.",
            vid, pid
        ))?;
        device.set_blocking_mode(true)?;

        Ok(Self { api, device })
    }

    pub fn send_host_message(&self, msg: HostMessage) -> anyhow::Result<()> {
        let report = msg.into_hid_report();

        self.device.write(&report)?;
        self.api.check_error()?;

        Ok(())
    }

    pub fn read_device_message(&self, timeout: u16) -> anyhow::Result<DeviceMessage> {
        let mut data = [0_u8; USB_HID_REPORT_ID1_LEN];

        self.device.read_timeout(&mut data, timeout as i32)?;
        log::debug!("USB: RECV DATA: {:?}", data);

        self.api.check_error()?;

        DeviceMessage::from_hid_report_data(&data)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
