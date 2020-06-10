#!/bin/sh
set -e

if [ "$EUID" -ne 0 ]
  then echo "Please run as root"
  exit
fi

# Apt packages
apt update
apt install gcc clang make automake autoconf libtool pkg-config ffmpeg curl -y

# Youtube dl
curl -L https://yt-dl.org/downloads/latest/youtube-dl -o /usr/local/bin/youtube-dl
chmod a+rx /usr/local/bin/youtube-dl

echo "Installed 'gcc' 'clang' 'make' 'automake' 'autoconf' 'libtool' 'pkg-config' 'ffmpeg' and 'youtube-dl` `curl" 