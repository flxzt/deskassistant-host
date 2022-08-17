use std::path::PathBuf;

use clap::Parser;
use deskassistant_driver::epdimage::EpdImageFormat;
use deskassistant_driver::{
    EpdImage, EpdPage, HostMessage, UsbConnection, EPD_HEIGHT, EPD_WIDTH,
    USB_HID_REPORT_ID3_LEN,
};

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
    let connection = UsbConnection::init()?;

    if let Some(command) = cli.command {
        match command {
            CliCommand::Status => {
                connection.send_host_message(HostMessage::RequestDeviceStatus)?;

                let device_message = connection.read_device_message(5_000)?;
                println!("device message: {device_message:?}");
            }
            CliCommand::Switch { page } => {
                connection.send_host_message(HostMessage::SwitchPage(page))?;
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

                connection.send_host_message(HostMessage::UpdateUserImage { format })?;

                let byte_chunks = image_bytes.as_slice().chunks_exact(63);
                for chunk in byte_chunks.clone() {
                    connection
                        .send_host_message(HostMessage::DataTransfer(chunk.try_into().unwrap()))?;
                }

                let mut remainding = byte_chunks.remainder().to_vec();
                remainding.resize(USB_HID_REPORT_ID3_LEN - 1, 0x00);

                connection
                    .send_host_message(HostMessage::DataTransfer(remainding.try_into().unwrap()))?;

                connection.send_host_message(HostMessage::UpdateUserImageComplete)?;
            }
            CliCommand::RefreshDisplay => {
                connection.send_host_message(HostMessage::RefreshDisplay)?;
            }
        }
    }

    Ok(())
}
