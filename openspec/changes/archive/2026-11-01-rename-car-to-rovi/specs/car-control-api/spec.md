## ADDED Requirements

### Requirement: Advertised car name

The car SHALL advertise exactly `Rovi` as its BLE device name and in its BLE scan response, and SHALL use exactly `Rovi` as its car AP SSID.

#### Scenario: A client finds the car by name

- **WHEN** the car is free and running its car network
- **THEN** a BLE scan shows a device named `Rovi` and a Wi-Fi scan shows the network `Rovi`
