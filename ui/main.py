import sys
from PySide6.QtWidgets import QApplication, QStyle
from PySide6.QtCore import QTimer, Qt

import ui

if __name__ == "__main__":
    qt_app = QApplication(sys.argv)
    qt_app.setStyle("Fusion")

    app_window = ui.AppWindow()
    app_window.resize(800, 600)
    app_window.show()

    sys.exit(qt_app.exec())
