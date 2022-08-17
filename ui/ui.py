from PySide6.QtCore import Qt, Slot, Signal
from PySide6.QtGui import *
from PySide6.QtWidgets import *

from deskassistant_driver import EpdPage

app_name = "Deskassistant"


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
        self.menu_general.addAction(exit_action)

        # Status Bar
        self.status = self.statusBar()

        self.central_widget = AppCentralWidget(self)
        self.setCentralWidget(self.central_widget)


class AppCentralWidget(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()

        self.app_window = app_window

        self.edit_page = EditPage(app_window)
        self.status_page = StatusPage(app_window)

        self.tab_area = QTabWidget()
        self.tab_area.addTab(self.status_page, "Status")
        self.tab_area.addTab(self.edit_page, "Edit")

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.tab_area)


class StatusPage(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()
        self.app_window = app_window

        self.status_label = QLabel("", alignment=Qt.AlignCenter)

        self.refresh_button = QPushButton(icon=QIcon(
            QApplication.style().standardIcon(QStyle.SP_BrowserReload)))
        self.refresh_button.clicked.connect(self.StatusRefreshRequest)
        self.switch_page_first_button = QPushButton("first")
        self.switch_page_second_button = QPushButton("second")
        self.switch_page_third_button = QPushButton("third")

        self.switch_page_first_button.clicked.connect(
            lambda: self.__SwitchPage(EpdPage.First))
        self.switch_page_second_button.clicked.connect(
            lambda: self.__SwitchPage(EpdPage.Second))
        self.switch_page_third_button.clicked.connect(
            lambda: self.__SwitchPage(EpdPage.Third))

        self.layout = QGridLayout(self)
        self.layout.addWidget(self.status_label, 0, 0, 1,
                              2, alignment=(Qt.AlignLeft | Qt.AlignTop))
        self.layout.addWidget(self.refresh_button, 0, 2,
                              alignment=(Qt.AlignRight | Qt.AlignTop))
        self.layout.addWidget(self.switch_page_first_button, 1, 0,
                              alignment=Qt.AlignBottom)
        self.layout.addWidget(self.switch_page_second_button, 1, 1,
                              alignment=Qt.AlignBottom)
        self.layout.addWidget(self.switch_page_third_button, 1, 2,
                              alignment=Qt.AlignBottom)
        self.layout.setRowStretch(0, 10)
        self.layout.setRowStretch(1, 0)

        self.StatusRefreshRequest()

    @Slot()
    def StatusRefreshRequest(self):
        #device_status = self.app_window.device_connection.retreive_device_status(5000)
        #current_epd_page = device_status.current_epd_page
        current_epd_page = int(EpdPage.First)

        self.status_label.setText(f"""
<h3>Device Status</h3><br>
<b>current Page:</br> {current_epd_page}
        """)

        self.app_window.status.showMessage("Status refreshed", 2000)

    def __SwitchPage(self, page: EpdPage):
        print(f"Switching to page {page}")
        self.app_window.device_connection.switch_page(page)


class EditPage(QWidget):
    def __init__(self, app_window: AppWindow):
        super().__init__()
        self.app_window = app_window

        self.file_label = QLabel("Edit", alignment=Qt.AlignCenter)
        self.import_file_dialog_button = QPushButton("Import File")
        self.import_file_dialog_button.clicked.connect(self.__onImportFileDialogButtonClicked)

        self.layout = QVBoxLayout(self)
        self.layout.addWidget(self.file_label)
        self.layout.addWidget(self.import_file_dialog_button)

    def __onImportFileDialogButtonClicked(self):
        filename, filter = QFileDialog.getOpenFileName(
            parent=self, caption="Open file", dir=".", filter="(*.jpg)")
        self.file_label.setText(filename)
