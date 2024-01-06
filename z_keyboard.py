import pygame

from event import ControlEvent
import pickle
import zenoh

pygame.init()
window = pygame.display.set_mode((300, 300))
clock = pygame.time.Clock()

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

while run:
    clock.tick(60)
    for event in pygame.event.get():
        cevent = None
        if event.type == pygame.QUIT:
            run = False
        if event.type == pygame.KEYDOWN:
            key_name = pygame.key.name(event.key)
            print("press", key_name)
            cevent = key_to_event.get(key_name, ControlEvent.STOP)

        if event.type == pygame.KEYUP:
            print("stop")
            cevent = ControlEvent.STOP
        if cevent:
            print(cevent)
            pub.put(value=pickle.dumps(cevent))

    keys = pygame.key.get_pressed()

    rect.x += (keys[pygame.K_RIGHT] - keys[pygame.K_LEFT]) * vel
    rect.y += (keys[pygame.K_DOWN] - keys[pygame.K_UP]) * vel

    rect.centerx = rect.centerx % window.get_width()
    rect.centery = rect.centery % window.get_height()

    window.fill(0)
    pygame.draw.rect(window, (255, 0, 0), rect)
    pygame.display.flip()

pygame.quit()
exit()
