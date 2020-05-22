#!/bin/sh

# Apt packages
sudo apt update
sudo apt upgrade -y
Sudo apt install gcc libclang make automake autoconf libtool pkg-config audiopus_sys ffmpeg -y


# Youtube dl
sudo curl -L https://yt-dl.org/downloads/latest/youtube-dl -o /usr/local/bin/youtube-dl
sudo chmod a+rx /usr/local/bin/youtube-dl

