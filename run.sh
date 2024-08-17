#!/bin/sh
exec xrandr --output DP-0 --output HDMI-0 --right-of DP-0 &
# exec ~/scripts/.theme/run-shapebar &
exec picom --xrender-sync-fence &
cargo run --release
