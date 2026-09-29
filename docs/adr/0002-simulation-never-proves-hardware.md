# Simulation never counts as hardware-milestone proof

M1's Wokwi CI run (GPIO/PWM timing) was initially treated as if it validated the TB6612FNG motor-driver milestone, but Wokwi doesn't model the IC electrically, and can't simulate BLE radio at all (open upstream: [wokwi-features#225](https://github.com/wokwi/wokwi-features/issues/225)). Decided: a hardware milestone is never accepted from simulation results alone — physical bench verification is required, for every hardware component (drivers, sensors, radios), not just this one.
