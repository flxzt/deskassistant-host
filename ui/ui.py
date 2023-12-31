import os

from enum import Enum
from re import I
from typing import List
from PySide6.QtCore import Qt, Slot, Signal, QTimer, QMargins
from PySide6.QtGui import *
from PySide6.QtWidgets import *

from deskassistant_driver import EpdPage, PyUsbConnection, DeviceStatus

import core

app_name = "Deskassistant"


class ConnectedState(Enum):
    DISCONNECTED = 0
    CONNECTED = 1


class ImportFileDialog(QDialog):
    def __init__(self, parent=None):
        super(ImportFileDialog, self).__init__(parent)
        self.setWindowTitle("Import file")


class AppWindow(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle(app_name)

        self.active_app_name = core.get_active_app_name()

        self.device_connection = PyUsbConnection.new()

        # Menu
        self.menu = self.menuBar()
        self.menu_general = self.menu.addMenu("General")

        # Exit QAction
        exit_action = QAction("Exit", self)
        exit_action.setShortcut(QKeySequence.Quit)
        exit_action.triggered.connect(self.close)

        # Connect action
        connect_action = QAction("Connect", self)
        connect_action.triggered.connect(self.connection_handle_events)

        self.menu_general.addAction(connect_action)
        self.menu_general.addAction(exit_action)

        # Status Bar
        self.status = self.statusBar()

        self.central_widget = AppCentralWidget(self)
        self.setCentralWidget(self.central_widget)

        # Report active app timer
        active_app_check = QTimer(self)
        active_app_check.setInterval(500)
        active_app_check.timeout.connect(self.report_active_app_exe_name)
        active_app_check.start()

        # Device connection event timer
        device_connection_event_timer = QTimer(self)
        device_connection_event_timer.setInterval(200)
        device_connection_event_timer.timeout.connect(self.connection_handle_events)
        device_connection_event_timer.start()

    @Slot()
    def connection_handle_events(self):
        self.device_connection.handle_events()

        if self.device_connection.is_connected():
            self.central_widget.set_view(1)
        else:
            self.central_widget.set_view(0)

    @Slot()
    def report_active_app_exe_name(self):
        active_app_name = core.get_active_app_name()
        if self.active_app_name != active_app_name:
            self.active_app_name = active_app_name

            if self.active_app_name != None:
                self.central_widget.status_widget.UpdateAppStatus()

                if self.device_connection.is_connected():
                    self.device_connection.report_active_app_name(active_app_name, 5000)


class AppCentralWidget(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()

        self.app_window = app_window
        self.disconnected_view = AppDisconnectedView(app_window)
        self.connected_view = AppConnectedView(app_window)

        self.status_widget = HostStatusWidget(app_window)

        self.stack = QStackedWidget()
        self.stack.addWidget(self.disconnected_view)
        self.stack.addWidget(self.connected_view)

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.status_widget)
        self.layout.addWidget(self.stack)

        self.set_view(1)

    def set_view(self, index):
        self.stack.setCurrentIndex(index)


class HostStatusWidget(QGroupBox):
    def __init__(self, app_window: AppWindow):
        super().__init__()

        self.setTitle("Host Status")

        self.app_window = app_window
        self.active_app_label = QLabel(self.app_window.active_app_name)

        self.active_app_container = QWidget()
        self.active_app_container.layout = QHBoxLayout(self.active_app_container)
        self.active_app_container.layout.addWidget(QLabel("active app:"))
        self.active_app_container.layout.addWidget(self.active_app_label)

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.active_app_container)

    @Slot()
    def UpdateAppStatus(self):
        self.active_app_label.setText(self.app_window.active_app_name)


class AppDisconnectedView(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()

        self.app_window = app_window
        self.text = QLabel("Disconnected.", alignment=(Qt.AlignCenter | Qt.AlignTop))

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
            lambda: self.SwitchPage(EpdPage.Overview)
        )
        self.switch_page_appscreen_button.clicked.connect(
            lambda: self.SwitchPage(EpdPage.AppScreen)
        )
        self.switch_page_userimage_button.clicked.connect(
            lambda: self.SwitchPage(EpdPage.UserImage)
        )

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
        if self.app_window.device_connection.is_connected():
            self.app_window.status.showMessage(f"Switching to page {page}", 2000)
            self.app_window.device_connection.switch_page(page, 5000)
            self.app_window.central_widget.connected_view.status_page.StatusRefresh()

    @Slot()
    def DisplayRefresh(self):
        if self.app_window.device_connection.is_connected():
            self.app_window.status.showMessage("Refresh display", 2000)
            self.app_window.device_connection.refresh_display(5000)


