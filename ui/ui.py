from enum import Enum
from PySide6.QtCore import Qt, Slot, Signal
from PySide6.QtGui import *
from PySide6.QtWidgets import *

from deskassistant_driver import EpdPage

app_name = "Deskassistant"


class ConnectedState(Enum):
    DISCONNECTED = 0
    CONNECTED = 1


@Slot()
def say_hello():
    print("Button clicked, hello!")


class ImportFileDialog(QDialog):

    def __init__(self, parent=None):
        super(ImportFileDialog, self).__init__(parent)
        self.setWindowTitle("Import file")


class AppWindow(QMainWindow):
    def __init__(self, device_connection):
        super().__init__()
        self.setWindowTitle(app_name)
        self.device_connection = device_connection

        # Menu
        self.menu = self.menuBar()
        self.menu_general = self.menu.addMenu("General")

        # Exit QAction
        exit_action = QAction("Exit", self)
        exit_action.setShortcut(QKeySequence.Quit)
        exit_action.triggered.connect(self.close)

        # Connect action
        connect_action = QAction("Connect", self)
        connect_action.triggered.connect(self.open_device)

        self.menu_general.addAction(connect_action)
        self.menu_general.addAction(exit_action)

        # Status Bar
        self.status = self.statusBar()

        self.central_widget = AppCentralWidget(self)
        self.setCentralWidget(self.central_widget)

        self.open_device()

    @Slot()
    def open_device(self):
        try:
            self.device_connection.open()
        except:
            self.status.showMessage("Could not connect to device.", 2000)
            self.central_widget.set_view(0)
        else:
            self.status.showMessage("Connected successfully.", 2000)
            self.central_widget.connected_view.status_page.StatusRefresh()
            self.central_widget.set_view(1)


class AppCentralWidget(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()

        self.app_window = app_window
        self.disconnected_view = AppDisconnectedView(app_window)
        self.connected_view = AppConnectedView(app_window)

        self.stack = QStackedWidget()
        self.stack.addWidget(self.disconnected_view)
        self.stack.addWidget(self.connected_view)

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.stack)

        self.set_view(1)

    def set_view(self, index):
        self.stack.setCurrentIndex(index)


class AppDisconnectedView(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()

        self.app_window = app_window
        self.text = QLabel("Disconnected.", alignment=(
            Qt.AlignCenter | Qt.AlignTop))

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.text)


class AppConnectedView(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()

        self.app_window = app_window

        self.device_controls = DeviceControls(app_window)
        self.edit_page = EditPage(app_window)
        self.status_page = StatusPage(app_window)

        self.tab_area = QTabWidget()
        self.tab_area.addTab(self.status_page, "Status")
        self.tab_area.addTab(self.edit_page, "Edit")

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.device_controls)
        self.layout.addWidget(self.tab_area)


class DeviceControls(QGroupBox):
    def __init__(self, app_window: AppWindow):
        super().__init__()
        self.app_window = app_window
        self.setTitle("Device controls")

        self.switch_page_label = QLabel("Pages:")
        self.switch_page_overview_button = QPushButton("Overview")
        self.switch_page_appscreen_button = QPushButton("App Screen")
        self.switch_page_userimage_button = QPushButton("User Image")

        self.switch_page_overview_button.clicked.connect(
            lambda: self.SwitchPage(EpdPage.Overview))
        self.switch_page_appscreen_button.clicked.connect(
            lambda: self.SwitchPage(EpdPage.AppScreen))
        self.switch_page_userimage_button.clicked.connect(
            lambda: self.SwitchPage(EpdPage.UserImage))

        self.disp_refresh_button = QPushButton("Refresh Display")
        self.disp_refresh_button.clicked.connect(self.DisplayRefresh)

        self.switch_page_container = QWidget()
        self.switch_page_container.layout = QHBoxLayout(self.switch_page_container)
        self.switch_page_container.layout.addWidget(self.switch_page_label)
        self.switch_page_container.layout.addWidget(self.switch_page_overview_button)
        self.switch_page_container.layout.addWidget(self.switch_page_appscreen_button)
        self.switch_page_container.layout.addWidget(self.switch_page_userimage_button)

        self.misc_container = QWidget()
        self.misc_container.layout = QHBoxLayout(self.misc_container)
        self.misc_container.layout.addWidget(self.disp_refresh_button)

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.misc_container, alignment=(Qt.AlignRight))
        self.layout.addWidget(self.switch_page_container)

    @Slot()
    def SwitchPage(self, page: EpdPage):
        self.app_window.status.showMessage(f"Switching to page {page}", 2000)
        self.app_window.device_connection.switch_page(page, 5000)
        self.app_window.central_widget.connected_view.status_page.StatusRefresh()

    @Slot()
    def DisplayRefresh(self):
        self.app_window.status.showMessage("Refresh display", 2000)
        self.app_window.device_connection.refresh_display(5000)

class StatusPage(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()
        self.app_window = app_window

        self.status_label = QLabel("", alignment=Qt.AlignLeft)

        self.status_refresh_button = QPushButton(icon=QIcon(
            QApplication.style().standardIcon(QStyle.SP_BrowserReload)))
        self.status_refresh_button.setToolTip("Refresh Status")
        self.status_refresh_button.clicked.connect(
            self.StatusRefresh)


        self.layout = QHBoxLayout(self)
        self.layout.addWidget(self.status_label,
                              alignment=(Qt.AlignLeft | Qt.AlignTop))
        self.layout.addWidget(self.status_refresh_button, alignment=(Qt.AlignRight | Qt.AlignTop))
        self.layout.setStretch(0, 10)
        self.layout.setStretch(1, 0)

    @Slot()
    def StatusRefresh(self):
        device_status = self.app_window.device_connection.retreive_device_status(
            5000)

        self.app_window.central_widget.connected_view.status_page.status_label.setText(f"""
<h3>Device Status</h3><br>
<b>Current Page:</b> {device_status.current_epd_page}
        """)

        self.app_window.status.showMessage("Refresh Device status", 2000)


class EditPage(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()
        self.app_window = app_window

        self.file_label = QLabel("Edit", alignment=Qt.AlignCenter)
        self.import_file_dialog_button = QPushButton("Pick and send Image")
        self.import_file_dialog_button.clicked.connect(
            self.__onImportFileDialogButtonClicked)

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.file_label)
        self.layout.addWidget(self.import_file_dialog_button)

    def __onImportFileDialogButtonClicked(self):
        file_path, filter = QFileDialog.getOpenFileName(
            parent=self, caption="Open file", dir=".", filter="(*.jpg *.png)")
        self.file_label.setText(file_path)

        self.app_window.device_connection.send_image(file_path, 5000)
