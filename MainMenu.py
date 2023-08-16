from PyQt6 import QtWidgets, uic
import sys
from FocusTreeDesigner import FocusTreeTool  # Import the FocusTreeTool class

class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, *args, **kwargs):
        super(MainWindow, self).__init__(*args, **kwargs)
        uic.loadUi('Hearts Of Iron 4 Arsenal.ui', self)

        self.tools = {}  # Dictionary to store open tools

        self.tabWidget.clear()

        # Load tool names from text file
        with open('ToolList.txt', 'r') as file:
            tool_names = [line.strip() for line in file]

        # Generate tool buttons based on the loaded tool names
        for tool_name in tool_names:
            tool_button = QtWidgets.QPushButton(tool_name)
            tool_button.clicked.connect(lambda _, btn=tool_button: self.openTool(btn))
            self.verticalLayout.addWidget(tool_button)

        self.tabWidget.tabCloseRequested.connect(self.closeTool)  # Connect the closeTool method to the tabCloseRequested signal

    def openTool(self, button):
        tool_name = button.text()
        print("Loading tool:", tool_name)

        # Create a new instance of the tool widget
        if tool_name == "Focus Tree Designer":  # Special case for FocusTreeTool
            tool_widget = FocusTreeTool()
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
    app = QtWidgets.QApplication(sys.argv)
    main = MainWindow()
    main.show()
    sys.exit(app.exec())

if __name__ == '__main__':
    main()
