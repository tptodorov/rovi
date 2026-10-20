"""Pure local input policy. Times are monotonic integer milliseconds."""
from .protocol import ARM, ClientError

ZERO = (0.0, 0.0, 0.0)
DIRECTIONS = {ord('w'): (0,1), ord('s'): (0,-1), ord('a'): (1,1),
              ord('d'): (1,-1), ord('q'): (2,1), ord('e'): (2,-1)}

class Policy:
    def __init__(self, caps):
        self.limits = (min(.1,caps.max_vx), min(.1,caps.max_vy), min(.3,caps.max_wz))
        self.armed = False
        self.intent = ZERO
        self.until = 0
        self.last_iteration = None

    def clear(self):
        self.armed = False
        self.intent = ZERO
        self.until = 0

    def input(self, keys, now):
        keys = keys.lower()
        if any(k in keys for k in (3,4,27)):
            self.clear()
            return 'quit'
        if b' ' in keys:
            self.clear()
            return 'stop'
        if b'r' in keys:
            self.clear()
            return 'arm'
        directions = [DIRECTIONS[k] for k in keys if k in DIRECTIONS]
        if self.armed and directions:
            axis, sign = directions[-1]
            vector = [0.0,0.0,0.0]
            vector[axis] = sign * self.limits[axis]
            self.intent = tuple(vector)
            self.until = now + 250
        return None

    def vector(self, now):
        if not self.armed or now >= self.until:
            self.intent = ZERO
        return self.intent

    def observe(self, status, control):
        if status.state != 'Armed' or status.reason or status.error:
            self.clear()
        elif control == ARM:
            self.clear()
            self.armed = True

    def iteration(self, now):
        if self.last_iteration is not None and (now < self.last_iteration or now-self.last_iteration >= 200):
            self.clear()
            raise ClientError('send loop stalled; start a new session')
        self.last_iteration = now
