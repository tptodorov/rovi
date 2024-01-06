import pickle
import time
import sys

import zenoh
from rovi import *

from event import ControlEvent

class MyController():
    def __init__(self, **kwargs):
       Car.setup()
       self.car= Car()

    def listener(self,sample):
        event = pickle.loads(sample.payload)
        print(f"Received {sample.kind} ('{sample.key_expr}': '{type(event)}')")
        match event:
            case ControlEvent.UP:
                print("car up")
                self.car.forward()
            case ControlEvent.DOWN:
                self.car.backward()
                print("car down")
            case ControlEvent.LEFT:
                print("car left")
                self.car.left()
            case ControlEvent.RIGHT:
                print("car right")
                self.car.right()
            case ControlEvent.ROTATE_RIGHT:
                print("rotate right")
                self.car.rotate_right()
            case ControlEvent.ROTATE_LEFT:
                print("rotate left")
                self.car.rotate_left()
            case _:
                self.car.stop()


key = "rovi/controller/event"

if __name__ == "__main__":
    zenoh.init_logger()

    session = zenoh.open()
    car = MyController()
    sub = session.declare_subscriber(key,lambda s: car.listener(s))

    print("Enter 'q' to quit...")
    c = '\0'
    while c != 'q':
        c = sys.stdin.read(1)
        if c == '':
            time.sleep(1)
    sub.undeclare()
    session.close()
    time.sleep(0.1)
    print("destroying")
    Car.destroy()
