"""Loopback probes validate evidence and prove the recorder never claims/arms."""
import contextlib
import io
import json
from pathlib import Path
import socket
import subprocess
import tempfile
import threading
import unittest
from unittest.mock import ANY, Mock, call, patch

import wifi_soak as probe


class EchoPeer:
    def __init__(self, responses):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.bind(('127.0.0.1', 0)); self.socket.settimeout(.02)
        self.address = self.socket.getsockname()
        self.responses = responses
        self.received = []
        self.running = True
        self.thread = threading.Thread(target=self.serve)
        self.thread.start()

    def serve(self):
        while self.running:
            try:
                packet, peer = self.socket.recvfrom(1500)
            except socket.timeout:
                continue
            self.received.append(probe.parse(packet))
            for reply in self.responses(packet):
                self.socket.sendto(reply, peer)

    def close(self):
        self.running = False
        self.thread.join(timeout=1)
        self.socket.close()


class WifiSoak(unittest.TestCase):
    def request(self, responses):
        peer = EchoPeer(responses)
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_DGRAM) as sock:
                sock.connect(peer.address)
                result = probe.echo(sock, 42)
            self.assertTrue(peer.received)
            self.assertEqual(peer.received[0][:3], (probe.ECHO, 0, 42))
            return result
        finally:
            peer.close()

    def test_only_matching_echo_counts(self):
        def replies(packet):
            return [b'bad', probe.frame(probe.ECHO, 1, 42, packet[12:]),
                    probe.frame(probe.ECHO, 0, 41, packet[12:]),
                    probe.frame(probe.ECHO, 0, 42, b'old nonce'), packet]
        self.assertTrue(self.request(replies)['ok'])
        self.assertFalse(self.request(lambda p: [probe.frame(probe.ECHO, 0, 41, p[12:])])['ok'])

    def test_loss_then_recovery_is_retained_without_claim_or_rejoin(self):
        received = 0
        def responses(packet):
            nonlocal received
            received += 1
            return [] if received == 2 else [packet]
        peer = EchoPeer(responses)
        snapshots = []
        try:
            with tempfile.TemporaryDirectory() as d, contextlib.redirect_stdout(io.StringIO()), \
                 patch.object(probe, 'command', return_value={'exit': 1, 'stderr': 'journal unavailable'}):
                out = Path(d)/'evidence'
                rc = probe.observe(peer.address, .7, out, 'test-interface',
                                   collect=lambda _,label,*args: snapshots.append(label))
                rows = [json.loads(s) for s in (out/'echo.jsonl').read_text().splitlines()]
                summary = json.loads((out/'summary.json').read_text())
                self.assertEqual(rc, 1)
                self.assertTrue(rows[0]['ok']); self.assertFalse(rows[1]['ok']); self.assertTrue(rows[-1]['ok'])
                self.assertEqual(summary['lost'], 1)
                self.assertTrue(summary['complete']); self.assertFalse(summary['reassociation_requested'])
                self.assertIn('journal unavailable', (out/'journal.json').read_text())
                self.assertEqual(snapshots, ['start', 'transition-001', 'transition-002', 'transition-003', 'end'])
                self.assertTrue(all(p[:2] == (probe.ECHO, 0) for p in peer.received))
                with self.assertRaises(FileExistsError):
                    probe.observe(peer.address, .1, out, 'test-interface')
        finally:
            peer.close()

    def test_missing_tools_and_timeouts_remain_visible(self):
        for error in (FileNotFoundError(), subprocess.TimeoutExpired(['iw'], 5)):
            with patch.object(probe.subprocess, 'run', side_effect=error):
                self.assertIn('error', probe.command(['iw']))

    def test_echo_send_also_has_a_timeout(self):
        sock = Mock()
        sock.send.side_effect = socket.timeout
        self.assertFalse(probe.echo(sock, 1)['ok'])
        self.assertEqual(sock.mock_calls[:2], [call.settimeout(.15), call.send(ANY)])

    def test_no_loss_and_interruption_have_distinct_evidence(self):
        peer = EchoPeer(lambda packet: [packet])
        try:
            with tempfile.TemporaryDirectory() as d, contextlib.redirect_stdout(io.StringIO()), \
                 patch.object(probe, 'command', return_value={'exit': 0}):
                out = Path(d)/'complete'
                self.assertEqual(probe.observe(peer.address, .25, out, 'lo', collect=lambda *a: None), 0)
                self.assertEqual(json.loads((out/'summary.json').read_text())['verdict'], 'no loss observed')
                out = Path(d)/'interrupted'
                with patch.object(probe, 'echo', side_effect=KeyboardInterrupt), self.assertRaises(KeyboardInterrupt):
                    probe.observe(peer.address, .25, out, 'lo', collect=lambda *a: None)
                self.assertEqual(json.loads((out/'summary.json').read_text())['verdict'], 'incomplete')
        finally:
            peer.close()

    def test_snapshots_only_read_the_connection(self):
        with tempfile.TemporaryDirectory() as d, patch.object(probe, 'command', return_value={'exit': 0}) as run:
            probe.snapshot(Path(d), 'start', 'wifi-test', '192.0.2.1')
            self.assertEqual([c.args[0] for c in run.call_args_list], [
                ['nmcli', '-g', 'GENERAL.STATE,GENERAL.CONNECTION,IP4.ADDRESS,IP4.GATEWAY', 'device', 'show', 'wifi-test'],
                ['iw', 'dev', 'wifi-test', 'link'], ['iw', 'dev', 'wifi-test', 'get', 'power_save'],
                ['ip', '-j', 'route', 'get', '192.0.2.1'], ['ping', '-n', '-c', '3', '-W', '1', '192.0.2.1'],
            ])

    def test_duration_rejects_invalid_bounds(self):
        for value in ('nan', 'inf', '0', '-1', '3601'):
            with self.assertRaises(probe.argparse.ArgumentTypeError): probe.duration(value)


if __name__ == '__main__':
    unittest.main()
