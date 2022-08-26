use rusb::UsbContext;
use std::sync::mpsc;
use std::time::Duration;

use crate::{DeviceMessage, HostMessage};

pub const USB_DEVICE_VID: u16 = 0x0483;
pub const USB_DEVICE_PID: u16 = 0x0456;

pub const USB_HOST_MSG_LEN: usize = 64;
pub const USB_DEVICE_MSG_LEN: usize = 64;

const EPNUM_HOST_MSG: u8 = 0x01;
const EPNUM_DEVICE_MSG: u8 = 0x82;

const ITF_NUM_MSG: u8 = 0;

enum HotplugMessage {
    DeviceArrived(rusb::Device<rusb::Context>),
    DeviceLeft(rusb::Device<rusb::Context>),
}

struct HotPlugHandler {
    hotplugmessage_sender: mpsc::Sender<HotplugMessage>,
}

impl HotPlugHandler {
    pub fn new(hotplugmessage_sender: mpsc::Sender<HotplugMessage>) -> Self {
        Self {
            hotplugmessage_sender,
        }
    }
}

impl rusb::Hotplug<rusb::Context> for HotPlugHandler {
    fn device_arrived(&mut self, device: rusb::Device<rusb::Context>) {
        println!("device arrived: {device:?}");

        if let Err(e) = self
            .hotplugmessage_sender
            .send(HotplugMessage::DeviceArrived(device))
        {
            println!("device arrived, but sending it to the usb connection failed with Err {e}");
        }
    }

    fn device_left(&mut self, device: rusb::Device<rusb::Context>) {
        println!("device left: {device:?}");

        if let Err(e) = self
            .hotplugmessage_sender
            .send(HotplugMessage::DeviceLeft(device))
        {
            println!("device left, but sending it to the usb connection failed with Err {e}");
        }
    }
}

pub struct UsbConnection {
    context: rusb::Context,
    #[allow(unused)]
    hotplug_reg: rusb::Registration<rusb::Context>,
    hotplugmessage_receiver: mpsc::Receiver<HotplugMessage>,
    device_handle: Option<rusb::DeviceHandle<rusb::Context>>,
}

impl UsbConnection {
    pub fn new() -> anyhow::Result<Self> {
        let context = rusb::Context::new()?;
        let (sender, receiver) = mpsc::channel();

        let hotplug_reg = rusb::HotplugBuilder::new()
            .enumerate(true)
            .enumerate(true)
            .vendor_id(USB_DEVICE_VID)
            .product_id(USB_DEVICE_PID)
            .register(&context, Box::new(HotPlugHandler::new(sender)))?;

        Ok(Self {
            context,
            hotplug_reg,
            hotplugmessage_receiver: receiver,
            device_handle: None,
        })
    }

    pub fn is_connected(&self) -> bool {
        self.device_handle.is_some()
    }

    /// Handles already-pending non-synchronous events. It drives the hotplug support,
    /// which then automatically connects to the device when it is found
    pub fn handle_events(&mut self) -> anyhow::Result<()> {
        // If timeout is less than a microsecond, handle_events() only processes already-pending events
        // and then returns in non-blocking style
        self.context.handle_events(Some(Duration::from_nanos(1)))?;

        for hotplugmessage in self.hotplugmessage_receiver.try_iter() {
            match hotplugmessage {
                HotplugMessage::DeviceArrived(arrived_device) => {
                    if let Some(mut device_handle) = arrived_device.open().ok() {
                        // By dropping the old handle the claimed interfaces get disconnected.
                        self.device_handle.take().map(|h| drop(h));

                        device_handle.claim_interface(ITF_NUM_MSG)?;

                        self.device_handle.replace(device_handle);
                    }
                }
                HotplugMessage::DeviceLeft(left_device) => {
                    if let Some(ref device_handle) = self.device_handle {
                        if device_handle.device() == left_device {
                            self.device_handle.take().map(|h| drop(h));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Sends a host message.
    /// Blocks until finished
    pub fn send_host_message(&self, msg: HostMessage, timeout: Duration) -> anyhow::Result<()> {
        let data = msg.into_data();

        let device_handle = self
            .device_handle
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Device not connected."))?;

        device_handle.write_bulk(EPNUM_HOST_MSG, &data, timeout)?;
        Ok(())
    }

    /// Reads a message from the device.
    /// Blocks until finished
    pub fn read_device_message(&self, timeout: Duration) -> anyhow::Result<DeviceMessage> {
        let mut data = [0_u8; USB_HOST_MSG_LEN];

        let device_handle = self
            .device_handle
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Device not connected."))?;

        device_handle.read_bulk(EPNUM_DEVICE_MSG, &mut data, timeout)?;

        DeviceMessage::from_data(&data)
    }

    /// Transmits the entire slice to the device with Data messages.
    /// Blocks until finished
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

    /// Reads data from the device until the buf is filled.
    /// Blocks until finished
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
