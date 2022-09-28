use std::path::PathBuf;
use std::time::Duration;

use pyo3::prelude::*;

use crate::{actions, DeviceStatus, EpdPage, UsbConnection};

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
        Ok(actions::retreive_device_status(
            &self.0,
            Duration::from_millis(timeout_ms),
        )?)
    }

    pub fn refresh_display(&self, timeout_ms: u64) -> PyResult<()> {
        Ok(actions::refresh_display(
            &self.0,
            Duration::from_millis(timeout_ms),
        )?)
    }

    pub fn switch_page(&self, page: EpdPage, timeout_ms: u64) -> PyResult<()> {
        Ok(actions::switch_page(
            &self.0,
            page,
            Duration::from_millis(timeout_ms),
        )?)
    }

    pub fn convert_send_user_image_from_file(
        &self,
        image_file: PathBuf,
        timeout_ms: u64,
    ) -> PyResult<()> {
        Ok(actions::update_user_image_from_file(
            &self.0,
            image_file,
            Duration::from_millis(timeout_ms),
        )?)
    }

    pub fn convert_send_app_image_from_file(
        &self,
        app_name: String,
        image_file: PathBuf,
        timeout_ms: u64,
    ) -> PyResult<()> {
        Ok(actions::update_app_image_from_file(
            &self.0,
            app_name,
            image_file,
            Duration::from_millis(timeout_ms),
        )?)
    }

    pub fn report_active_app_name(&self, app_name: String, timeout_ms: u64) -> PyResult<()> {
        Ok(actions::report_active_app(
            &self.0,
            app_name,
            Duration::from_millis(timeout_ms),
        )?)
    }

    pub fn retreive_app_images_list(&self, timeout_ms: u64) -> PyResult<Vec<String>> {
        Ok(actions::retreive_app_images_list(
            &self.0,
            Duration::from_millis(timeout_ms),
        )?)
    }
}
