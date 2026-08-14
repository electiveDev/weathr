# weathr

This is a modified version of [Veirt/weathr](https://github.com/Veirt/weathr).
This fork defaults to the DWD global ICON weather model.

A terminal weather app with real-time data, ASCII animations, automatic location detection,
and day/night scenes.

## Demo

![Thunderstorm night](docs/thunderstorm-night.gif)
![Snow](docs/snow.gif)
![Night](docs/night.gif)

## Installation

Use a binary from this repository's [Releases](https://github.com/electiveDev/weathr/releases):

```sh
curl -fsSL https://raw.githubusercontent.com/electiveDev/weathr/main/install.sh | sh
```

The installer downloads the matching release asset from `electiveDev/weathr`.

To build from this repository instead:

```sh
git clone https://github.com/electiveDev/weathr.git
cd weathr
cargo install --path .
```

A published container is available from this repository's GHCR package:

```sh
docker run --rm -it ghcr.io/electiveDev/weathr:latest
```

To build and run the image locally:

```sh
docker build -t weathr .
docker run --rm -it weathr
```

For Nix, add the fork as a flake input and install its package:

```nix
inputs.weathr.url = "github:electiveDev/weathr";
environment.systemPackages = [ inputs.weathr.packages.${system}.default ];
```

## Usage

```sh
weathr
weathr --auto-location
weathr --simulate rain
weathr --simulate snow --night
weathr --simulate clear --season spring
weathr --simulate clear --season summer
weathr --simulate clear --season autumn
weathr --simulate clear --season winter
```

`--season` follows the local calendar by default (spring: March-May, summer: June-August,
autumn: September-November, winter: December-February); pass it to override the season
for an offline preview. Simulation mode previews
weather scenes without requesting live forecast data.

Press F1 to show or hide the HUD location and quit hint (hidden by default); press q or Ctrl+C to exit. Use `weathr --help` for all options.

### Configuration

The configuration file is `~/.config/weathr/config.toml` on Linux,
`~/Library/Application Support/weathr/config.toml` on macOS, and
`%APPDATA%\\weathr\\config.toml` on Windows. A minimal configuration is:

```toml
[location]
latitude = 52.5200
longitude = 13.4050
auto = false
```

By default, weathr requests the DWD global ICON model through the Open-Meteo forecast API.
Set `auto = true` or pass `--auto-location` to estimate location from your IP via `ipinfo.io`.
City names use Nominatim/OpenStreetMap reverse geocoding.

## License and sources

The application is licensed under [GPL-3.0-or-later](LICENSE).

- Weather data: [DWD ICON](https://www.dwd.de/EN/ourservices/nwp_forecast_data/nwp_forecast_data.html)
  via [Open-Meteo](https://open-meteo.com/), licensed under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/).
- Geocoding: [Nominatim](https://nominatim.openstreetmap.org/) and
  [OpenStreetMap contributors](https://www.openstreetmap.org/copyright), licensed under [ODbL](https://opendatacommons.org/licenses/odbl/).
- ASCII art sources include [ASCII Art](https://www.asciiart.eu/), with credits for Joan G. Stark,
  Hayley Jane Wakenshaw (Flump), and other original artists retained in the project.
