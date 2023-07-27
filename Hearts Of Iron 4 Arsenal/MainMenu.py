from PyQt6 import QtWidgets, uic
import sys
from FocusTreeDesigner.FocusToolDesigner import FocusTreeDesignerTool

class FocusTreeTool(QtWidgets.QWidget):
    def __init__(self, parent=None):
        super().__init__(parent)
        scroll_area = QtWidgets.QScrollArea()
        scroll_area.setWidgetResizable(True)

        tool_content = QtWidgets.QWidget()
        uic.loadUi("FocusToolDesigner.ui", tool_content)

        scroll_area.setWidget(tool_content)

        layout = QtWidgets.QVBoxLayout(self)
        layout.addWidget(scroll_area)


class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, *args, **kwargs):
        super(MainWindow, self).__init__(*args, **kwargs)
        uic.loadUi('Hearts Of Iron 4 Arsenal.ui', self)

        self.tools = {}  # Dictionary to store open tools

        self.tabWidget.clear()

        # Connect button signals to slots/functions
        self.CountryCreatorButton.clicked.connect(lambda: self.openTool(self.CountryCreatorButton))
        self.FocusTreeDesignerButton_2.clicked.connect(lambda: self.openTool(self.FocusTreeDesignerButton_2))
        self.LocalizationButton.clicked.connect(lambda: self.openTool(self.LocalizationButton))
        self.MapDesignerButton.clicked.connect(lambda: self.openTool(self.MapDesignerButton))
        self.TechTreeDesignerButton.clicked.connect(lambda: self.openTool(self.TechTreeDesignerButton))
        self.DataStructureFunctionsButton.clicked.connect(lambda: self.openTool(self.DataStructureFunctionsButton))
        self.UIDesignerButton.clicked.connect(lambda: self.openTool(self.UIDesignerButton))
        self.PeaceConferenceDesignerButton.clicked.connect(lambda: self.openTool(self.PeaceConferenceDesignerButton))
        self.AchievementDesignerButton.clicked.connect(lambda: self.openTool(self.AchievementDesignerButton))
        self.AIButton.clicked.connect(lambda: self.openTool(self.AIButton))
        self.PluginsButton.clicked.connect(lambda: self.openTool(self.PluginsButton))

        self.tabWidget.tabCloseRequested.connect(self.closeTool)

    def openTool(self, button):
        tool_name = button.text()
        print("Loading tool:", tool_name)

        # Check if the tool is already open
        if tool_name in self.tools:
            # If already open, select its tab
            index = self.tabWidget.indexOf(self.tools[tool_name])
            self.tabWidget.setCurrentIndex(index)
        else:
            # If not open, create a new tab and add the tool
            tool_widget = QtWidgets.QWidget()  # Create a new instance of QWidget
            tab_index = self.tabWidget.addTab(tool_widget, tool_name)
            self.tools[tool_name] = tool_widget
            self.tabWidget.setCurrentIndex(tab_index)

            if tool_name == "Focus Tree Designer":
                # Create an instance of the FocusTreeTool widget
                focus_tree_tool = FocusTreeTool(tool_widget)
                layout = QtWidgets.QVBoxLayout(tool_widget)
                layout.addWidget(focus_tree_tool)
                tool_widget.setLayout(layout)

                # Load FocusTreeTool.ui into the Viewport
                focus_tree_tool_ui = uic.loadUi("FocusToolDesigner.ui")
                self.Viewport.setLayout(QtWidgets.QVBoxLayout())
                self.Viewport.layout().addWidget(focus_tree_tool_ui)

    def closeTool(self, index):
        tool_widget = self.tabWidget.widget(index)
        tool_name = self.tabWidget.tabText(index)
        print("Closing tool:", tool_name)
        self.tools.pop(tool_name, None)  # Remove the tool from the dictionary
        self.tabWidget.removeTab(index)

def main():
    app = QtWidgets.QApplication(sys.argv)
    main = MainWindow()
    main.show()
    sys.exit(app.exec())

if __name__ == '__main__':
    main()
