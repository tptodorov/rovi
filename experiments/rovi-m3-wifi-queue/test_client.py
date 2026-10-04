"""Host-only client framing/stream checks; not radio acceptance."""
import contextlib
import io
import socket
import struct
import threading
import unittest
from unittest.mock import patch

from client import Client, main


class ClientTest(unittest.TestCase):
    def test_discovery_fragmented_ack_and_wire_layout(self):
        client_end, server_end = socket.socketpair()
        received = []

        def server():
            with server_end:
                server_end.sendall(b"ROVI version=1 session=42 now_ms=100 max_age_ms=250 watchdog_ms=1000 commands=0=stop,1=forward,2=reverse,3=ping\n")
                reader = server_end.makefile("rb")
                received.append(reader.read(22))
                server_end.sendall(b"ACK seq=1 status=Ping order=1 rx_ms=100 depth=0\n")
                received.append(reader.read(22))
                server_end.sendall(b"ACK seq=2 sta")
                server_end.sendall(b"tus=Queued order=2 rx_ms=100 depth=1\n")
                reader.close()

        thread = threading.Thread(target=server)
        thread.start()
        with patch("client.socket.create_connection", return_value=client_end), \
             patch("client.time.monotonic_ns", return_value=1_000_000), \
             contextlib.redirect_stdout(io.StringIO()):
            client = Client("unused")
            try:
                frame = client.frame("reverse")
                self.assertEqual(frame, struct.pack("<BBQIQ", 1, 2, 42, 2, 225))
                self.assertIn("status=Queued", client.send(frame))
                with self.assertRaises(ConnectionError):
                    client.receive(1_000_000, frame)
            finally:
                client.close()
        thread.join(timeout=2)
        self.assertFalse(thread.is_alive())
        self.assertEqual(received, [struct.pack("<BBQIQ", 1, 3, 42, 1, 225), frame])

    def test_late_discovery_does_not_make_commands_stale(self):
        # Client power save can deliver the banner late; the calibration ping's
        # board receive time, not the banner, anchors command deadlines.
        client_end, server_end = socket.socketpair()

        def server():
            with server_end:
                server_end.sendall(b"ROVI version=1 session=42 now_ms=100 max_age_ms=250 watchdog_ms=1000 commands=0=stop,1=forward,2=reverse,3=ping\n")
                reader = server_end.makefile("rb")
                reader.read(22)
                server_end.sendall(b"ACK seq=1 status=Stale order=0 rx_ms=400 depth=0\n")
                reader.close()

        thread = threading.Thread(target=server)
        thread.start()
        with patch("client.socket.create_connection", return_value=client_end), \
             patch("client.time.monotonic_ns", return_value=1_000_000), \
             contextlib.redirect_stdout(io.StringIO()):
            client = Client("unused")
            try:
                self.assertEqual(client.frame("forward"), struct.pack("<BBQIQ", 1, 1, 42, 2, 525))
            finally:
                client.close()
        thread.join(timeout=2)

    def test_refused_reconnect_is_retried(self):
        # The board refuses connections while it tears down the previous session.
        client_end, server_end = socket.socketpair()
        with server_end:
            server_end.sendall(b"ROVI version=2\n")
            with patch("client.socket.create_connection",
                       side_effect=[ConnectionRefusedError(), client_end]) as connect, \
                 patch("client.time.sleep"):
                with self.assertRaises(ValueError):
                    Client("unused")
            self.assertEqual(connect.call_count, 2)

    def test_hold_reports_reset_socket_as_closed(self):
        # The board aborts (RST) a session whose lease ended.
        listener = socket.create_server(("127.0.0.1", 0))
        port = listener.getsockname()[1]

        def server():
            with listener, listener.accept()[0] as conn:
                conn.sendall(b"ROVI version=1 session=42 now_ms=100 max_age_ms=250 watchdog_ms=1000 commands=0=stop,1=forward,2=reverse,3=ping\n")
                for seq in (1, 2):
                    conn.recv(22, socket.MSG_WAITALL)
                    conn.sendall(b"ACK seq=%d status=Queued order=%d rx_ms=100 depth=0\n" % (seq, seq))
                conn.setsockopt(socket.SOL_SOCKET, socket.SO_LINGER, struct.pack("ii", 1, 0))

        thread = threading.Thread(target=server)
        thread.start()
        connect = socket.create_connection
        out = io.StringIO()
        with patch("client.socket.create_connection", lambda _, timeout: connect(("127.0.0.1", port), timeout)), \
             patch("sys.argv", ["client.py", "forward", "--hold", "0.2"]), \
             contextlib.redirect_stdout(out):
            main()
        thread.join(timeout=2)
        self.assertIn('"socket_closed": true', out.getvalue())

    def test_unknown_protocol_fails_closed(self):
        client_end, server_end = socket.socketpair()
        with server_end:
            server_end.sendall(b"ROVI version=2\n")
            with patch("client.socket.create_connection", return_value=client_end):
                with self.assertRaises(ValueError):
                    Client("unused")


if __name__ == "__main__":
    unittest.main()
