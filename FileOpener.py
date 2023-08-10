import os
import subprocess
import platform
import tkinter as tk
from tkinter import filedialog

class FileSelectorUtility:
    def __init__(self):
        self.root = tk.Tk()
        self.root.withdraw()  # Hide the main window
        self.selected_file = None  # Initialize the selected_file variable

    def open_file_explorer(self, path):
        if platform.system() == "Windows":
            os.startfile(path)
        elif platform.system() == "Darwin":  # macOS
            subprocess.run(["open", path])
        else:  # Assuming Linux or other Unix-like systems
            subprocess.run(["xdg-open", path])

    def select_file(self, initial_dir):
        file_path = filedialog.askopenfilename(initialdir=initial_dir)
        self.selected_file = file_path
        return self.selected_file

if __name__ == "__main__":
    file_selector = FileSelectorUtility()
    selected_file = file_selector.select_file(r"C:\Users\micks\OneDrive\Documents\GitHub\CG-Black-Requiem")

    if selected_file:
        print("Selected File (Inside Module):", selected_file)
        
