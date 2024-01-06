import pickle
import time

import zenoh

from event import ControlEvent


def listener(sample):
    event = pickle.loads(sample.payload)
    print(f"Received {sample.kind} ('{sample.key_expr}': '{type(event)}')")
    match event:
        case ControlEvent.UP:
            print("car up")
        case ControlEvent.DOWN:
            print("car down")
        case ControlEvent.LEFT:
            print("car left")
        case ControlEvent.RIGHT:
            print("car right")
        case ControlEvent.ROTATE_RIGHT:
            print("rotate right")
        case ControlEvent.ROTATE_LEFT:
            print("rotate left")
        case _:
            raise NotImplementedError


key = "rovi/controller/event"

if __name__ == "__main__":
    session = zenoh.open()
    sub = session.declare_subscriber(key, listener)
    time.sleep(60)
