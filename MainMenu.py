from PyQt6 import QtWidgets, uic
import sys
from FocusTreeDesigner import FocusTreeTool  # Import the FocusTreeTool class

class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, *args, **kwargs):
        super(MainWindow, self).__init__(*args, **kwargs)
        uic.loadUi('Hearts Of Iron 4 Arsenal.ui', self)

        self.tools = {}  # Dictionary to store open tools

        self.tabWidget.clear()

        # Create a QStackedWidget to hold the tool widgets
        self.stacked_widget = QtWidgets.QStackedWidget()
        self.verticalLayout.addWidget(self.stacked_widget)


        # Connect button signals to slots/functions
        self.CountryCreatorButton.clicked.connect(lambda: self.openTool(self.CountryCreatorButton))
        self.FocusTreeDesignerButton_2.clicked.connect(lambda: self.openTool(self.FocusTreeDesignerButton_2))  # Button for FocusTreeTool
        self.LocalizationButton.clicked.connect(lambda: self.openTool(self.LocalizationButton))
        self.MapDesignerButton.clicked.connect(lambda: self.openTool(self.MapDesignerButton))
        self.TechTreeDesignerButton.clicked.connect(lambda: self.openTool(self.TechTreeDesignerButton))
        self.DataStructureFunctionsButton.clicked.connect(lambda: self.openTool(self.DataStructureFunctionsButton))
        self.UIDesignerButton.clicked.connect(lambda: self.openTool(self.UIDesignerButton))
        self.PeaceConferenceDesignerButton.clicked.connect(lambda: self.openTool(self.PeaceConferenceDesignerButton))
        self.AchievementDesignerButton.clicked.connect(lambda: self.openTool(self.AchievementDesignerButton))
        self.AIButton.clicked.connect(lambda: self.openTool(self.AIButton))
        self.PluginsButton.clicked.connect(lambda: self.openTool(self.PluginsButton))
        self.tabWidget.tabCloseRequested.connect(self.closeTool)  # Connect the closeTool method to the tabCloseRequested signal

    def openTool(self, button):
        tool_name = button.text()
        print("Loading tool:", tool_name)

        # Check if the tool is already open
        if tool_name in self.tools:
            # If already open, select its tool index
            tool_index = self.tools[tool_name]
        else:
            # If not open, create a new instance of the tool widget and add it to the stacked widget
            if tool_name == "Focus Tree Designer":  # Special case for FocusTreeTool
                tool_widget = FocusTreeTool()
            else:
                tool_widget = QtWidgets.QWidget()
            tool_index = self.stacked_widget.addWidget(tool_widget)
            self.tools[tool_name] = tool_index

        # Switch to the selected tool index
        self.stacked_widget.setCurrentIndex(tool_index)

    def closeTool(self, index):
        tool_name = self.tabWidget.tabText(index)
        print("Closing tool:", tool_name)
        tool_index = self.tools.pop(tool_name, None)
        if tool_index is not None:
            self.stacked_widget.removeWidget(self.stacked_widget.widget(tool_index))
        self.tabWidget.removeTab(index)

def main():
    app = QtWidgets.QApplication(sys.argv)
    main = MainWindow()
    main.show()
    sys.exit(app.exec())

if __name__ == '__main__':
    main()
