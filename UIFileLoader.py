from PyQt6 import QtWidgets, uic
import sys

class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, *args, **kwargs):
        super(MainWindow, self).__init__(*args, **kwargs)
        uic.loadUi('Hearts Of Iron 4 Arsenal.ui', self)

        self.tools = {}  # Dictionary to store open tools

        self.tabWidget.clear()  # Clear any existing tabs

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
