"""Short loss tone through the existing PipeWire player; no device effects."""
import math
import struct
import subprocess


def beep():
    # Playback only runs after motion is cleared; cap a missing/hung audio service.
    tone = b''.join(struct.pack('<h',int(8000*math.sin(2*math.pi*880*i/16000)))
                    for i in range(1920))
    try:
        subprocess.run(['pw-play','--raw','--rate','16000','--channels','1',
                        '--format','s16','-'],input=tone,stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL,timeout=.5,check=True)
        return True
    except (OSError,subprocess.SubprocessError):
        return False
