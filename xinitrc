# xinitrc file for testing our window manager.

# 1. Start some programs to play with.
# xclock &
# xeyes &
# sleep 2 && xterm &
# sleep 2 && kitty &
nitrogen --restore &
polybar & 

# 2. Start our window manager.
export GLOG_logtostderr=1
exec ./target/release/E3WM
