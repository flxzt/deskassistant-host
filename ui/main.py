import sys
from PySide6.QtWidgets import QApplication, QStyle

from deskassistant_driver import PyUsbConnection, DeviceStatus

import ui

if __name__ == "__main__":
    qt_app = QApplication(sys.argv)
    qt_app.setStyle("Fusion")

    device_connection = PyUsbConnection.init()
    #device_status = DeviceStatus(device_connection.retreive_device_status(2000))
    #print(device_status.current_epd_page)

    window = ui.AppWindow(device_connection)
    window.resize(800, 600)
    window.show()

    sys.exit(qt_app.exec())
