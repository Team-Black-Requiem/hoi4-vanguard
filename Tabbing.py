from PyQt6.QtWidgets import QWidget, QVBoxLayout, QVBoxLayout, QWidget, QFrame
from Editor import EditorWidget
from PyQt6 import uic
from FocusEditor.FocusTreeDesigner import FocusTreeTool, FocusEditorUI
from FocusEditor.FocusTreeDesigner import Viewport

class Tabbing(QWidget):
    def __init__(self, tab):
        super().__init__()
        self.tab = tab
        uic.loadUi('Tabbing.ui', self)

        #Focus Tree Code Signal
        self.viewport = Viewport()
        self.viewport.nodeClicked.connect(self.updateUI)

        # Load the left widget UI into the left container
        self.left_container = self.findChild(QVBoxLayout, 'leftContainer')
        self.loadLeftWidget(self.tab) #Dynamic load

        # Get references to the right widget
        self.right_widget = self.findChild(QFrame, 'FrameOfMind') #Right Widget is text editor
        self.right_widget.hide()

        self.toolButton.clicked.connect(self.toggleRightWidget)            
        self.is_right_widget_visible = False # Track the toggle state
        
        self.setupEditorWidget2()
        self.setupEditorWidget()
        
    def setupEditorWidget(self):
        print("Setting up Editor Widget")
        self.editor_widget = EditorWidget()
        self.right_widget.layout().addWidget(self.editor_widget)

    def setupEditorWidget2(self):
        print("Setting up Editor Widget")
        self.FocusEditorUI = FocusEditorUI()
        self.right_widget.layout().addWidget(self.FocusEditorUI)      

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

    def updateUI(self, selected_node):
        print("updateUI called")
        print("Selected Node Information:")
        print("Focus Name:", selected_node["FocusName"])