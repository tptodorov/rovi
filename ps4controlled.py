from pyPS4Controller.controller import Controller
from rovi import *

def _normalize(v):
    return abs(round(v/32767 * 100))

def connect():
    print("controller connected")

def disconnect():
    print("controller disconnected")
    Car.destroy()
    
class MyController(Controller):

    def __init__(self, **kwargs):
        Controller.__init__(self, **kwargs)
        Car.setup()
        self.car= Car()

    def on_x_press(self):
       print("Hello world")

    def on_x_release(self):
        print("Goodbye world")

    def on_R3_up(self, value):
        value=_normalize(value)
        print("drive forward",value)
        self.car.backward()
    def on_R3_down(self, value):
        value=_normalize(value)
        print("drive backward",value)
        self.car.forward()
    def on_R3_left(self, value):
        value=_normalize(value)
        print("pressed left",value)
        self.car.right()
    def on_R3_right(self, value):
        value=_normalize(value)
        print("pressed right",value)
        self.car.left()
    def on_R3_y_at_rest(self):
        self.car.stop()
    def on_R3_x_at_rest(self):
        self.car.stop()
    def on_L3_x_at_rest(self):
        self.car.stop()
    def on_L3_y_at_rest(self):
        self.car.stop()
    def on_L3_right(self, value):
        value=_normalize(value)
        self.car.rotate_right()
    def on_L3_left(self, value):
        value=_normalize(value)
        self.car.rotate_left()

controller = MyController(interface="/dev/input/js0", connecting_using_ds4drv=False)
# you can start listening before controller is paired, as long as you pair it within the timeout window
controller.listen(timeout=60, on_connect=connect, on_disconnect=disconnect)