use pyo3::prelude::*;

use crate::{DeviceMessage, DeviceStatus, HostMessage, UsbConnection, EpdPage};

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
    pub fn init() -> PyResult<Self> {
        Ok(Self(UsbConnection::init()?))
    }

    pub fn retreive_device_status(&self, timeout: u16) -> PyResult<DeviceStatus> {
        self.0.send_host_message(HostMessage::RequestDeviceStatus)?;
        match self.0.read_device_message(timeout)? {
            DeviceMessage::DeviceStatus(status) => Ok(status),
        }
    }

    pub fn switch_page(&self, page: EpdPage) -> PyResult<()> {
        Ok(self.0.send_host_message(HostMessage::SwitchPage(page))?)
    }
}
