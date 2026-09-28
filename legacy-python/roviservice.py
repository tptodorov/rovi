from remote.websocket import application
import tornado.httpserver
import tornado.websocket
import tornado.ioloop
import tornado.web
from tornado_proxy_handlers.handlers import ProxyHandler

PORT=8000

if __name__ == "__main__":
    proxy_url="http://localhost:7000"
    handlers = [
        (r"/webcam/state", ProxyHandler, {"proxy_url": proxy_url + "/state"}),
        (r"/webcam/stream", ProxyHandler, {"proxy_url": proxy_url + "/stream"}),
    ]
    application.add_handlers(".*$", handlers)
    http_server = tornado.httpserver.HTTPServer(application)
    http_server.listen(PORT)
    main_loop = tornado.ioloop.IOLoop.instance()

    print("Tornado Server started")
    main_loop.start()
