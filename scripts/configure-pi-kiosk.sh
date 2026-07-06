#!/usr/bin/env bash
set -euo pipefail

config_root="${XDG_CONFIG_HOME:-$HOME/.config}"
panel_config="$config_root/wf-panel-pi.ini"
labwc_dir="$config_root/labwc"
labwc_config="$labwc_dir/rc.xml"

mkdir -p "$config_root"
touch "$panel_config"

if grep -q '^autohide=' "$panel_config"; then
    sed -i 's/^autohide=.*/autohide=1/' "$panel_config"
else
    printf '\nautohide=1\n' >> "$panel_config"
fi

mkdir -p "$labwc_dir"
if [[ ! -f "$labwc_config" ]]; then
    if [[ -f /etc/xdg/labwc/rc.xml ]]; then
        cp /etc/xdg/labwc/rc.xml "$labwc_config"
    else
        printf '%s\n' '<?xml version="1.0"?>' '<labwc_config>' '</labwc_config>' > "$labwc_config"
    fi
fi

cp -n "$labwc_config" "$labwc_config.codex-backup" 2>/dev/null || true

if ! grep -q 'identifier="pi-constellation-mapper"' "$labwc_config"; then
    if grep -q '</windowRules>' "$labwc_config"; then
        sed -i '/<\/windowRules>/i\
    <windowRule identifier="pi-constellation-mapper" serverDecoration="no">\
      <action name="ToggleFullscreen" />\
    </windowRule>' "$labwc_config"
    elif grep -q '</labwc_config>' "$labwc_config"; then
        sed -i '/<\/labwc_config>/i\
  <windowRules>\
    <windowRule identifier="pi-constellation-mapper" serverDecoration="no">\
      <action name="ToggleFullscreen" />\
    </windowRule>\
  </windowRules>' "$labwc_config"
    elif grep -q '</openbox_config>' "$labwc_config"; then
        sed -i '/<\/openbox_config>/i\
  <windowRules>\
    <windowRule identifier="pi-constellation-mapper" serverDecoration="no">\
      <action name="ToggleFullscreen" />\
    </windowRule>\
  </windowRules>' "$labwc_config"
    else
        printf 'Could not find a supported labwc root closing tag in %s\n' "$labwc_config" >&2
        exit 1
    fi
fi

if command -v labwc >/dev/null 2>&1 && [[ -n "${WAYLAND_DISPLAY:-}" ]]; then
    labwc --reconfigure || true
fi

printf 'Enabled panel autohide in %s\n' "$panel_config"
printf 'Installed the Pi Constellation Mapper fullscreen rule in %s\n' "$labwc_config"
printf 'Close and relaunch the starmap. Log out and back in if the rule is not applied immediately.\n'
