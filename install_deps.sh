#!/bin/sh
set -e

# Apt packages
sudo apt update
sudo apt upgrade -y
sudo apt install gcc clang make automake autoconf libtool pkg-config ffmpeg -y

# Youtube dl
sudo curl -L https://yt-dl.org/downloads/latest/youtube-dl -o /usr/local/bin/youtube-dl
sudo chmod a+rx /usr/local/bin/youtube-dl