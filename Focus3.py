import tkinter as tk
import csv

class DraggableWidget:
    def __init__(self, widget):
        self.widget = widget
        self.widget.bind("<Button-1>", self.on_drag_init)
        self.widget.bind("<B1-Motion>", self.on_drag_motion)
        self.widget.lift()  # Ensure the dragged widget is on top
        self.marked_pointx = 0
        self.marked_pointy = 0

    def on_drag_init(self, event):
        self.marked_pointx = self.widget.winfo_pointerx()
        self.marked_pointy = self.widget.winfo_pointery()

    def on_drag_motion(self, event):
        dx = self.marked_pointx - self.widget.winfo_pointerx()
        dy = self.marked_pointy - self.widget.winfo_pointery()
        cx, cy = self.widget.winfo_x(), self.widget.winfo_y()
        self.widget.place(x=cx - dx, y=cy - dy)
        self.marked_pointx = self.widget.winfo_pointerx()
        self.marked_pointy = self.widget.winfo_pointery()

class Box(tk.Frame):
    def __init__(self, parent, name, x, y):
        super().__init__(parent, bg="white", relief=tk.SOLID, bd=1)
        self.name = name
        self.x = x
        self.y = y
        self.text_boxes = []
        self.collapsed = False

        # Set up the widget
        self.create_widgets()
        self.draggable = DraggableWidget(self)

    def create_widgets(self):
        # Label to display the box name
        self.box_label = tk.Label(self, text=self.name)
        self.box_label.pack()

        # Button to toggle collapse
        self.collapse_button = tk.Button(self, text="Collapse", command=self.toggle_collapse)
        self.collapse_button.pack()

        # Create text boxes
        for _ in range(5):
            box_text = tk.Text(self, height=4, width=15, state=tk.NORMAL)
            box_text.pack(pady=5, padx=5)
            self.text_boxes.append(box_text)

    def toggle_collapse(self):
        if self.collapsed:
            for text_box in self.text_boxes:
                text_box.pack()
            self.collapsed = False
        else:
            for text_box in self.text_boxes:
                text_box.pack_forget()
            self.collapsed = True

def generate_csv():
    # Specify the filename for the CSV file
    filename = "output.csv"
    
    # Open the file in write mode and create a CSV writer
    with open(filename, mode='w', newline='') as file:
        writer = csv.writer(file)
        
        # Write the header row
        header = ["Box Name", "Text Box 1", "Text Box 2", "Text Box 3", "Text Box 4", "Text Box 5"]
        writer.writerow(header)
        
        # Write the data for each box
        for box in boxes_in_viewport:
            data_row = [box.name]
            for text_box in box.text_boxes:
                data_row.append(text_box.get("1.0", "end").strip())
            writer.writerow(data_row)
    
    print(f"CSV file '{filename}' generated successfully.")

def create_new_box(name):
    new_box = Box(viewport, name, 0, 0)
    new_box.pack()
    boxes_in_viewport.append(new_box)

# Make the viewport draggable
def start_viewport_drag(event):
    viewport.scan_mark(event.x, event.y)

def on_viewport_drag(event):
    viewport.scan_dragto(event.x, event.y, gain=1)

if __name__ == "__main__":
    window = tk.Tk()
    window.title("GUI Example")

    # Create the top bar as the first UI element
    top_bar = tk.Frame(window, bg="lightgray")
    top_bar.pack(side=tk.TOP, fill=tk.X)

    # Add a button to generate the CSV file
    generate_button = tk.Button(top_bar, text="Generate CSV", command=generate_csv)
    generate_button.pack(side=tk.RIGHT)

    # Create the left-side bar as the second UI element
    left_bar = tk.Frame(window, bg="lightgray", width=200)
    left_bar.pack(side=tk.LEFT, fill=tk.Y)

    # Add different UI elements (boxes) to the left-side bar
    box1 = Box(left_bar, "Box 1", 0, 0)
    box1.pack(pady=10)

    box2 = Box(left_bar, "Box 2", 0, 0)
    box2.pack(pady=10)
    
    # Create the viewport as the third UI element
    viewport = tk.Canvas(window, bg="blue", width=800, height=600)
    viewport.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

    # Create a list to keep track of the boxes in the viewport
    boxes_in_viewport = []

    # Create the viewport space
    viewport_space = viewport.create_rectangle(0, 0, 800, 600, outline="black")

    # Bind events for creating instances in the viewport
    box1.box_label.bind("<Button-1>", lambda e: create_new_box("Box 1"))
    box2.box_label.bind("<Button-1>", lambda e: create_new_box("Box 2"))
    
    # Make the viewport draggable
    viewport.bind("<ButtonPress-1>", start_viewport_drag)
    viewport.bind("<B1-Motion>", on_viewport_drag)


    # Start the GUI event loop
    window.mainloop()
