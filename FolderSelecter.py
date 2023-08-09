import os
import subprocess
import platform
import tkinter as tk
from tkinter import filedialog

class FileAndFolderDialogUtility:
    def __init__(self):
        self.root = tk.Tk()
        self.root.withdraw()  # Hide the main window
        self.selected_folder = None  # Initialize the selected_folder variable

    def open_folder_dialog(self, initial_dir):
        folder_path = filedialog.askdirectory(initialdir=initial_dir)
        return folder_path

    def open_file_explorer(self, path):
        if platform.system() == "Windows":
            os.startfile(path)
        elif platform.system() == "Darwin":  # macOS
            subprocess.run(["open", path])
        else:  # Assuming Linux or other Unix-like systems
            subprocess.run(["xdg-open", path])

    def run(self):
        preset_path = r"C:\Users\micks\OneDrive\Documents\GitHub\CG-Black-Requiem"

        self.selected_folder = self.open_folder_dialog(preset_path)

        if self.selected_folder:
            print("Selected Folder:", self.selected_folder)

if __name__ == "__main__":
    dialog_utility = FileAndFolderDialogUtility()
    dialog_utility.run()

    # Access the selected_folder variable outside the class
    if dialog_utility.selected_folder:
        print("Selected Folder (Outside the Class):", dialog_utility.selected_folder)
