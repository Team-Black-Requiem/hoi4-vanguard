import sys
from PyQt6.QtWidgets import QVBoxLayout, QFrame
from PyQt6.QtGui import QPen, QColor
from PyQt6.Qsci import *

class EditorWidget(QFrame):
    def __init__(self):
        super().__init__()

        self.setStyleSheet("QWidget { background-color: #ffeaeaea }")
        self.layout = QVBoxLayout(self)
        self.setLayout(self.layout)
        self.myFont = self.font()
        self.myFont.setPointSize(14)

        self.setupEditor()

    def setupEditor(self):
        self.editor = QsciScintilla()
        self.editor.setText("Hello\n")
        self.editor.append("world")
        self.editor.setUtf8(True)
        self.editor.setFont(self.myFont)
        self.editor.setMarginWidth(0, "0000")
        self.editor.setMarginLineNumbers(0, True)
        self.editor.setMarginsBackgroundColor(QColor('#ffeaeaea')) 
        self.editor.setCaretLineVisible(True)
        self.editor.setCaretLineBackgroundColor(QColor('#e6e6e6'))
        self.editor.setUtf8(True)  # Set encoding to UTF-8
        #lexer = QsciScintillaLexerPython()
        #self.editor.setLexer(lexer)

        self.layout.addWidget(self.editor)