## UI

Using Python 3.10, which can be installed on fedora (and some other dependencies):
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
to generate and install the bindings in the venv, run
```bash
maturin develop
```