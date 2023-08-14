from PyQt6.QtWidgets import QWidget, QGraphicsView, QGraphicsScene, QApplication, QVBoxLayout
from PyQt6.QtCore import Qt, QPointF, QLineF
from PyQt6.QtGui import QPen, QColor

class FocusTreeTool(QWidget):
    def __init__(self):
        super().__init__()

        self.view = QGraphicsView(self)
        self.view.setScene(QGraphicsScene())
        self.view.setViewportUpdateMode(QGraphicsView.ViewportUpdateMode.MinimalViewportUpdate)
        self.view.setSceneRect(-500, -500, 1000, 1000)

        # Disable the scrollbars and set the background color
        self.view.setHorizontalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)
        self.view.setVerticalScrollBarPolicy(Qt.ScrollBarPolicy.ScrollBarAlwaysOff)

        # Store the current grid origin
        self.grid_origin = QPointF(0, 0)

        # Add a test item to the scene
        self.addTestItem()

        # Variables to handle drag functionality
        self.last_mouse_pos = None
        self.view.setMouseTracking(True)

        # Store the zoom factor
        self.zoom_factor = 1.0

        layout = QVBoxLayout()
        layout.addWidget(self.view)
        self.setLayout(layout)

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

    def addTestItem(self):
        item = self.view.scene().addRect(-100, -100, 200, 200, QPen(Qt.GlobalColor.green), Qt.GlobalColor.green)
        item.setPos(QPointF(0, 0))

    def wheelEvent(self, event):
        # Enable zooming using the mouse wheel
        zoom_in = event.angleDelta().y() > 0
        zoom_factor = 1.25 if zoom_in else 0.8
        self.zoom_factor *= zoom_factor
        self.view.scale(zoom_factor, zoom_factor)

    def mousePressEvent(self, event):
        self.last_mouse_pos = event.pos()

    def mouseMoveEvent(self, event):
        if self.last_mouse_pos is not None:
            dx = (event.pos().x() - self.last_mouse_pos.x()) / self.zoom_factor
            dy = (event.pos().y() - self.last_mouse_pos.y()) / self.zoom_factor
            self.translate(dx, dy)
            self.last_mouse_pos = event.pos()

    def mouseReleaseEvent(self, event):
        self.last_mouse_pos = None

    def translate(self, dx, dy):
        # Helper method to perform translation while keeping the scene centered
        self.grid_origin -= QPointF(dx, dy)
        self.view.setSceneRect(self.grid_origin.x() - 500, self.grid_origin.y() - 500, 1000, 1000)
        self.view.centerOn(self.grid_origin)
