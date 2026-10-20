"""Compatibility import for the shared client codecs; keep old harness imports working."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[3]))
from apps.client.wire import STATES, REASONS, ERRORS, Status, Capabilities, cdr, ble_setpoint
