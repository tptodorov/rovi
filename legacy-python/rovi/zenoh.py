import pickle
import time
import sys

import zenoh
from rovi.rovi import Car

from event import ControlEvent


class MyController:
    def __init__(self):
        Car.setup()
        self.car = Car()

    def listener(self, sample):
        event = pickle.loads(sample.payload)
        print(f"Received {sample.kind} ('{sample.key_expr}': '{type(event)}')")
        match event:
            case ControlEvent.UP:
                self.car.forward()
            case ControlEvent.DOWN:
                self.car.backward()
            case ControlEvent.LEFT:
                self.car.left()
            case ControlEvent.RIGHT:
                self.car.right()
            case ControlEvent.ROTATE_RIGHT:
                self.car.rotate_right()
            case ControlEvent.ROTATE_LEFT:
                self.car.rotate_left()
            case _:
                self.car.stop()


key = "rovi/controller/event"


if __name__ == "__main__":
    zenoh.init_logger()

    session = zenoh.open()
    car = MyController()
    sub = session.declare_subscriber(key, lambda s: car.listener(s))

    print("Enter 'q' to quit...")
    c = "\0"
    while c != "q":
        c = sys.stdin.read(1)
        if c == "":
            time.sleep(1)
    sub.undeclare()
    session.close()
    time.sleep(0.1)
    print("destroying")
    Car.destroy()
