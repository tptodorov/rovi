#!/usr/bin/env python3
"""Plots what the car's own log says it commanded: the wheel commands over time, and the path they integrate to.
This is the commanded motion, not measured motion (that needs the camera). Needs matplotlib (shell.nix).

    python3 sim/plot_car.py car.log out.png     # the last boot in the log
"""
import math, sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt

import carlog

R, L, MAX_W = 0.05, 0.2, 40.0  # wheel radius m, centre projection sum m, rad/s at command 1.0: the firmware GEOMETRY, placeholders until M5


def body_velocity(w):
    """Forward mecanum kinematics, the inverse of src/kinematics.rs; wheel order FL, FR, RR, RL."""
    a, b, c, d = (x * MAX_W for x in w)
    return R / 4 * (a + b + c + d), R / 4 * (-a + b - c + d), R / (4 * L) * (-a + b + c - d)


def commanded_path(every):
    """(t s, x m, y m, heading rad) from the first `ev=state` of one boot's events, starting at 0; each wheel command
    holds until the next state event, the last one until the last logged event. None without state events."""
    ev = [e for e in every if e["ev"] == "state"]
    if not ev:
        return None
    t0, end = ev[0]["at_ms"], every[-1]["at_ms"]
    t = [(e["at_ms"] - t0) / 1000 for e in ev]
    x = y = th = 0.0
    path = [(0.0, 0.0, 0.0, 0.0)]
    for e, nxt in zip(ev, t[1:] + [(end - t0) / 1000]):
        vx, vy, wz = body_velocity([w / 1000 for w in e["wheels_pm"]])
        dt = nxt - (e["at_ms"] - t0) / 1000
        x += (vx * math.cos(th) - vy * math.sin(th)) * dt
        y += (vx * math.sin(th) + vy * math.cos(th)) * dt
        th += wz * dt
        path.append((nxt, x, y, th))
    return path


def main(path, out):
    every = carlog.segments(carlog.read(path))[-1]  # the last boot
    pose = commanded_path(every)
    if not pose:
        sys.exit("no state events in the log")
    ev = [e for e in every if e["ev"] == "state"]
    t0, end = ev[0]["at_ms"], every[-1]["at_ms"]
    t = [(e["at_ms"] - t0) / 1000 for e in ev]
    path_xy, th = [(x, y) for _, x, y, _ in pose], pose[-1][3]
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(11, 4))
    for i, name in enumerate(("FL", "FR", "RR", "RL")):
        ax1.step(t + [(end - t0) / 1000], [e["wheels_pm"][i] / 1000 for e in ev] + [ev[-1]["wheels_pm"][i] / 1000], where="post",
                 label=name, ls=("-", "--", ":", "-.")[i], lw=2)
    ax1.set(xlabel="s", ylabel="wheel command", title="wheel commands"), ax1.legend()
    ax2.plot(*zip(*path_xy), marker=".")
    ax2.set(xlabel="x m (forward)", ylabel="y m (left)", title=f"commanded path, final heading {math.degrees(th):.0f} deg", aspect="equal")
    fig.tight_layout()
    fig.savefig(out, dpi=80)


if __name__ == "__main__":
    main(*sys.argv[1:3])
