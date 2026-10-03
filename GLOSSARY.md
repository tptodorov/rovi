# Rovi

Rovi is a modular robotics hardware and software platform. Its layers support different built devices and the applications that use them.

## Language

**Hardware platform**:
The physical chassis and electronics that provide the base for a built device.

**Device**:
A particular robot built on Rovi's hardware platform, with capabilities and supported commands defined for that build.

**Motor control**:
The capability to control a device's motors independently, without defining the higher-level movement behavior of the device.

**Device control API**:
The set of commands a device supports and the meaning of those commands. Each device defines its API from its own capabilities.

**Device command**:
An operation requested through a device's control API.

**Control interface**:
A device-facing boundary that turns received control input into commands supported by that device.

**Transport**:
A communication path that carries control input between a client and a device's control interface. The transport does not define the meaning of device commands.

**Application**:
Software behavior that uses a device's control API, such as teleoperation or autonomous driving.
