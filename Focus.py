import tkinter as tk
import csv


class DraggableRectangle:
    def __init__(self, canvas, x, y, color, width, height):
        self.canvas = canvas
        self.x = x
        self.y = y
        self.width = width
        self.height = height
        self.color = color
        self.rect = None
        self.text = None  # Added for displaying data points
        self.prev_x = None
        self.prev_y = None
        self.data_points = {}
        self.canvas.bind("<B1-Motion>", self.drag)
        self.canvas.bind("<ButtonRelease-1>", self.drop)

    def draw(self):
        self.rect = self.canvas.create_rectangle(
            self.x, self.y, self.x + self.width, self.y + self.height, fill=self.color
        )

        # Create a text element to display data points
        self.text = self.canvas.create_text(
            self.x + self.width / 2,
            self.y + self.height / 2,
            text=self.get_data_points_string(),
            fill="white"
        )

    def update_text(self):
        # Update the text element with the latest data points
        self.canvas.itemconfig(self.text, text=self.get_data_points_string())

    def drag(self, event):
        if self.prev_x is not None and self.prev_y is not None:
            dx = event.x - self.prev_x
            dy = event.y - self.prev_y

            new_x = self.x + dx
            new_y = self.y + dy

            if self.is_inside_shape(new_x, new_y):
                self.canvas.move(self.rect, dx, dy)
                self.canvas.move(self.text, dx, dy)  # Move the text element
                self.x = new_x
                self.y = new_y

        self.prev_x = event.x
        self.prev_y = event.y

    def is_inside_shape(self, x, y):
        x1, y1, x2, y2 = self.canvas.coords(self.rect)
        return x1 <= x <= x2 and y1 <= y <= y2

    def drop(self, event):
        self.prev_x = None
        self.prev_y = None

    def add_data_point(self, label, value):
        self.data_points[label] = value
        self.update_text()  # Update the displayed text when adding a data point

    def get_data_points_string(self):
        # Convert the data points dictionary to a string
        return "\n".join([f"{label}: {value}" for label, value in self.data_points.items()])

    def export_as_csv(self):
        csv_data = [["Label", "Value"]]
        for label, value in self.data_points.items():
            csv_data.append([label, value])

        file_name = f"box_data_{self.x}_{self.y}.csv"
        with open(file_name, "w", newline="") as file:
            writer = csv.writer(file)
            writer.writerows(csv_data)


def place_block(event):
    x = (event.x + canvas_x_offset) // block_size
    y = (event.y + canvas_y_offset) // block_size

    # Create a new DraggableRectangle
    draggable_rect = DraggableRectangle(
        overlay_canvas, x * block_size, y * block_size, "blue", block_size, block_size
    )
    draggable_rect.draw()
    draggable_rects.append(draggable_rect)


def start_drag(event):
    global prev_x, prev_y
    prev_x = event.x_root
    prev_y = event.y_root


def drag(event):
    global canvas_x_offset, canvas_y_offset, prev_x, prev_y

    dx = prev_x - event.x_root
    dy = prev_y - event.y_root

    canvas_x_offset += dx
    canvas_y_offset += dy

    canvas.move("all", dx, dy)

    prev_x = event.x_root
    prev_y = event.y_root


def on_canvas_configure(event):
    canvas.configure(scrollregion=canvas.bbox("all"))


def on_canvas_mousewheel(event):
    x = (event.x + canvas_x_offset) // block_size
    y = (event.y + canvas_y_offset) // block_size

    if event.delta > 0:
        zoom_in(x, y)
    else:
        zoom_out(x, y)


def zoom_in(x, y):
    global block_size, canvas_x_offset, canvas_y_offset

    if block_size < 200:
        scale_factor = 1.1
        block_size = int(block_size * scale_factor)
        canvas_x_offset = int((canvas_x_offset + x * block_size) * scale_factor - x * block_size)
        canvas_y_offset = int((canvas_y_offset + y * block_size) * scale_factor - y * block_size)

        canvas.scale("all", x * block_size, y * block_size, scale_factor, scale_factor)
        canvas.configure(scrollregion=canvas.bbox("all"))


def zoom_out(x, y):
    global block_size, canvas_x_offset, canvas_y_offset

    if block_size > 10:
        scale_factor = 0.9
        block_size = int(block_size * scale_factor)
        canvas_x_offset = int((canvas_x_offset + x * block_size) * scale_factor - x * block_size)
        canvas_y_offset = int((canvas_y_offset + y * block_size) * scale_factor - y * block_size)

        canvas.scale("all", x * block_size, y * block_size, scale_factor, scale_factor)
        canvas.configure(scrollregion=canvas.bbox("all"))


