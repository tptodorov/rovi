## Remote Controlled Car with 4 mecanum-wheels

### Car

* 4x mecanum wheels
* 1x car frame
* 4x electric engines
* 2x 2-motor drivers
* 1-2 rechargable batteries with USB ports
* raspberry pi zero2 w

### Control Software 

The following package is ment to run on Raspberry PI with Python 3.11

The 4 motors are controlled via WPM pins connected to the motor drivers. 
For each motor, one pin is used to control the direction and another pin to control the speed.

Implemented Control methods
* over bluethooth PS4-controller directly connected to the car
* network via websockets from any internet host using web page
* network via zenoh from the local network using a keyboard
* 

  


