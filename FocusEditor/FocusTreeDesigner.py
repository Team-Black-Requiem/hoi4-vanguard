from PyQt6.QtWidgets import QWidget, QGraphicsView, QGraphicsScene, QVBoxLayout, QGraphicsItem
from PyQt6.QtCore import Qt, QPointF, QLineF, QRectF
from PyQt6.QtGui import QPen, QColor, QPainter, QBrush 
from PyQt6 import uic

class FocusTreeTool(QWidget):
    def __init__(self):
        super().__init__()
        
        layout = QVBoxLayout()
        self.viewport = Viewport()
        layout.addWidget(self.viewport)
        # Create and set up the QGraphicsView
        self.setLayout(layout)

class FocusEditorUI(QWidget):
    def __init__(self):
        super().__init__()
        uic.loadUi('FocusEditor/FocusEditor.ui', self)
        self.setFixedSize(600, 469)
       
       
    #### Begining of Focus Templates

class defaultFocusNode(QGraphicsItem):
    def __init__(self):
        super().__init__()
        self.rect = QRectF(-50, -25, 100, 50)  # Default size for the node
        self.setFlag(QGraphicsItem.GraphicsItemFlag.ItemIsMovable)  
        self.setFlag(QGraphicsItem.GraphicsItemFlag.ItemIsSelectable)
        self.setFlag(QGraphicsItem.GraphicsItemFlag.ItemSendsScenePositionChanges) 
        self.setZValue(1)  # Ensure nodes appear on top of the grid

    def boundingRect(self):
        return self.rect

    def paint(self, painter: QPainter, option, widget):
        color = QColor(255, 255, 255)  # Default node color (white)
        if self.isSelected():
            color = QColor(0, 0, 255)  # Selected node color (blue)

        pen = QPen(QColor(0, 0, 0))  # Node border color (black)
        painter.setPen(pen)
        painter.setBrush(QBrush(color))
        painter.drawRoundedRect(self.rect, 5, 5)

        ### End of Focus Templates

class Viewport(QGraphicsView):
    def __init__(self):
        super().__init__()

        self.setScene(QGraphicsScene())
        self.setViewportUpdateMode(QGraphicsView.ViewportUpdateMode.MinimalViewportUpdate)
        self.setSceneRect(-500, -500, 1000, 1000)

        self.min_zoom_factor = 0.4
        self.max_zoom_factor = 5.0

        # Disable the scrollbars and set the background color
        self.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)

        # Store the current grid origin
        self.grid_origin = QPointF(0, 0)

        # Variables to handle drag functionality
        self.last_mouse_pos = None
        self.setMouseTracking(True)
        self.dragged_item = None

        # Store the zoom factor
        self.zoom_factor = 1.0

        # Store created focus nodes
        self.nodes = []

    def drawBackground(self, painter, rect):
        # Override the drawBackground method to draw the grid lines dynamically
        grid_size = 50
        pen = QPen(QColor(128, 128, 128), 1)  # Gray pen for the grid lines

        left = int(rect.left()) - (int(rect.left()) % grid_size)
        top = int(rect.top()) - (int(rect.top()) % grid_size)

        lines = []
        for x in range(left, int(rect.right()), grid_size):
            lines.append(QLineF(x, rect.top(), x, rect.bottom()))
        for y in range(top, int(rect.bottom()), grid_size):
            lines.append(QLineF(rect.left(), y, rect.right(), y))

        painter.setPen(pen)
        painter.drawLines(lines)

    def wheelEvent(self, event):
        # Enable zooming using the mouse wheel
        zoom_in = event.angleDelta().y() > 0
        zoom_factor = 1.25 if zoom_in else 0.8
        new_zoom = self.zoom_factor * zoom_factor
        if self.min_zoom_factor <= new_zoom <= self.max_zoom_factor:
            self.zoom_factor = new_zoom
            self.scale(zoom_factor, zoom_factor)
            
    def mousePressEvent(self, event):
        if event.button() == Qt.MouseButton.LeftButton:
            items_at_pos = self.items(event.pos())
            if items_at_pos:
                # Check if a defaultFocusNode instance is found
                for item in items_at_pos:
                    if isinstance(item, defaultFocusNode):
                        self.dragged_item = item
                        self.dragged_item_offset = item.pos() - QPointF(event.pos())
                        return  # Exit early if a node is found
            else:
                # No node found, update last_mouse_pos for panning
                self.last_mouse_pos = event.pos()

        # Right-click behavior (creating a new node)
        if event.button() == Qt.MouseButton.RightButton:
            self.createFocusNode(event.pos())

    def mouseMoveEvent(self, event):
        if self.last_mouse_pos is not None:
            dx = event.pos().x() - self.last_mouse_pos.x()
            dy = event.pos().y() - self.last_mouse_pos.y()
            zoomed_dx = dx / self.zoom_factor
            zoomed_dy = dy / self.zoom_factor
            self.translate(zoomed_dx, zoomed_dy)
            self.last_mouse_pos = event.pos()

        if self.dragged_item is not None:
            new_pos = self.mapToScene(event.pos())
            self.dragged_item.setPos(new_pos)


    def mouseReleaseEvent(self, event):
        self.last_mouse_pos = None
        if self.dragged_item is not None:
            self.dragged_item = None
        else:
            # Check if a node was just clicked (not dragged)
            items_at_pos = self.items(event.pos())
            if items_at_pos:
                for item in items_at_pos:
                    if isinstance(item, defaultFocusNode):
                        # Select the clicked node
                        item.setSelected(True)
                        return


    def translate(self, dx, dy):
        # Helper method to perform translation while keeping the scene centered
        self.grid_origin -= QPointF(dx, dy)
        self.setSceneRect(self.grid_origin.x() - 500, self.grid_origin.y() - 500, 1000, 1000)
        self.centerOn(self.grid_origin)

    def createFocusNode(self, position):
        # Create a new focus node and add it to the scene
        focus_node = defaultFocusNode()
        focus_node.setPos(self.mapToScene(position))  # Convert viewport coordinates to scene coordinates
        self.scene().addItem(focus_node)
        self.nodes.append(focus_node)  # Store the created node for further processing