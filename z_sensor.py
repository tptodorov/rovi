import pickle
import zenoh
from getkey import getkey

from event import ControlEvent

key = "rovi/controller/event"
if __name__ == "__main__":
    session = zenoh.open()
    pub = session.declare_publisher(key)

    while True:  # Breaks when key is pressed
        key_byte = getkey()
        if key:
            key = key_byte.encode()
        else:
            continue
        event = None
        if key == b"q":
            break
        elif key in [b"\x1b[A", b"i"]:
            event = ControlEvent.UP
        elif key in [b"\x1b[B", b"k"]:
            event = ControlEvent.DOWN
        elif key in [b"\x1b[C", b"l"]:
            event = ControlEvent.RIGHT
        elif key in [b"\x1b[D", b"j"]:
            event = ControlEvent.LEFT
        elif key in [b"\x1b[D", b"u"]:
            event = ControlEvent.ROTATE_LEFT
        elif key in [b"\x1b[D", b"o"]:
            event = ControlEvent.ROTATE_RIGHT
        else:
            event = ControlEvent.STOP

        if event:
            print(event)
            pub.put(value=pickle.dumps(event))
        else:
            print("unknown", key)
