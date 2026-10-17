# M5 F6: first floor diagonal — 2026-10-17

Measured-geometry firmware: radius 0.030 m, centre projection sum 0.1275 m; speed normalization remains a placeholder. Setup requested: car centred at HOME, FRONT unchanged, motor bank awake.

BLE command: `drive --vx 0.05 --vy 0.05 --seconds 0.5`, after a 5 s lead-in. [Client output](diagonal-left-005-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error.

Todor measured **4.5 cm forward and 4.5 cm left**. From these displacements, the path angle is approximately **45° left of forward** and total displacement approximately **6.4 cm**. Heading change and physical stopping were not separately reported; a 45° displacement does not establish an unchanged chassis heading.

One diagonal result only. Repeatability, forward-right motion, bank charge, surface and measurement uncertainty remain uncharacterized. The command values are not calibrated m/s.
