"""Host-only client framing/stream checks; not radio acceptance."""
import contextlib
import io
import socket
import struct
import threading
import unittest
from unittest.mock import patch

from client import Client


class ClientTest(unittest.TestCase):
    def test_discovery_fragmented_ack_and_wire_layout(self):
        client_end, server_end = socket.socketpair()
        received = []

        def server():
            with server_end:
                server_end.sendall(b"ROVI version=1 session=42 now_ms=100 max_age_ms=250 watchdog_ms=1000 commands=0=stop,1=forward,2=reverse,3=ping\n")
                reader = server_end.makefile("rb")
                received.append(reader.read(22))
                server_end.sendall(b"ACK seq=1 sta")
                server_end.sendall(b"tus=Queued order=1 rx_ms=100 depth=1\n")
                reader.close()

        thread = threading.Thread(target=server)
        thread.start()
        with patch("client.socket.create_connection", return_value=client_end), \
             patch("client.time.monotonic_ns", return_value=1_000_000), \
             contextlib.redirect_stdout(io.StringIO()):
            client = Client("unused")
            try:
                frame = client.frame("reverse")
                self.assertEqual(frame, struct.pack("<BBQIQ", 1, 2, 42, 1, 225))
                self.assertIn("status=Queued", client.send(frame))
                with self.assertRaises(ConnectionError):
                    client.receive(1_000_000, frame)
            finally:
                client.close()
        thread.join(timeout=2)
        self.assertFalse(thread.is_alive())
        self.assertEqual(received, [frame])

    def test_unknown_protocol_fails_closed(self):
        client_end, server_end = socket.socketpair()
        with server_end:
            server_end.sendall(b"ROVI version=2\n")
            with patch("client.socket.create_connection", return_value=client_end):
                with self.assertRaises(ValueError):
                    Client("unused")


if __name__ == "__main__":
    unittest.main()
