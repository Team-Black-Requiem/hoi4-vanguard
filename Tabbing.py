from PyQt6.QtWidgets import QWidget, QVBoxLayout, QVBoxLayout, QWidget, QFrame
from Editor import EditorWidget
from PyQt6 import uic
from FocusTreeDesigner import FocusTreeTool 

class Tabbing(QWidget):
    def __init__(self, tab):
        super().__init__()
        self.tab = tab
        uic.loadUi('Tabbing.ui', self)

        # Load the left widget UI into the left container
        self.left_container = self.findChild(QVBoxLayout, 'leftContainer')
        self.loadLeftWidget(self.tab) #Dynamic load

        # Get references to the right widget
        self.right_widget = self.findChild(QFrame, 'FrameOfMind') #Right Widget is text editor
        self.right_widget.hide()

        self.toolButton.clicked.connect(self.toggleRightWidget)            
        self.is_right_widget_visible = False # Track the toggle state
        
        self.setupEditorWidget()
        
    def setupEditorWidget(self):
        print("Setting up Editor Widget")
        self.editor_widget = EditorWidget()
        self.right_widget.layout().addWidget(self.editor_widget)

    def loadLeftWidget(self, var):
        if var == "Focus Tree Designer":
            #left_ui = uic.loadUi('Focus Tree Designer.ui', self)  # Load the UI for Focus Tree Designer
            left_ui = FocusTreeTool()
            self.left_container.layout().addWidget(left_ui)
        else:
            #left_ui = uic.loadUi('DefaultLeftWidget.ui')  # Load a default UI
            #self.left_container.layout().addWidget(left_ui)
            None

    def toggleRightWidget(self):
        # Toggle the visibility of the right widget
        self.is_right_widget_visible = not self.is_right_widget_visible
        self.right_widget.setVisible(self.is_right_widget_visible)