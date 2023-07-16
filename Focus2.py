import tkinter as tk
import csv

class Box:
    def __init__(self, name):
        self.name = name
        self.x = None
        self.y = None
        self.text_boxes = []
        self.collapsed = False

def create_instance(item):
    box = Box(item)
    boxes_in_viewport.append(box)
    box.frame = tk.Frame(viewport, bg="white", relief=tk.SOLID, bd=1)
    box.frame.bind("<ButtonPress-1>", lambda event, b=box: start_drag(event, b))
    box.frame.bind("<B1-Motion>", on_drag)
    box.frame.bind("<ButtonRelease-1>", on_drop)
    box.frame.pack(pady=10)
    
    box_label = tk.Label(box.frame, text=box.name)
    box_label.pack()

    collapse_button = tk.Button(box.frame, text="Collapse", command=lambda b=box: toggle_collapse(b))
    collapse_button.pack()

    drag_button = tk.Button(box.frame, text="Drag")
    drag_button.bind("<ButtonPress-1>", lambda event, b=box: start_box_drag(event, b))
    drag_button.bind("<B1-Motion>", on_box_drag)
    drag_button.bind("<ButtonRelease-1>", on_box_drop)
    drag_button.pack()

    for _ in range(5):
        box_text = tk.Text(box.frame, height=4, width=15, state=tk.NORMAL)
        box_text.pack(pady=5, padx=5)
        box.text_boxes.append(box_text)

def toggle_collapse(box):
    if box.collapsed:
        for text_box in box.text_boxes:
            text_box.pack()
        box.collapsed = False
    else:
        for text_box in box.text_boxes:
            text_box.pack_forget()
        box.collapsed = True
    
def start_drag(event, box):
    global drag_data
    drag_data = {'box': box, 'x': event.x, 'y': event.y}
    
def on_drag(event):
    if 'drag_data' in globals():
        box = drag_data['box']
        if box.x is None or box.y is None:
            box.x = event.x
            box.y = event.y
        else:
            dx = event.x - drag_data['x']
            dy = event.y - drag_data['y']
            x = box.x + dx
            y = box.y + dy
            viewport.move(box.frame, dx, dy)
            box.x = x
            box.y = y

def start_box_drag(event, box):
    global box_drag_data
    box_drag_data = {'box': box, 'x': event.x, 'y': event.y}
    
def on_box_drag(event):
    if 'box_drag_data' in globals():
        box = box_drag_data['box']
        dx = event.x - box_drag_data['x']
        dy = event.y - box_drag_data['y']
        x = box.x + dx
        y = box.y + dy
        viewport.move(box.frame, dx, dy)
        box.x = x
        box.y = y
    
def on_box_drop(event):
    if 'box_drag_data' in globals():
        del globals()['box_drag_data']

def on_drop(event):
    if 'drag_data' in globals():
        del globals()['drag_data']
    
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

# Create the main window
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
box1 = tk.Frame(left_bar, bg="white", relief=tk.SOLID, bd=1)
box1.pack(pady=10)

box1_label = tk.Label(box1, text="Box 1")
box1_label.pack()

for _ in range(5):
    box1_text = tk.Text(box1, height=4, width=15, state=tk.NORMAL)
    box1_text.pack(pady=5, padx=5)

box2 = tk.Frame(left_bar, bg="white", relief=tk.SOLID, bd=1)
box2.pack(pady=10)

box2_label = tk.Label(box2, text="Box 2")
box2_label.pack()

for _ in range(5):
    box2_text = tk.Text(box2, height=4, width=15, state=tk.NORMAL)
    box2_text.pack(pady=5, padx=5)

# Create the viewport as the third UI element
viewport = tk.Canvas(window, bg="blue", width=800, height=600)
viewport.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

# Create a list to keep track of the boxes in the viewport
boxes_in_viewport = []

# Create the viewport space
viewport_space = viewport.create_rectangle(0, 0, 800, 600, outline="black")

# Bind events for creating instances in the viewport
box1_label.bind("<Button-1>", lambda e: create_instance("Box 1"))
box2_label.bind("<Button-1>", lambda e: create_instance("Box 2"))

# Make the viewport draggable
def start_viewport_drag(event):
    viewport.scan_mark(event.x, event.y)

def on_viewport_drag(event):
    viewport.scan_dragto(event.x, event.y, gain=1)

viewport.bind("<ButtonPress-1>", start_viewport_drag)
viewport.bind("<B1-Motion>", on_viewport_drag)

# Start the GUI event loop
window.mainloop()
