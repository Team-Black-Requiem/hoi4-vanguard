import os
import subprocess
import platform
import tkinter as tk
from tkinter import filedialog

def open_file_explorer(path):
    if platform.system() == "Windows":
        os.startfile(path)
    elif platform.system() == "Darwin":  # macOS
        subprocess.run(["open", path])
    else:  # Assuming Linux or other Unix-like systems
        subprocess.run(["xdg-open", path])

def open_file_dialog(initial_dir):
    root = tk.Tk()
    root.withdraw()  # Hide the main window

    file_path = filedialog.askopenfilename(initialdir=initial_dir)
    return file_path

def execute_python_file(file_path):
    if file_path:
        try:
            with open(file_path, "r") as file:
                code = file.read()
                exec(code)
        except Exception as e:
            print("Error executing the file:", e)

if __name__ == "__main__":
    preset_path = r"C:\Users\micks\OneDrive\Documents\GitHub\CG-Black-Requiem"
    selected_file = open_file_dialog(preset_path)

    if selected_file:
        print("Selected File:", selected_file)
