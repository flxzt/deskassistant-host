import sys
from PySide6.QtWidgets import QApplication, QStyle

from deskassistant_driver import PyUsbConnection, DeviceStatus

import ui

if __name__ == "__main__":
    qt_app = QApplication(sys.argv)
    qt_app.setStyle("Fusion")

    device_connection = PyUsbConnection.new()

    window = ui.AppWindow(device_connection)
    window.resize(800, 600)
    window.show()

    sys.exit(qt_app.exec())
