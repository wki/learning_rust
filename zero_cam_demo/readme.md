# Readme

simple v4l demo program for Raspberry Pi Zero 2w


# Prerequisites

    cargo install cross --git https://github.com/cross-rs/cross


# Compiling with cross -- fails due to libcam...
    cross build --target aarch64-unknown-linux-gnu -r
    ls -l target/aarch64-unknown-linux-gnu/release/
    scp target/aarch64-unknown-linux-gnu/release/zero_cam_demo 192.168.2.213:

# Building my own docker container -- fails due to libcam 0.4/0.5 mismatch

    docker build -t debian .
    docker run --rm -v `PWD`:/app -w /app debian sh -l -c 'cargo build -r' 
    scp target/release/zero_cam_demo 192.168.2.106:
    ssh 192.168.2.106

# building on a respberry Pi 5

    rsync -avxSH . --exclude target 192.168.2.107:zero_cam_demo/
    # on Rapberry Pi 5:
    cargo build -r && echo 'copying...' && scp target/release/zero_cam_demo 192.168.2.106:
    # on zero:
    ./zero_cam_demo
