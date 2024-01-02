import pickle
import zenoh, random, time
from getkey import getkey

from event import ControlEvent

key = "rovi/controller/event"
if __name__ == "__main__":
    session = zenoh.open()
    pub = session.declare_publisher(key)

    while True:  # Breaks when key is pressed
        key = getkey().encode()
        print("key", key)  # Optionally prints out the key.
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
        if event:
            pub.put(pickle.dumps(event))

    # while True:
    #     t = read_temp()
    #     buf = f"{t}"
    #     print(f"Putting Data ('{key}': '{buf}')...")
    #     pub.put(buf)
    #     time.sleep(1)