def cancel_selection():
    if selected_rect.get():
        selected_rect.get().prev_x = None
        selected_rect.get().prev_y = None
        selected_rect.set(None)


# Configuration
block_size = 50
grid_width = 200
grid_height = 200

# Create the main window
window = tk.Tk()
window.title("Infinite Grid")

# Create a frame with a scrollable canvas
frame = tk.Frame(window)
frame.pack(fill=tk.BOTH, expand=True)

ui_frame = tk.Frame(frame, width=200)
ui_frame.pack(side=tk.LEFT, fill=tk.Y)

canvas_frame = tk.Frame(frame)
canvas_frame.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)

canvas = tk.Canvas(canvas_frame, width=800, height=600)
canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)
canvas.bind("<Configure>", on_canvas_configure)
canvas.bind("<MouseWheel>", on_canvas_mousewheel)  # Scroll to zoom

scrollbar = tk.Scrollbar(canvas_frame, orient=tk.VERTICAL, command=canvas.yview)
scrollbar.pack(side=tk.RIGHT, fill=tk.Y)
canvas.configure(yscrollcommand=scrollbar.set)

# Create a canvas for the grid and UI elements
overlay_canvas = tk.Canvas(canvas, width=grid_width * block_size, height=grid_height * block_size)
overlay_canvas.pack()

background_canvas = tk.Canvas(canvas, width=grid_width * block_size, height=grid_height * block_size)
background_canvas.pack()
background_canvas.pack_propagate(0)  # Prevent automatic resizing
background_canvas.bind("<Button-1>", lambda event: "break")  # Disable selection of background grid

# Bind mouse events for panning
overlay_canvas.bind("<ButtonPress-1>", start_drag)
overlay_canvas.bind("<B1-Motion>", drag)
overlay_canvas.bind("<ButtonRelease-1>", lambda event: None)

# Variables for panning and selection
canvas_x_offset = 0
canvas_y_offset = 0
prev_x = 0
prev_y = 0
selected_rect = tk.StringVar()

# Draw the grid lines on the overlay canvas
for i in range(grid_width):
    overlay_canvas.create_line(i * block_size, 0, i * block_size, grid_height * block_size, fill="lightgray")
for j in range(grid_height):
    overlay_canvas.create_line(0, j * block_size, grid_width * block_size, j * block_size, fill="lightgray")

# Create rectangles in the UI interface
ui_canvas = tk.Canvas(ui_frame, width=150, height=600)
ui_canvas.pack(side=tk.LEFT, fill=tk.BOTH, expand=True)


def create_draggable_rectangle(color, width, height):
    x = width // 2
    y = height // 2
    draggable_rect = DraggableRectangle(overlay_canvas, x, y, color, width, height)
    draggable_rect.draw()
    draggable_rects.append(draggable_rect)


def handle_ui_rectangle_click(event):
    cancel_selection()
    item = ui_canvas.find_closest(event.x, event.y)
    if item:
        bbox = ui_canvas.bbox(item)
        color = ui_canvas.itemcget(item, "fill")
        width = bbox[2] - bbox[0]
        height = bbox[3] - bbox[1]
        create_draggable_rectangle(color, width, height)


ui_canvas.create_rectangle(20, 20, 80, 80, fill="red")
ui_canvas.create_rectangle(20, 100, 80, 160, fill="green")
ui_canvas.create_rectangle(20, 180, 80, 240, fill="blue")

ui_canvas.bind("<Button-1>", handle_ui_rectangle_click)


def add_data_point():
    if selected_rect.get():
        label = data_point_label_entry.get()
        value = data_point_value_entry.get()
        selected_rect.get().add_data_point(label, value)


def export_data_as_csv():
    for draggable_rect in draggable_rects:
        draggable_rect.export_as_csv()


# UI elements for adding data points and exporting data
data_point_label_entry = tk.Entry(ui_frame, width=20)
data_point_label_entry.pack()
data_point_value_entry = tk.Entry(ui_frame, width=20)
data_point_value_entry.pack()

add_data_point_button = tk.Button(ui_frame, text="Add Data Point", command=add_data_point)
add_data_point_button.pack()

export_csv_button = tk.Button(ui_frame, text="Export CSV", command=export_data_as_csv)
export_csv_button.pack()

# List to store all draggable rectangles
draggable_rects = []

# Start the GUI event loop
window.mainloop()
