# Waver

> Simple tool for working with the Wave XLR over USB

## What?

This project is split up into three main parts:

- `waver-core` crate is a crate for interacting with the Wave XLR programmatically over USB (Built using the knowledge of the USB protocol from this other project that someone has reverse engineered it for https://github.com/rikkichy/openwave/).
- `waver-cli` is a CLI tool for using the `waver-core` functionality
- `waver-service` is a background service which tracks the configuration of the Wave XLR and stores it to a file which is automatically restored from on startup as soon as the USB device is connected.

## Why?

Have a Wave XLR? Switched to Linux and got blasted with feedback through your microphone and headphones? Well I have,
that's what this project fixes for me.

By default the Linux configuration doesn't understand and save the special Wave XLR data that says how much it
should mix the Headphones and Microphone audio. In my case it decides to reset that to zero every time I reboot
(Full feedback maximum volume, basically play the microphone as loud as you can through the headphones) and my microphone
happens to be near my computer fans meaning when my computer would startup I'd get blasted with feedback

## Installation

To install the background service you can run the automated script in `./scripts/waver-startup.sh`. You can do this in one line using the following command if you have curl installed:

```
curl --proto '=https' --tlsv1.2 -LsSf https://raw.githubusercontent.com/jacobtread/waver/refs/heads/main/scripts/waver-startup.sh | sh
```

This will add the `waver-service` binary to `/usr/local/bin/waver-service`, it will also create a service in `/etc/systemd/system/waver-service.service`. The script will also create `/etc/udev/rules.d/99-waver.rules` which is a rule that will start the server when the Wave XLR USB device is added.
