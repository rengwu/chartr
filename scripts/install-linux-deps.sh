#!/bin/sh
# Ubuntu 24.04 build dependencies shared by CI and release jobs.
set -eu
sudo apt-get update -qq
sudo apt-get install -y --no-install-recommends \
    build-essential clang cmake pkg-config \
    libasound2-dev libfontconfig-dev libglib2.0-dev libssl-dev \
    libva-dev libvulkan1 libwayland-dev libx11-xcb-dev \
    libxkbcommon-x11-dev libzstd-dev libwebkit2gtk-4.1-dev