class StatusPage(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()
        self.app_window = app_window

        self.status_label = QLabel("", alignment=Qt.AlignLeft)

        self.status_refresh_button = QPushButton(
            icon=QIcon(QApplication.style().standardIcon(QStyle.SP_BrowserReload))
        )
        self.status_refresh_button.setToolTip("Refresh Status")
        self.status_refresh_button.clicked.connect(self.StatusRefresh)

        self.layout = QHBoxLayout(self)
        self.layout.addWidget(self.status_label, alignment=(Qt.AlignLeft | Qt.AlignTop))
        self.layout.addWidget(
            self.status_refresh_button, alignment=(Qt.AlignRight | Qt.AlignTop)
        )
        self.layout.setStretch(0, 1)
        self.layout.setStretch(1, 0)

        status_refresh_timer = QTimer(self)
        status_refresh_timer.setInterval(500)
        status_refresh_timer.timeout.connect(self.StatusRefresh)
        status_refresh_timer.start()

    @Slot()
    def StatusRefresh(self):
        if self.app_window.device_connection.is_connected():
            device_status = self.app_window.device_connection.retreive_device_status(
                5000
            )

            status_text = f"""
<h3>Device Status</h3>
<b>Current EPD Page:</b> {device_status.current_epd_page}<br>
"""

            app_images_list = (
                self.app_window.device_connection.retreive_app_images_list(5000)
            )

            status_text += "<b>App Images:</b><br>"

            for app_image_entry in app_images_list:
                status_text += f"{app_image_entry}<br>"

            self.app_window.central_widget.connected_view.status_page.status_label.setText(
                status_text
            )

            self.app_window.status.showMessage("Refresh Device status", 2000)


class EditPage(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()
        self.app_window = app_window
        self.image_file: str | None = None

        self.scene = QGraphicsScene()
        self.pixmapitem = self.scene.addPixmap(QPixmap())
        self.scene.setSceneRect(0, 0, 400, 300)

        self.graphics_view = QGraphicsView(self.scene)

        self.pick_image_dialog_button = QPushButton("Pick Image")
        self.pick_image_dialog_button.clicked.connect(
            self.__onImportFileDialogButtonClicked
        )

        self.send_user_image_button = QPushButton("Send as User Image")
        self.send_user_image_button.clicked.connect(self.SendUserImageFile)

        self.app_image_name_edit = QLineEdit()
        self.app_image_name_edit.setPlaceholderText("App name for image")
        self.app_image_name_edit.setContentsMargins(QMargins(30, 0, 0, 0))

        self.send_app_image_button = QPushButton("Send as App Image")
        self.send_app_image_button.clicked.connect(self.SendAppImageFile)

        self.edit_controls_container = QWidget()
        self.edit_controls_container.layout = QHBoxLayout(self.edit_controls_container)
        self.edit_controls_container.layout.addWidget(self.send_user_image_button)
        self.edit_controls_container.layout.addWidget(self.app_image_name_edit)
        self.edit_controls_container.layout.addWidget(self.send_app_image_button)
        self.edit_controls_container.layout.setStretch(0, 0)
        self.edit_controls_container.layout.setStretch(1, 1)
        self.edit_controls_container.layout.setStretch(2, 0)

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(
            self.pick_image_dialog_button, alignment=(Qt.AlignLeft | Qt.AlignTop)
        )
        self.layout.addWidget(
            self.graphics_view, alignment=(Qt.AlignCenter | Qt.AlignTop)
        )
        self.layout.addWidget(self.edit_controls_container)
        self.layout.setStretch(0, 0)
        self.layout.setStretch(1, 1)
        self.layout.setStretch(2, 0)

    def __updateScenePixmap(self):
        new_pixmap = QPixmap(self.image_file)
        self.pixmapitem.setPixmap(new_pixmap)
        self.scene.update()

    def __onImportFileDialogButtonClicked(self):
        file_path, filter = QFileDialog.getOpenFileName(
            parent=self, caption="Open file", dir=".", filter="(*.jpg *.png)"
        )
        self.image_file = file_path
        self.app_image_name_edit.clear()
        self.app_image_name_edit.insert(
            os.path.splitext(os.path.basename(file_path))[0]
        )
        self.__updateScenePixmap()

    @Slot()
    def SendUserImageFile(self):
        if self.image_file != None:
            if self.app_window.device_connection.is_connected():
                self.app_window.device_connection.convert_send_user_image_from_file(
                    self.image_file, 5000
                )

    @Slot()
    def SendAppImageFile(self):
        if self.image_file != None:
            if self.app_window.device_connection.is_connected():
                self.app_window.device_connection.convert_send_app_image_from_file(
                    self.app_image_name_edit.text(), self.image_file, 5000
                )
