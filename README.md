# weathr

[![Crates.io](https://img.shields.io/crates/v/weathr.svg)](https://crates.io/crates/weathr)
[![Downloads](https://img.shields.io/crates/d/weathr.svg)](https://crates.io/crates/weathr)
[![License](https://img.shields.io/crates/l/weathr.svg)](https://github.com/electiveDev/weathr/blob/main/LICENSE)

This repository is a modified version of [Veirt/weathr](https://github.com/Veirt/weathr),
originally authored by Dony Mulya. It remains licensed under GPL-3.0-or-later; this
repository's changes select DWD's global ICON model as the default weather model
(2026-08-13).

A terminal weather app with ASCII animations driven by real-time weather data.

Features real-time weather from the DWD's global ICON model via Open-Meteo, with animated rain, snow, thunderstorms, flying airplanes, day/night cycles, animated chickens and a roaming cat in the farm scene, and auto-location detection.

## Demo

### Thunderstorm Night

![Thunderstorm night demo](docs/thunderstorm-night.gif)

### Snow

![Snow demo](docs/snow.gif)

### Night

![Night demo](docs/night.gif)

## Contents

- [Installation](#installation)
- [Packaging Status](#packaging-status)
- [Configuration](#configuration)
- [Usage](#usage)
- [Privacy](#privacy)
- [Roadmap](#roadmap)
- [License](#license)

## Packaging Status

[![Packaging status](https://repology.org/badge/vertical-allrepos/weathr.svg)](https://repology.org/project/weathr/versions)<br>
![Winget](https://img.shields.io/winget/v/Veirt.weathr)

## Installation

### Quick Install (macOS, Linux, FreeBSD)

Download and install the latest binary with one command:

```sh
curl -fsSL https://raw.githubusercontent.com/electiveDev/weathr/main/install.sh | sh
```

### Via Cargo

```bash
cargo install weathr
```

### Build from Source

You need Rust installed.

```bash
git clone https://github.com/electiveDev/weathr.git
cd weathr
cargo install --path .
```

### Docker

Run the published image from GHCR:

```bash
docker run --rm -it ghcr.io/electiveDev/weathr:latest
```

Mount your config if you want to use your existing settings:

```bash
docker run --rm -it \
  -v "$HOME/.config/weathr:/.config/weathr:ro" \
  ghcr.io/electiveDev/weathr:latest
```

Build the image locally:

```bash
docker build -t weathr .
```

Run it interactively so the TUI can access your terminal:

```bash
docker run --rm -it weathr
```

Mount your config if you want to use your existing settings:

```bash
docker run --rm -it \
  -v "$HOME/.config/weathr:/.config/weathr:ro" \
  weathr
```

### Homebrew (macOS)

```bash
brew install Veirt/veirt/weathr
```

### MacPorts (macOS)

```bash
sudo port install weathr
```

### Arch Linux

Available in AUR:

```bash
yay -S weathr
```

or

```bash
yay -S weathr-bin
```

### Nix flake (NixOS)

Available as a flake:

```nix
inputs = {
    weathr.url = "github:electiveDev/weathr";
};
```

Add to packages:

```nix
environment.systemPackages = [
    inputs.weathr.packages.${system}.default
];
```

or use home-manager module option:

```nix
imports = [
    inputs.weathr.homeModules.weathr
];

programs.weathr = {
    enable = true;
    settings = {
        hide_hud = true;
    };
};
```

### Windows

Available through Winget:

```
winget install -i Veirt.weathr
```

## Configuration

The config file location depends on your platform:

- **Linux**: `~/.config/weathr/config.toml` (or `$XDG_CONFIG_HOME/weathr/config.toml`)
- **macOS**: `~/Library/Application Support/weathr/config.toml`
- **Windows**: `~/AppData/Roaming/weathr/config.toml`

### Setup

```bash
# Linux
mkdir -p ~/.config/weathr

# macOS
mkdir -p ~/Library/Application\ Support/weathr

# Windows (PowerShell)
New-Item -Path $env:APPDATA/weathr -Type Directory

# Windows (Command Prompt)
mkdir %APPDATA%/weathr
```

Edit the config file at the appropriate path for your platform:

```toml
# Hide the HUD (Heads Up Display) with weather details
hide_hud = false

# Run silently without startup messages (errors still shown)
silent = false

[location]
# Location coordinates (overridden if auto = true)
latitude = 52.5200
longitude = 13.4050

# Auto-detect location via IP (defaults to true if config missing)
auto = false

# Hide the location name in the UI
hide = false

# How to display the location in the HUD: "coordinates" | "city" | "mixed"
display = "mixed"

# Optional: manually override the city name shown in the HUD.
# When set, skips reverse geocoding entirely.
# city = "Berlin"

# Language for the resolved city name. "auto" uses the locale of the coordinates.
# Accepts BCP-47 language tags: "en", "de", "ru", "ja", etc.
# city_name_language = "auto"

[units]
# Temperature unit: "celsius" or "fahrenheit"
temperature = "celsius"

# Wind speed unit: "kmh", "ms", "mph", or "kn"
wind_speed = "kmh"

# Precipitation unit: "mm" or "inch"
precipitation = "mm"
```

### Weather Provider

By default, weathr requests the DWD's global ICON model through the Open-Meteo forecast
API (`models=icon_global`). This model is used for all locations, including Germany.
The existing Open-Meteo provider remains available as an explicit alternative:

```toml
[provider.OpenMeteo]
```

### Location Display Modes

The `display` option controls how the location appears in the HUD. City names are resolved
via reverse geocoding (Nominatim/OpenStreetMap). When a city cannot be resolved (e.g. open
sea or no Nominatim match), all modes fall back to showing coordinates.

| Mode          | City resolved                         | City not resolved            |
| :------------ | :------------------------------------ | :--------------------------- |
| `coordinates` | `Location: 52.52°N, 13.41°E`          | `Location: 52.52°N, 13.41°E` |
| `city`        | `Location: Berlin`                    | `Location: 52.52°N, 13.41°E` |
| `mixed`       | `Location: Berlin (52.52°N, 13.41°E)` | `Location: 52.52°N, 13.41°E` |

### Example Locations

```toml
# Tokyo, Japan
latitude = 35.6762
longitude = 139.6503

# Sydney, Australia
latitude = -33.8688
longitude = 151.2093
```

## Usage

Run with real-time weather:

```bash
weathr
```

### CLI Options

Simulate weather conditions for testing:

```bash
# Simulate rain
weathr --simulate rain

# Simulate snow at night
weathr --simulate snow --night

# Clear day with falling leaves
weathr --simulate clear --leaves
```

Available weather conditions:

- Clear Skies: `clear`, `partly-cloudy`, `cloudy`, `overcast`
- Precipitation: `fog`, `drizzle`, `rain`, `freezing-rain`, `rain-showers`
- Snow: `snow`, `snow-grains`, `snow-showers`
- Storms: `thunderstorm`, `thunderstorm-hail`

Override configuration:

```bash
# Use imperial units (°F, mph, inch)
weathr --imperial

# Use metric units (°C, km/h, mm) - default
weathr --metric

# Auto-detect location via IP
weathr --auto-location

# Hide location coordinates
weathr --hide-location

# Hide status HUD
weathr --hide-hud

# Run silently (suppress non-error output)
weathr --silent

# Combine flags
weathr --imperial --auto-location
```

### Keyboard Controls

- `q` or `Q` - Quit
- `Ctrl+C` - Exit

### Environment Variables

The application respects several environment variables:

- `NO_COLOR` - When set, disables all color output (accessibility feature)
- `COLORTERM` - Detects truecolor support (values: "truecolor", "24bit")
- `TERM` - Used for terminal capability detection (e.g., "xterm-256color")

Examples:

```bash
# Disable colors for accessibility
NO_COLOR=1 weathr
```

## Privacy

### Location Detection

When using `auto = true` in config or the `--auto-location` flag, the application makes a request to `ipinfo.io` to detect your approximate location based on your IP address.

This is optional. You can disable auto-location and manually specify coordinates in your config file to avoid external API calls.

## Roadmap

- [ ] Support for OpenWeatherMap, WeatherAPI, etc.
- [x] Installation via AUR.
- [ ] Key bindings for manual refresh, speed up animations, pause animations, and toggle HUD.

## License

GPL-3.0-or-later

## Credits

### Weather Data

Weather data uses the DWD's global ICON model through [Open-Meteo.com](https://open-meteo.com/) under the [CC BY 4.0 license](https://creativecommons.org/licenses/by/4.0/). The ICON model is provided by the [Deutscher Wetterdienst (DWD)](https://www.dwd.de/EN/ourservices/nwp_forecast_data/nwp_forecast_data.html).

### Geocoding

City name resolution powered by [Nominatim](https://nominatim.openstreetmap.org/) (OpenStreetMap).
Data © [OpenStreetMap contributors](https://www.openstreetmap.org/copyright), licensed under [ODbL](https://opendatacommons.org/licenses/odbl/).

### ASCII Art

- **Source**: https://www.asciiart.eu/
- **House**: Joan G. Stark
- **Airplane**: Joan G. Stark
- **Sun**: Hayley Jane Wakenshaw (Flump)
- **Moon**: Joan G. Stark

_Note: If any ASCII art is uncredited or misattributed, it belongs to the original owner._
