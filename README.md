# Deskassistant

## Client

The client is a STM32-F4 MCU (STM32-F446-RE). Connected to it are:
- waveshare EPD module (through SPI)
- a separate USB-C port
- mSD port (for storing data and configuration permanently)

The client is able to a display a custom user image that is sent to it.
It displays different custom images for active apps that are reported to it.
All images are saved permanently on the mSD.
It is able to store/load settings to/from a JSON file on the mSD.
In order to communicate with the host it implements a custom USB device class.
The firmware utilizes the [fatFS](http://elm-chan.org/fsw/ff/00index_e.html), [parson](https://github.com/kgabis/parson) and [tinyusb](https://github.com/hathach/tinyusb) libraries.

## Host

The host application for the deskassistant project. It consists of:
- The backend, written in Rust, that communicates with the client through a custom USB device class.
- A UI written with Python and QT, interfacing with the Rust backend through [py03](https://crates.io/crates/pyo3).
- A CLI written in Rust

## UI/CLI Capabilities

- Refresh Display - refreshes the EPP
- Read Status - Read the status of the client
- Switch Page - Switches through the different pages
- Update User Image - send/update the user image through USB
- Report active app - the host automatically reports the current active (focused) app on the host to the client

## Dependencies

System dependencies:
```bash
sudo dnf install python3.10 patchelf
```

Python dependencies:
```bash
pip install python-libxdo proc
```

# Setup
Bindings are generated with the `pyo3` crate and the `maturin` tool.

A virtual env is used to install packages.

```bash
python3.10 -m venv .env
```

To activate the venv:
```bash
source .env/bin/activate
```

Then install maturin and the other python dependencies
```bash
pip install maturin PySide6
```

And init:
```bash
cd driver
maturin init
```

## Regenerate Bindings

To generate and install the bindings in the `venv`, run
```bash
maturin develop
```
