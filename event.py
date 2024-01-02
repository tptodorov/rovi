from enum import StrEnum, auto


class ControlEvent(StrEnum):
    UP = auto()
    DOWN = auto()
    LEFT = auto()
    RIGHT = auto()
    ROTATE_LEFT = auto()
    ROTATE_RIGHT = auto()
