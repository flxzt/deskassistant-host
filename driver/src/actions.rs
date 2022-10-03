use std::ffi::CString;
use std::path::PathBuf;
use std::time::Duration;

use crate::{
    DeviceMessage, DeviceStatus, EpdImage, EpdImageFormat, EpdPage, HostMessage, UsbConnection,
    EPD_HEIGHT, EPD_WIDTH,
};

pub fn retreive_device_status(
    connection: &UsbConnection,
    timeout: Duration,
) -> anyhow::Result<DeviceStatus> {
    connection.send_host_message(HostMessage::RequestDeviceStatus, timeout)?;

    match connection.read_device_message(timeout)? {
        DeviceMessage::DeviceStatus(status) => Ok(status),
        msg => Err(anyhow::anyhow!(
            "failed to retreive device status, received unexpected device message: `{:?}`",
            msg
        )),
    }
}

pub fn refresh_display(connection: &UsbConnection, timeout: Duration) -> anyhow::Result<()> {
    connection.send_host_message(HostMessage::RefreshDisplay, timeout)?;
    Ok(())
}

pub fn switch_page(
    connection: &UsbConnection,
    page: EpdPage,
    timeout: Duration,
) -> anyhow::Result<()> {
    connection.send_host_message(HostMessage::SwitchPage(page), timeout)?;
    Ok(())
}

pub fn update_user_image_from_file(
    connection: &UsbConnection,
    img_file: PathBuf,
    timeout: Duration,
) -> anyhow::Result<()> {
    if !img_file.exists() {
        return Err(anyhow::anyhow!(
            "`{}` does not exist. Exiting",
            img_file.display()
        ));
    }

    let format = EpdImageFormat {
        width: EPD_WIDTH,
        height: EPD_HEIGHT,
    };
    let image_bytes = EpdImage::load_from_file(&img_file)?.export(&format)?;

    connection.send_host_message(HostMessage::UpdateUserImage { format }, timeout)?;
    connection.transmit_host_data(&image_bytes, timeout)?;

    connection.send_host_message(HostMessage::DataComplete, timeout)?;
    Ok(())
}

pub fn update_app_image_from_file(
    connection: &UsbConnection,
    app_name: String,
    img_file: PathBuf,
    timeout: Duration,
) -> anyhow::Result<()> {
    if !img_file.exists() {
        return Err(anyhow::anyhow!(
            "`{}` does not exist. Exiting",
            img_file.display()
        ));
    }

    let format = EpdImageFormat {
        width: EPD_WIDTH,
        height: EPD_HEIGHT,
    };
    let image_bytes = EpdImage::load_from_file(&img_file)?.export(&format)?;

    let app_name_cstr = CString::new(app_name)?.into_bytes_with_nul();
    let str_len = (app_name_cstr.len() as u16).saturating_sub(1);

    connection.send_host_message(
        HostMessage::UpdateAppImage {
            app_name_str_len: str_len,
            format,
        },
        timeout,
    )?;

    // First send the app name string
    connection.transmit_host_data(&app_name_cstr, timeout)?;
    connection.send_host_message(HostMessage::DataComplete, timeout)?;

    // Then the app image data
    connection.transmit_host_data(&image_bytes, timeout)?;
    connection.send_host_message(HostMessage::DataComplete, timeout)?;

    Ok(())
}

pub fn report_active_app(
    connection: &UsbConnection,
    app_name: String,
    timeout: Duration,
) -> anyhow::Result<()> {
    let app_name_cstr = CString::new(app_name)?.into_bytes_with_nul();
    let str_len = (app_name_cstr.len() as u16).saturating_sub(1);

    connection.send_host_message(HostMessage::ReportActiveApp { str_len }, timeout)?;
    connection.transmit_host_data(&app_name_cstr, timeout)?;
    connection.send_host_message(HostMessage::DataComplete, timeout)?;
    Ok(())
}

pub fn retreive_app_images_list(
    connection: &UsbConnection,
    timeout: Duration,
) -> anyhow::Result<Vec<String>> {
    connection.send_host_message(HostMessage::RequestListAppImages, timeout)?;

    let str_len = match connection.read_device_message(timeout)? {
        DeviceMessage::ListAppImages { str_len } => str_len,
        msg => {
            return Err(anyhow::anyhow!(
                "failed to retreive app images list. Received unexpected device message: `{:?}`",
                msg
            ))
        }
    };

    let mut app_images_list_str = connection.receive_device_data(timeout, None)?;

    app_images_list_str.resize(str_len as usize + 1, 0x00);

    let app_images_list_str = CString::from_vec_with_nul(app_images_list_str)
        .map_err(anyhow::Error::from)?
        .into_string()?;

    Ok(app_images_list_str.lines().map(|s| s.to_string()).collect())
}
