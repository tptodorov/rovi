#!/usr/bin/env python3
"""Overlays the path the camera saw (scripts/track_marker.py CSV, with --side-m) on the path the car log commanded
(plot_car.commanded_path), and prints the drift between them. Needs matplotlib (shell.nix).

    python3 sim/compare_track.py car.log track.csv out.png [--start-us EPOCH_US]

The car clock and the camera clock are not linked, so the first wheel command is aligned with the first frame where
the marker moved (or with --start-us, the camera epoch of the first command). The observed path is turned into the
car's start frame (x forward, y left) assuming the marker's top edge points to the car's front. Drift is compared at
the end of the commanded motion, or at the last frame if the recording stops earlier.
"""
import argparse, csv, math, sys

import matplotlib
matplotlib.use("Agg")
import matplotlib.pyplot as plt
import numpy as np

import carlog, plot_car

MIN_MOVE_M = 0.005  # marker displacement that counts as motion; above the detection jitter


def commanded(every):
    """plot_car.commanded_path from the first non-zero wheel command on, its time shifted to start there."""
    pose = plot_car.commanded_path(every)
    if not pose:
        sys.exit("no state events in the log")
    ev = [e for e in every if e["ev"] == "state"]
    k = next((i for i, e in enumerate(ev) if any(e["wheels_pm"])), None)
    if k is None:
        sys.exit("the log commands no wheel motion")
    t0 = pose[k][0]
    return [(t - t0, x, y, th) for t, x, y, th in pose[k:]]


def read_track(path):
    """(epoch_us, x_m, y_m, heading_rad) rows of a track_marker.py CSV; it must have been written with --side-m."""
    with open(path, newline="") as f:
        rows = list(csv.DictReader(f))
    if not rows or "x_m" not in rows[0]:
        sys.exit(f"{path}: no rows, or no x_m/y_m columns (run track_marker.py with --side-m)")
    return [(int(r["epoch_us"]), float(r["x_m"]), float(r["y_m"]), float(r["heading_rad"])) for r in rows]


def motion_start_us(track, min_move=MIN_MOVE_M):
    """The epoch of the last frame before the marker first moved more than min_move from where it started."""
    _, x0, y0, _ = track[0]
    for prev, (t, x, y, _) in zip(track, track[1:]):
        if math.hypot(x - x0, y - y0) > min_move:
            return prev[0]
    return track[0][0]


def observed(track, start_us):
    """(t s, x m, y m, heading rad) from start_us on, in the car's frame at that moment; heading unwrapped."""
    i = max((j for j, r in enumerate(track) if r[0] <= start_us), default=0)
    _, x0, y0, h0 = track[i]
    c, s = math.cos(h0), math.sin(h0)
    rest = track[i:]
    heading = np.unwrap([h for *_, h in rest]) - h0
    return [((t - start_us) / 1e6, c * (x - x0) + s * (y - y0), -s * (x - x0) + c * (y - y0), float(th))
            for (t, x, y, _), th in zip(rest, heading)]


def at(path, t):
    """(x, y, heading) of a (t, x, y, heading) path at time t, linear between samples."""
    ts = [p[0] for p in path]
    return tuple(float(np.interp(t, ts, [p[i] for p in path])) for i in (1, 2, 3))


def drift(cmd, obs):
    """Commanded against observed at the end of the commanded motion (or the last frame): distance from the start,
    heading change, and the gap between the two end points."""
    t = min(cmd[-1][0], obs[-1][0])
    (cx, cy, ch), (ox, oy, oh) = at(cmd, t), at(obs, t)
    cd, od = math.hypot(cx, cy), math.hypot(ox, oy)
    return {"t_s": t, "cmd_dist_m": cd, "obs_dist_m": od, "dist_err_m": od - cd, "pos_err_m": math.hypot(ox - cx, oy - cy),
            "cmd_heading_rad": ch, "obs_heading_rad": oh, "heading_err_rad": oh - ch}


def main(argv=None):
    a = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    a.add_argument("log", help="car log (the last boot is used)")
    a.add_argument("track", help="CSV from scripts/track_marker.py --side-m")
    a.add_argument("out", help="PNG to write")
    a.add_argument("--start-us", type=int, help="camera epoch of the first wheel command (default: first marker motion)")
    args = a.parse_args(argv)
    cmd = commanded(carlog.segments(carlog.read(args.log))[-1])
    track = read_track(args.track)
    start = args.start_us if args.start_us is not None else motion_start_us(track)
    obs = observed(track, start)
    d = drift(cmd, obs)
    fig, ax = plt.subplots(figsize=(6, 5))
    for path, (x, y, h), style in ((cmd, at(cmd, d["t_s"]), dict(marker=".", label="commanded (car log)")),
                                   (obs, at(obs, d["t_s"]), dict(ls="--", label="observed (camera)"))):
        line, = ax.plot([p[1] for p in path], [p[2] for p in path], **style)
        ax.quiver(x, y, math.cos(h), math.sin(h), color=line.get_color(), width=0.008)  # heading at the compare time
    ax.set_aspect("equal", adjustable="datalim")
    ax.margins(0.2)
    ax.set(xlabel="x m (forward at start)", ylabel="y m (left at start)",
           title=f"at {d['t_s']:.1f} s: distance {d['dist_err_m']:+.3f} m, heading {math.degrees(d['heading_err_rad']):+.0f} deg")
    ax.legend()
    fig.tight_layout()
    fig.savefig(args.out, dpi=80)
    print(f"start {start} us; compared at {d['t_s']:.2f} s after the first wheel command")
    print(f"distance  commanded {d['cmd_dist_m']:.3f} m  observed {d['obs_dist_m']:.3f} m  drift {d['dist_err_m']:+.3f} m")
    print(f"heading   commanded {math.degrees(d['cmd_heading_rad']):.1f} deg  observed {math.degrees(d['obs_heading_rad']):.1f} deg"
          f"  drift {math.degrees(d['heading_err_rad']):+.1f} deg")
    print(f"end point gap {d['pos_err_m']:.3f} m -> {args.out}")
    return d


if __name__ == "__main__":
    main()
