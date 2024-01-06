import pickle

import pygame
import zenoh
from event import ControlEvent

pygame.init()
clock = pygame.time.Clock()

window = pygame.display.set_mode((300, 300))
rect = pygame.Rect(0, 0, 20, 20)
rect.center = window.get_rect().center
vel = 5

run = True

key_to_event = {
    "u": ControlEvent.ROTATE_LEFT,
    "o": ControlEvent.ROTATE_RIGHT,
    "i": ControlEvent.UP,
    "k": ControlEvent.DOWN,
    "j": ControlEvent.LEFT,
    "l": ControlEvent.RIGHT,
}

key = "rovi/controller/event"
session = zenoh.open()
pub = session.declare_publisher(key)

print("controls are defined as", key_to_event)

while run:
    clock.tick(60)
    for event in pygame.event.get():
        cevent = None
        if event.type == pygame.QUIT:
            run = False
        if event.type == pygame.KEYDOWN:
            key_name = pygame.key.name(event.key)
            cevent = key_to_event.get(key_name, ControlEvent.STOP)
        if event.type == pygame.KEYUP:
            cevent = ControlEvent.STOP
        if cevent:
            print(cevent)
            pub.put(value=pickle.dumps(cevent))

    keys = pygame.key.get_pressed()
    pygame.display.flip()

pygame.quit()
exit()
