#! /usr/bin/python

import os.path
import tornado.httpserver
import tornado.websocket
import tornado.ioloop
import tornado.web

from event import ControlEvent

from rovi.rovi import Car

# Tornado Folder Paths
settings = dict(
    template_path=os.path.join(os.path.dirname(__file__), "templates"),
    static_path=os.path.join(os.path.dirname(__file__), "static"),
)

# Tonado server port
PORT = 8000

msg_to_event = {
    "STOP": ControlEvent.STOP,
    "LEFT": ControlEvent.LEFT,
    "RIGHT": ControlEvent.RIGHT,
    "UP": ControlEvent.UP,
    "DOWN": ControlEvent.DOWN,
    "ROTATE_LEFT": ControlEvent.ROTATE_LEFT,
    "ROTATE_RIGHT": ControlEvent.ROTATE_RIGHT,
}


class MainHandler(tornado.web.RequestHandler):
    def get(self):
        print("[HTTP](MainHandler) User Connected.")
        self.render("index.html")


class WSHandler(tornado.websocket.WebSocketHandler):
    def open(self):
        print("[WS] Connection was opened.")
        Car.setup()
        self.car = Car()

    def on_message(self, message):
        print("[WS] Incoming message:", message)
        event = msg_to_event.get(str(message), ControlEvent.STOP)
        print("[WS] Incoming event:", event)
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

    def on_close(self):
        print("[WS] Connection was closed.")
        Car.destroy()


application = tornado.web.Application(
    [
        (r"/", MainHandler),
        (r"/ws", WSHandler),
    ],
    **settings,
)


if __name__ == "__main__":
    http_server = tornado.httpserver.HTTPServer(application)
    http_server.listen(PORT)
    main_loop = tornado.ioloop.IOLoop.instance()

    print("Tornado Server started")
    main_loop.start()
