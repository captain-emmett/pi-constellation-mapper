#!/usr/bin/env bash
set -euo pipefail

config_root="${XDG_CONFIG_HOME:-$HOME/.config}"
panel_config="$config_root/wf-panel-pi.ini"

mkdir -p "$config_root"
touch "$panel_config"

if grep -q '^autohide=' "$panel_config"; then
    sed -i 's/^autohide=.*/autohide=1/' "$panel_config"
else
    printf '\nautohide=1\n' >> "$panel_config"
fi

printf 'Enabled Raspberry Pi panel autohide in %s\n' "$panel_config"
printf 'Log out and back in, or reboot the Pi, for the panel change to take effect.\n'
