import subprocess
import unittest
from unittest.mock import patch
from apps.teleop.audio import beep


class AudioTests(unittest.TestCase):
    @patch('apps.teleop.audio.subprocess.run')
    def test_loss_tone_has_bounded_playback_without_shell(self, run):
        self.assertTrue(beep())
        args,kwargs=run.call_args
        self.assertEqual(args[0][0],'pw-play')
        self.assertGreater(len(kwargs['input']),0)
        self.assertLessEqual(kwargs['timeout'],.5)
        self.assertNotIn('shell',kwargs)

    @patch('apps.teleop.audio.subprocess.run')
    def test_missing_failed_or_stalled_audio_returns_fallback(self, run):
        for error in (FileNotFoundError(),subprocess.CalledProcessError(1,'pw-play'),
                      subprocess.TimeoutExpired('pw-play',.5)):
            run.side_effect=error
            self.assertFalse(beep())
