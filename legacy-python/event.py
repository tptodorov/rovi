from enum import Enum, auto


class ControlEvent(Enum):
    STOP = auto()
    UP = auto()
    DOWN = auto()
    LEFT = auto()
    RIGHT = auto()
    ROTATE_LEFT = auto()
    ROTATE_RIGHT = auto()
