use std::ffi::CString;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use deskassistant_driver::epdimage::EpdImageFormat;
use deskassistant_driver::{EpdImage, EpdPage, HostMessage, UsbConnection, EPD_HEIGHT, EPD_WIDTH};

#[derive(Debug, Clone, clap::Subcommand)]
#[non_exhaustive]
enum CliCommand {
    /// display the current status of the device
    #[clap(action)]
    Status,
    /// switch to another page
    Switch {
        #[clap(value_enum)]
        page: EpdPage,
    },
    /// Decode and send image for display on the EPD
    SendImage {
        #[clap(value_parser)]
        file: PathBuf,
    },
    /// Refresh the display
    #[clap(action)]
    RefreshDisplay,
    /// Report an active app name
    #[clap(action)]
    ReportActiveApp {
        #[clap(value_parser)]
        app_name: String,
    },
}

/// the cli for the deskassistant project
#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Option<CliCommand>,
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    log::debug!("init");

    let cli = Cli::parse();
    let mut connection = UsbConnection::new();
    connection.open()?;

    let timeout = Duration::from_millis(5_000);

    if let Some(command) = cli.command {
        match command {
            CliCommand::Status => {
                connection.send_host_message(HostMessage::RequestDeviceStatus, timeout)?;

                let device_message = connection.read_device_message(timeout)?;
                println!("device message: {device_message:?}");
            }
            CliCommand::Switch { page } => {
                connection.send_host_message(HostMessage::SwitchPage(page), timeout)?;
            }
            CliCommand::SendImage { file } => {
                if !file.exists() {
                    return Err(anyhow::anyhow!(
                        "`{}` does not exist. Exiting",
                        file.display()
                    ));
                }

                let format = EpdImageFormat {
                    width: EPD_WIDTH,
                    height: EPD_HEIGHT,
                };
                let image_bytes = EpdImage::load_from_file(&file)?.export(&format)?;

                connection.send_host_message(HostMessage::UpdateUserImage { format }, timeout)?;
                connection.transmit_host_data(&image_bytes, timeout)?;

                connection.send_host_message(HostMessage::DataComplete, timeout)?;
            }
            CliCommand::RefreshDisplay => {
                connection.send_host_message(HostMessage::RefreshDisplay, timeout)?;
            }
            CliCommand::ReportActiveApp { app_name } => {
                let app_name_cstr = CString::new(app_name)?.into_bytes_with_nul();
                let str_len = app_name_cstr.len() as u16;

                connection.send_host_message(HostMessage::ReportActiveApp { str_len }, timeout)?;
                connection.transmit_host_data(&app_name_cstr, timeout)?;
                connection.send_host_message(HostMessage::DataComplete, timeout)?;
            }
        }
    }

    Ok(())
}
