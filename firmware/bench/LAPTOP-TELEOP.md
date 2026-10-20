# Laptop teleoperation stand check — prepared, not run

Uses the implemented `python3 -m apps.teleop` client and installed normal ROS
firmware. No flash, router or firewall rule. Opening the serial capture resets
the board once. The operator must separately authorize this exact motor batch
with go; the laptop-only tests do not authorize it.

Rover on its stand, wheels clear, motor bank awake, USB connected. Save the
current laptop Wi-Fi connection, open a fresh serial capture, confirm a clean
boot, join Rovi's AP and start the client in a pseudo-terminal. The controlling
script/assistant supplies the same input bytes as an interactive terminal.

1. Startup idle for 5 s. Send W without R for 1 s: all wheels must stay still.
2. R, then 1 s at zero; require a matching Armed Status before motion.
3. Announce the first move with a 5 s lead. W, S, A, D, Q, E in order: repeat
   each key every 100 ms for **2 s**, then send no key for **1 s**. Translation
   requests are 0.1 m/s; rotation requests are 0.3 rad/s, clamped to capabilities.
   Verify all six commanded wheel-sign patterns and a fresh zero after every
   input expiry. Todor watches directions and stops; serial records commands.
4. Repeat W for 1 s, then Space while moving. Require stopped/Stop Status and
   zero commands. Send W for another 1 s without R: wheels must stay still.
5. R at zero, then a fresh W repeated for 1 s, then Ctrl-D. Require the client's
   matched STOP acknowledgement and serial zero/STOP/release/Open. The client
   itself labels BYE/release unconfirmed; serial is separate evidence.
6. Stop capture and restore the saved Wi-Fi even on failure. Do not retry a
   failed motion automatically. Preserve output, timestamps and serial evidence;
   obtain Todor's explicit observations of directions, every pause, Space, final
   stop and still wheels through the initial reset/startup.

About one minute. Abort on unexpected motion, missing fresh evidence, client
failure or operator concern; zero/STOP/BYE and the normal device lease provide
cleanup paths. This checks the new application on the stand only. It does not
measure electrical coast/STBY, current, temperature, loaded/floor performance
or a pin-level/physical stopping bound. M6 deferrals remain unchanged.
