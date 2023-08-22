from PyQt6 import QtWidgets, uic, QtCore
import sys
from FocusTreeDesigner import FocusTreeTool  # Import the FocusTreeTool class
from Tabbing import Tabbing

class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, *args, **kwargs):
        super(MainWindow, self).__init__(*args, **kwargs)
        uic.loadUi('Hearts Of Iron 4 Arsenal.ui', self)

        self.tools = {}  # Dictionary to store open tools

        self.tabWidget.clear()

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

        # Create a new instance of the tool widget
        if tool_name == "Focus Tree Designer":  # Special case for FocusTreeTool
            tool_widget = Tabbing(tool_name)
        else:
            tool_widget = QtWidgets.QWidget()

        # Add the tool widget as a tab and store its index in the dictionary
        tool_index = self.tabWidget.addTab(tool_widget, tool_name)

        # Store the tool instance in a list instead of using a dictionary
        if tool_name not in self.tools:
            self.tools[tool_name] = []
        self.tools[tool_name].append(tool_widget)

        # Switch to the selected tool index
        self.tabWidget.setCurrentIndex(tool_index)

    def closeTool(self, index):
        tool_widget = self.tabWidget.widget(index)
        tool_name = self.tabWidget.tabText(index)
        print("Closing tool:", tool_name)

        # Remove the tool instance from the list
        if tool_name in self.tools:
            self.tools[tool_name].remove(tool_widget)
            if len(self.tools[tool_name]) == 0:
                del self.tools[tool_name]

        # Remove the tab
        self.tabWidget.removeTab(index)

def main():
    if hasattr(QtCore.Qt, 'AA_EnableHighDpiScaling'):
        QtWidgets.QApplication.setAttribute(QtCore.Qt.AA_EnableHighDpiScaling, True)
    if hasattr(QtCore.Qt, 'AA_UseHighDpiPixmaps'):
        QtWidgets.QApplication.setAttribute(QtCore.Qt.AA_UseHighDpiPixmaps, True)
    app = QtWidgets.QApplication(sys.argv)
    QtWidgets.QApplication.setStyle("Fusion") #default stylesheet hurrdurr
    main = MainWindow()
    main.show()
    sys.exit(app.exec())

if __name__ == '__main__':
    main()