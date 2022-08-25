import sys
from PySide6.QtWidgets import QApplication, QStyle
from PySide6.QtCore import QTimer, Qt

from deskassistant_driver import PyUsbConnection, DeviceStatus

import ui

if __name__ == "__main__":
    qt_app = QApplication(sys.argv)
    qt_app.setStyle("Fusion")

    device_connection = PyUsbConnection.new()

    app_window = ui.AppWindow(device_connection)
    app_window.resize(800, 600)
    app_window.show()

    sys.exit(qt_app.exec())
