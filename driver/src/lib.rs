pub mod epdimage;
pub mod pybindings;

use std::time::Duration;

// Re-Exports
pub use epdimage::EpdImage;

use pyo3::prelude::*;

pub const USB_DEVICE_VID: u16 = 0x0483;
pub const USB_DEVICE_PID: u16 = 0x0456;

pub const USB_HOST_MSG_LEN: usize = 64;
pub const USB_DEVICE_MSG_LEN: usize = 64;

pub const EPD_WIDTH: u32 = 400;
pub const EPD_HEIGHT: u32 = 300;

const EPNUM_HOST_MSG: u8 = 0x01;
const EPNUM_DEVICE_MSG: u8 = 0x82;

const ITF_NUM_MSG: u8 = 0;

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

#[derive(Debug, Clone)]
pub enum HostMessage {
    Data { data: [u8; USB_HOST_MSG_LEN - 1] },
    DataComplete,
    RequestDeviceStatus,
    SwitchPage(EpdPage),
    UpdateUserImage { format: epdimage::EpdImageFormat },
    RefreshDisplay,
    ReportActiveApp { str_len: u16 },
}

impl HostMessage {
    pub fn into_data(self) -> [u8; USB_HOST_MSG_LEN] {
        let mut msg_data: [u8; USB_HOST_MSG_LEN] = [0; USB_HOST_MSG_LEN];

        match self {
            HostMessage::Data { data } => {
                msg_data[0] = 0x00; // Host message variant

                // Copy the data into the msg
                for (to, from) in msg_data.iter_mut().skip(1).zip(data.into_iter()) {
                    *to = from;
                }
            }
            HostMessage::DataComplete => {
                msg_data[0] = 0x01; // Host message variant
            }
            HostMessage::RequestDeviceStatus => {
                msg_data[0] = 0x02; // Host message variant
            }
            HostMessage::SwitchPage(page) => {
                msg_data[0] = 0x03; // Host message variant
                msg_data[1] = page.try_into().unwrap();
            }
            HostMessage::UpdateUserImage { format } => {
                msg_data[0] = 0x04; // Host message variant
                msg_data[1] = ((format.width >> 8) & 0xff) as u8;
                msg_data[2] = (format.width & 0xff) as u8;
                msg_data[3] = ((format.height >> 8) & 0xff) as u8;
                msg_data[4] = (format.height & 0xff) as u8;
            }
            HostMessage::RefreshDisplay => {
                msg_data[0] = 0x05; // Host message variant
            }
            HostMessage::ReportActiveApp { str_len } => {
                msg_data[0] = 0x06; // Host message variant
                msg_data[1] = ((str_len >> 8) & 0xff) as u8;
                msg_data[2] = (str_len & 0xff) as u8;
            }
        }

        msg_data
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DeviceMessage {
    DeviceStatus(DeviceStatus),
}

impl DeviceMessage {
    pub fn from_data(data: &[u8; USB_DEVICE_MSG_LEN]) -> anyhow::Result<Self> {
        match data[0] {
            0x00 => Ok(Self::DeviceStatus(DeviceStatus {
                current_epd_page: EpdPage::try_from(data[1])?,
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
    device_handle: Option<rusb::DeviceHandle<rusb::GlobalContext>>,
}

impl UsbConnection {
    pub fn new() -> Self {
        Self {
            device_handle: None,
        }
    }

    pub fn open(&mut self) -> anyhow::Result<()> {
        let vid = USB_DEVICE_VID;
        let pid = USB_DEVICE_PID;

        if self.device_handle.is_some() {
            // Already connected, early return
            return Ok(());
        }

        let mut device_h = rusb::open_device_with_vid_pid(vid, pid).ok_or_else(|| {
            anyhow::anyhow!(
                "Could not open device. Not found for VID `{}`, PID `{}",
                vid,
                pid
            )
        })?;

        device_h.claim_interface(ITF_NUM_MSG)?;

        self.device_handle = Some(device_h);

        Ok(())
    }

    pub fn send_host_message(&self, msg: HostMessage, timeout: Duration) -> anyhow::Result<()> {
        let data = msg.into_data();

        let device_handle = self
            .device_handle
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no device opened"))?;

        device_handle.write_bulk(EPNUM_HOST_MSG, &data, timeout)?;
        Ok(())
    }

    pub fn read_device_message(&self, timeout: Duration) -> anyhow::Result<DeviceMessage> {
        let mut data = [0_u8; USB_HOST_MSG_LEN];

        let device_handle = self
            .device_handle
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no device opened"))?;

        device_handle.read_bulk(EPNUM_DEVICE_MSG, &mut data, timeout)?;

        DeviceMessage::from_data(&data)
    }

    pub fn transmit_host_data(&self, data: &[u8], timeout: Duration) -> anyhow::Result<()> {
        let mut chunk_iter = data.chunks_exact(USB_HOST_MSG_LEN - 1);
        for chunk in chunk_iter.by_ref() {
            self.send_host_message(
                HostMessage::Data {
                    data: chunk[0..USB_HOST_MSG_LEN - 1].try_into().unwrap(),
                },
                timeout,
            )?;
        }

        let mut remainder = chunk_iter.remainder().to_vec();
        remainder.resize(USB_HOST_MSG_LEN - 1, 0x00);

        self.send_host_message(
            HostMessage::Data {
                data: remainder.try_into().unwrap(),
            },
            timeout,
        )?;

        Ok(())
    }

    pub fn receive_device_data(&self, _buf: &mut [u8], _timeout: Duration) -> anyhow::Result<()> {
        todo!();
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
