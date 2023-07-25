from PyQt6 import QtWidgets, uic
import os

class FocusTreeDesignerTool(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        scroll_area = QtWidgets.QScrollArea()
        scroll_area.setWidgetResizable(True)

        tool_content = QtWidgets.QWidget()
        ui_file = os.path.join(os.path.dirname(__file__), "FocusTreeDesigner/FocusToolDesigner.ui")
        uic.loadUi(ui_file, tool_content)

        scroll_area.setWidget(tool_content)

        layout = QtWidgets.QVBoxLayout(self)
        layout.addWidget(scroll_area)
