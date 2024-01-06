import RPi.GPIO as GPIO
import time

# left
in1 = 27
in2 = 22
# right
in3 = 23
in4 = 24

# back
# left
bin1 = 26
bin2 = 13
# right
bin3 = 5
bin4 = 6

chan_list = [in1, in2, in3, in4, bin1, bin2, bin3, bin4]


class Wheel:
    speed = 50

    def __init__(self, forwardpin, backwardpin):
        self.forwardpin = forwardpin
        self.backwardpin = backwardpin
        self.fpwm = GPIO.PWM(forwardpin, 2000)
        self.bpwm = GPIO.PWM(backwardpin, 2000)

    def drive(self, forward):
        if forward:
            self.fpwm.start(self.speed)
            self.bpwm.stop()
        else:
            self.bpwm.start(self.speed)
            self.fpwm.stop()

    def stop(self):
        self.fpwm.stop()
        self.bpwm.stop()


class Car:
    def __init__(self):
        self.fl = Wheel(in1, in2)
        self.fr = Wheel(in4, in3)
        self.bl = Wheel(bin1, bin2)
        self.br = Wheel(bin3, bin4)

    def forward(self):
        self.fl.drive(True)
        self.fr.drive(True)
        self.bl.drive(True)
        self.br.drive(True)

    def backward(self):
        self.fl.drive(False)
        self.fr.drive(False)
        self.bl.drive(False)
        self.br.drive(False)

    def right(self):
        self.fl.drive(True)
        self.fr.drive(False)
        self.bl.drive(False)
        self.br.drive(True)

    def left(self):
        self.fl.drive(False)
        self.fr.drive(True)
        self.bl.drive(True)
        self.br.drive(False)

    def rotate_right(self):
        self.fl.drive(True)
        self.fr.drive(False)
        self.bl.drive(True)
        self.br.drive(False)

    def rotate_left(self):
        self.fl.drive(False)
        self.fr.drive(True)
        self.bl.drive(False)
        self.br.drive(True)

    def stop(self):
        self.fl.stop()
        self.fr.stop()
        self.bl.stop()
        self.br.stop()

    @staticmethod
    def setup():
        # setting up
        GPIO.setmode(GPIO.BCM)
        GPIO.setup(chan_list, GPIO.OUT)
        GPIO.output(chan_list, GPIO.LOW)  # sets all to GPIO.LOW

    @staticmethod
    def destroy():
        GPIO.cleanup()
