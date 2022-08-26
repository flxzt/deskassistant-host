use std::ffi::CString;
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
    pub fn new() -> PyResult<Self> {
        Ok(Self(UsbConnection::new()?))
    }

    pub fn handle_events(&mut self) -> PyResult<()> {
        Ok(self.0.handle_events()?)
    }

    pub fn is_connected(&self) -> bool {
        self.0.is_connected()
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

    pub fn convert_send_image_data(
        &self,
        width: u32,
        height: u32,
        image_data: Vec<u8>,
        timeout_ms: u64,
    ) -> PyResult<()> {
        let timeout = Duration::from_millis(timeout_ms);

        let epd_format = EpdImageFormat {
            width: EPD_WIDTH,
            height: EPD_HEIGHT,
        };
        let image_bytes =
            EpdImage::load_from_data(width, height, image_data)?.export(&epd_format)?;

        self.0
            .send_host_message(HostMessage::UpdateUserImage { format: epd_format }, timeout)?;
        self.0.transmit_host_data(&image_bytes, timeout)?;

        self.0
            .send_host_message(HostMessage::DataComplete, timeout)?;

        Ok(())
    }

    pub fn convert_send_image_file(&self, image_file: PathBuf, timeout_ms: u64) -> PyResult<()> {
        let timeout = Duration::from_millis(timeout_ms);

        if !image_file.exists() {
            Err(anyhow::anyhow!(
                "`{}` does not exist. Exiting",
                image_file.display()
            ))?;
        }

        let epd_format = EpdImageFormat {
            width: EPD_WIDTH,
            height: EPD_HEIGHT,
        };
        let image_bytes = EpdImage::load_from_file(&image_file)?.export(&epd_format)?;

        self.0
            .send_host_message(HostMessage::UpdateUserImage { format: epd_format }, timeout)?;
        self.0.transmit_host_data(&image_bytes, timeout)?;

        self.0
            .send_host_message(HostMessage::DataComplete, timeout)?;

        Ok(())
    }

    pub fn refresh_display(&self, timeout_ms: u64) -> PyResult<()> {
        Ok(self.0.send_host_message(
            HostMessage::RefreshDisplay,
            Duration::from_millis(timeout_ms),
        )?)
    }

    pub fn report_active_app_name(&self, app_name: String, timeout_ms: u64) -> PyResult<()> {
        let timeout = Duration::from_millis(timeout_ms);
        let app_name_cstr = CString::new(app_name)?.into_bytes_with_nul();
        let str_len = app_name_cstr.len() as u16;

        self.0
            .send_host_message(HostMessage::ReportActiveApp { str_len }, timeout)?;
        self.0.transmit_host_data(&app_name_cstr, timeout)?;
        self.0
            .send_host_message(HostMessage::DataComplete, timeout)?;

        Ok(())
    }
}
