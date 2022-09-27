use crate::connection::{USB_DEVICE_MSG_LEN, USB_HOST_MSG_LEN};
use crate::{DeviceStatus, EpdImageFormat, EpdPage};

#[derive(Debug, Clone)]
pub enum HostMessage {
    Data { data: [u8; USB_HOST_MSG_LEN - 1] },
    DataComplete,
    RequestDeviceStatus,
    RefreshDisplay,
    SwitchPage(EpdPage),
    UpdateUserImage { format: EpdImageFormat },
    UpdateAppImage { app_name_str_len: u16, format: EpdImageFormat },
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
            HostMessage::RefreshDisplay => {
                msg_data[0] = 0x03; // Host message variant
            }
            HostMessage::SwitchPage(page) => {
                msg_data[0] = 0x04; // Host message variant
                msg_data[1] = page.try_into().unwrap();
            }
            HostMessage::UpdateUserImage { format } => {
                msg_data[0] = 0x05; // Host message variant
                msg_data[1] = ((format.width >> 8) & 0xff) as u8;
                msg_data[2] = (format.width & 0xff) as u8;
                msg_data[3] = ((format.height >> 8) & 0xff) as u8;
                msg_data[4] = (format.height & 0xff) as u8;
            }
            HostMessage::UpdateAppImage { app_name_str_len, format } => {
                msg_data[0] = 0x06; // Host message variant
                msg_data[1] = ((format.width >> 8) & 0xff) as u8;
                msg_data[2] = (format.width & 0xff) as u8;
                msg_data[3] = ((format.height >> 8) & 0xff) as u8;
                msg_data[4] = (format.height & 0xff) as u8;
                msg_data[5] = ((app_name_str_len >> 8) & 0xff) as u8;
                msg_data[6] = (app_name_str_len & 0xff) as u8;
            }
            HostMessage::ReportActiveApp { str_len } => {
                msg_data[0] = 0x07; // Host message variant
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
