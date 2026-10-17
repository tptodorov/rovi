# M5 F5: first floor rotation — 2026-10-17

Measured-geometry firmware: radius 0.030 m, centre projection sum 0.1275 m; speed normalization remains a placeholder. Setup requested: centred at HOME, FRONT aligned with a reference line, motor bank awake.

BLE command: `drive --vx 0 --wz 0.5 --seconds 0.5`, after a 5 s lead-in. [Client output](rotate-left-05-client.txt): 11 setpoints, Claimed → Armed → Stopped (`Stop`), no reported error.

Todor reported approximately **40° left rotation**. Centre displacement and physical stopping behavior were not separately reported. Angle measurement uncertainty, bank charge and floor surface were not recorded.

40° / 0.5 s corresponds to approximately **80°/s = 1.40 rad/s**, but acceleration and coast after STOP are not separated. This is not steady-state angular speed. Commanded `wz 0.5` is not calibrated to actual chassis turn rate. One leftward run only; repeatability and clockwise rotation remain untested.
