use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use deskassistant_driver::{actions, EpdPage, UsbConnection};

#[derive(Debug, Clone, clap::Subcommand)]
#[non_exhaustive]
enum CliCommand {
    /// display the current status of the device
    #[clap(action)]
    Status,
    /// Refresh the display
    #[clap(action)]
    RefreshDisplay,
    /// switch to a page
    SwitchPage {
        #[clap(value_enum)]
        page: EpdPage,
    },
    /// Decode and send image for display on the EPD
    UpdateUserImage {
        #[clap(value_parser, short, long)]
        image_file: PathBuf,
    },
    /// Decode and send image for display on the EPD when the specified app (executable name) is active
    UpdateAppImage {
        #[clap(value_parser, short, long)]
        app_name: String,
        #[clap(value_parser, short, long)]
        image_file: PathBuf,
    },
    /// Report an active app name
    #[clap(action)]
    ReportActiveApp {
        #[clap(value_parser, short, long)]
        app_name: String,
    },
    /// Retreive and list the saved app images
    #[clap(action)]
    ListAppImages,
}

/// the cli for the deskassistant project
#[derive(Debug, Clone, Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(short, long, value_parser)]
    verbose: bool,
    #[clap(subcommand)]
    command: Option<CliCommand>,
}

fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();
    log::debug!("init");

    let cli = Cli::parse();
    let mut connection = UsbConnection::new()?;
    // call handle events once to drive the hotplug callback
    connection.handle_events()?;

    if !connection.is_connected() {
        return Err(anyhow::anyhow!("device is not connected. Try again."));
    }

    let timeout = Duration::from_millis(5_000);

    if let Some(command) = cli.command {
        match command {
            CliCommand::Status => {
                let device_status = actions::retreive_device_status(&connection, timeout)?;
                println!("device status: {device_status:?}");
            }
            CliCommand::RefreshDisplay => {
                actions::refresh_display(&connection, timeout)?;
            }
            CliCommand::SwitchPage { page } => {
                actions::switch_page(&connection, page, timeout)?;
            }
            CliCommand::UpdateUserImage { image_file } => {
                actions::update_user_image_from_file(&connection, image_file, timeout)?;
            }
            CliCommand::UpdateAppImage {
                app_name,
                image_file,
            } => {
                actions::update_app_image_from_file(&connection, app_name, image_file, timeout)?;
            }
            CliCommand::ReportActiveApp { app_name } => {
                actions::report_active_app(&connection, app_name, timeout)?;
            }
            CliCommand::ListAppImages => {
                let app_images_list = actions::retreive_app_images_list(&connection, timeout)?;
                println!("{app_images_list:?}");
            }
        }
    }

    Ok(())
}
