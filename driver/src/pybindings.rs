use std::path::PathBuf;
use std::time::Duration;

use pyo3::prelude::*;

use crate::epdimage::EpdImageFormat;
use crate::{
    DeviceMessage, DeviceStatus, EpdImage, EpdPage, HostMessage, UsbConnection, EPD_HEIGHT,
    EPD_WIDTH,
};

#[pymodule]
fn deskassistant_driver(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyUsbConnection>()?;
    m.add_class::<DeviceStatus>()?;
    m.add_class::<EpdPage>()?;

    Ok(())
}

#[pyclass]
pub struct PyUsbConnection(UsbConnection);

#[pymethods]
impl PyUsbConnection {
    #[staticmethod]
    pub fn new() -> Self {
        Self(UsbConnection::new())
    }

    pub fn open(&mut self) -> PyResult<()> {
        Ok(self.0.open()?)
    }

    pub fn opened(&self) -> bool {
        self.0.device_handle.is_some()
    }

    pub fn retreive_device_status(&self, timeout_ms: u64) -> PyResult<DeviceStatus> {
        self.0.send_host_message(
            HostMessage::RequestDeviceStatus,
            Duration::from_millis(timeout_ms),
        )?;
        match self
            .0
            .read_device_message(Duration::from_millis(timeout_ms))?
        {
            DeviceMessage::DeviceStatus(status) => Ok(status),
        }
    }

    pub fn switch_page(&self, page: EpdPage, timeout_ms: u64) -> PyResult<()> {
        Ok(self.0.send_host_message(
            HostMessage::SwitchPage(page),
            Duration::from_millis(timeout_ms),
        )?)
    }

    pub fn send_image(&self, image_file: PathBuf, timeout_ms: u64) -> PyResult<()> {
        let timeout = Duration::from_millis(timeout_ms);

        if !image_file.exists() {
            Err(anyhow::anyhow!(
                "`{}` does not exist. Exiting",
                image_file.display()
            ))?;
        }

        let format = EpdImageFormat {
            width: EPD_WIDTH,
            height: EPD_HEIGHT,
        };
        let image_bytes = EpdImage::load_from_file(&image_file)?.export(&format)?;

        self.0
            .send_host_message(HostMessage::UpdateUserImage { format }, timeout)?;
        self.0.transmit_host_data(&image_bytes, timeout)?;

        self.0
            .send_host_message(HostMessage::UpdateUserImageComplete, timeout)?;

        Ok(())
    }

    pub fn refresh_display(&self, timeout_ms: u64) -> PyResult<()> {
        Ok(self.0.send_host_message(
            HostMessage::RefreshDisplay,
            Duration::from_millis(timeout_ms),
        )?)
    }
}
