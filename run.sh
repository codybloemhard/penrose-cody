#!/bin/bash
xrandr --auto
feh --bg-fill ~/img/background.jpg
xrandr --output DP-0 --output HDMI-0 --right-of DP-0 &
# exec --no-startup-id ~/scripts/run-shapebar
# exec picom --xrender-sync-fence
cargo run --release
