from PyQt6 import QtWidgets, uic
import sys

class MainWindow(QtWidgets.QMainWindow):
    def __init__(self, *args, **kwargs):
        super(MainWindow, self).__init__(*args, **kwargs)
        uic.loadUi('Hearts Of Iron 4 Arsenal.ui', self)

        self.tabWidget.clear()  # Clear any existing tabs

        # Connect button signal to slot/function
        self.CountryCreatorButton.clicked.connect(lambda: self.buttonClicked(self.CountryCreatorButton))
        self.FocusTreeDesignerButton_2.clicked.connect(lambda: self.buttonClicked(self.FocusTreeDesignerButton_2))
        self.LocalizationButton.clicked.connect(lambda: self.buttonClicked(self.LocalizationButton))
        self.MapDesignerButton.clicked.connect(lambda: self.buttonClicked(self.MapDesignerButton))
        self.TechTreeDesignerButton.clicked.connect(lambda: self.buttonClicked(self.TechTreeDesignerButton))
        self.DataStructureFunctionsButton.clicked.connect(lambda: self.buttonClicked(self.DataStructureFunctionsButton))
        self.UIDesignerButton.clicked.connect(lambda: self.buttonClicked(self.UIDesignerButton))
        self.PeaceConferenceDesignerButton.clicked.connect(lambda: self.buttonClicked(self.PeaceConferenceDesignerButton))
        self.AchievementDesignerButton.clicked.connect(lambda: self.buttonClicked(self.AchievementDesignerButton))
        self.AIButton.clicked.connect(lambda: self.buttonClicked(self.AIButton))
        self.PluginsButton.clicked.connect(lambda: self.buttonClicked(self.PluginsButton))
        self.tabWidget.tabCloseRequested.connect(self.tabCloseRequested)

    def buttonClicked(self, button):
        button_text = button.text()

        tab_index = self.tabWidget.addTab(QtWidgets.QWidget(), button_text)
        self.tabWidget.setCurrentIndex(tab_index)

        if button is self.CountryCreatorButton:
            self.countryCreatorClicked()
        elif button is self.FocusTreeDesignerButton_2:
            self.focusTreeDesignerClicked()
        elif button is self.LocalizationButton:
            self.localizationToolsClicked()
        elif button is self.MapDesignerButton:
            self.mapDesignerClicked()
        elif button is self.TechTreeDesignerButton:
            self.techTreeDesignerClicked()
        elif button is self.DataStructureFunctionsButton:
            self.dataStructureFunctionsClicked()
        elif button is self.UIDesignerButton:
            self.uiDesignerClicked()
        elif button is self.PeaceConferenceDesignerButton:
            self.peaceConferenceDesignerClicked()
        elif button is self.AchievementDesignerButton:
            self.achievementsDesignerClicked()
        elif button is self.AIButton:
            self.aiToolsClicked()
        elif button is self.PluginsButton:
            self.pluginsClicked()

    def countryCreatorClicked(self):
        print("CountryCreatorButton clicked")


    def focusTreeDesignerClicked(self):
        print("FocusTreeDesignerButton_2 clicked")


    def localizationToolsClicked(self):
        print("LocalizationButton clicked")


    def mapDesignerClicked(self):
        print("MapDesignerButton clicked")

    def techTreeDesignerClicked(self):
        print("TechTreeDesignerButton clicked")


    def dataStructureFunctionsClicked(self):
        print("DataStructureFunctionsButton clicked")


    def uiDesignerClicked(self):
        print("UIDesignerButton clicked")


    def peaceConferenceDesignerClicked(self):
        print("PeaceConferenceDesignerButton clicked")


    def achievementsDesignerClicked(self):
        print("AchievementDesignerButton clicked")


    def aiToolsClicked(self):
        print("AIButton clicked")


    def pluginsClicked(self):
        print("PluginsButton clicked")


    def tabCloseRequested(self, index):
        print(f"Closing tab {index}")
        self.tabWidget.removeTab(index)

    #Load Program
if __name__ == '__main__':
    app = QtWidgets.QApplication(sys.argv)
    main = MainWindow()
    main.show()
    sys.exit(app.exec())
