"""ROS serial evidence safeguards; the complete probe also runs on loopback in CI."""
import unittest

from ros_lease_timing import checked_stop


class LeaseEvidence(unittest.TestCase):
    def events(self, late=0):
        return [
            dict(ev="state", state="Armed", wheels_pm=[0, 0, 0, 0]),
            dict(ev="stop", reason="LeaseExpired", lease_late_ms=late),
        ]

    def test_waits_for_a_stop(self):
        self.assertIsNone(checked_stop([]))
        self.assertIsNone(checked_stop(self.events()[:1]))

    def test_zero_and_50_ms_boundaries_pass(self):
        for late in (0, 50):
            self.assertEqual(checked_stop(self.events(late)), late)

    def test_51_ms_fails(self):
        with self.assertRaisesRegex(AssertionError, "exceeds 50"):
            checked_stop(self.events(51))

    def test_missing_or_invalid_timing_fails(self):
        for late in (None, -1, "invalid"):
            with self.subTest(late=late), self.assertRaisesRegex(AssertionError, "lease_late_ms"):
                checked_stop(self.events(late))

    def test_nonzero_or_missing_wheel_commands_fail(self):
        for wheels in ([1, 0, 0, 0], None):
            events = self.events()
            events[0]["wheels_pm"] = wheels
            with self.assertRaisesRegex(AssertionError, "wheel command"):
                checked_stop(events)

    def test_missing_or_later_arm_cannot_validate_a_stop(self):
        events = self.events()
        for reordered in (events[1:], events[::-1]):
            with self.assertRaisesRegex(AssertionError, "Armed"):
                checked_stop(reordered)

    def test_wrong_reason_or_duplicate_stop_fails(self):
        events = self.events()
        events[1]["reason"] = "OwnerLost"
        with self.assertRaises(AssertionError):
            checked_stop(events)
        events = self.events()
        with self.assertRaisesRegex(AssertionError, "multiple stops"):
            checked_stop(events + events[1:])

    def test_reset_fault_drop_or_release_fails(self):
        for name in ("boot", "error", "dropped", "released"):
            with self.subTest(name=name), self.assertRaises(AssertionError):
                checked_stop([dict(ev=name)] + self.events())


if __name__ == "__main__":
    unittest.main()
