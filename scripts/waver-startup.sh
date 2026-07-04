#!/bin/bash
set -euo pipefail

# This script installs waver-service and sets it up to run when the USB
# device is ready to go

if command -v waver >/dev/null 2>&1; then
    echo "waver is already installed."
    exit 0
fi

BINARY_URL="https://github.com/jacobtread/waver/releases/download/0.2.0/waver-service-x86_64-unknown-linux-gnu.tar.xz"
curl -sL "$BINARY_URL" | sudo tar -xJ -C /usr/local/bin/ waver-service
sudo chmod +x /usr/local/bin/waver-service

# Device
VID="0fd9"
PID="007d"

# Setup startup service
SERVICE_FILE="/etc/systemd/system/waver-service.service"
sudo tee "$SERVICE_FILE" > /dev/null << 'EOF'
[Unit]
Description=Immediate Waver USB Property Initializer
DefaultDependencies=no

[Service]
Type=oneshot
ExecStart=/usr/local/bin/waver-service
RemainAfterExit=yes
EOF
sudo systemctl daemon-reload

# Setup udev rule to trigger the service when the device is available
UDEV_FILE="/etc/udev/rules.d/99-waver.rules"
echo "SUBSYSTEM==\"usb\", ACTION==\"add\", ATTRS{idVendor}==\"$VID\", ATTRS{idProduct}==\"$PID\", TAG+=\"systemd\", ENV{SYSTEMD_WANTS}+=\"waver-service.service\"" > "$UDEV_FILE"
sudo udevadm control --reload-rules
sudo udevadm trigger
