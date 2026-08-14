# weathr feature reference

This page records the behavior implemented by the current `weathr` binary. It is a user-facing reference for the seasonal renderer, weather simulation, animation layers, HUD, and keyboard controls.

## Seasons

The season is resolved from the local calendar when the application starts:

| Months | Season | CLI value |
| --- | --- | --- |
| March–May | Spring | `spring` |
| June–August | Summer | `summer` |
| September–November | Autumn | `autumn` |
| December–February | Winter | `winter` |

The month mapping is local-time based. December, January, and February all map to winter; there is no separate year-boundary case.

`--season <spring|summer|autumn|winter>` overrides the calendar season for the current run. The override is independent of the weather provider, so it can be used with live weather or with `--simulate` for a repeatable preview. It is an application-rendering override, not a change to the reported weather condition.

## Seasonal world design

The world scene always contains the centered house and its ground, pond, fence, and mailbox when they fit the terminal layout. A second pine tree is added on sufficiently wide terminals. The day/night palette is selected first; the season then adjusts the vegetation and ground styling.

- **Spring:** bright green daytime grass, green foliage, more frequent flowers, and a blossoming tree. The spring flower palette includes white as well as colored flowers.
- **Summer:** green grass and full foliage, with a lower flower frequency than spring.
- **Autumn:** yellow/brown ground and foliage, autumn-colored flowers, and an autumn tree with visible accent marks.
- **Winter:** no flowers and a bare tree; foliage uses subdued brown shades, while the base grass/ground palette remains day/night dependent.

The seasonal world styling is separate from precipitation animations. For example, `--leaves` explicitly enables the falling-leaves foreground system; it is not silently enabled or disabled by the calendar season. Falling leaves are suppressed while rain, thunderstorms, or snow are active.

## Weather conditions and sky layers

`--simulate` accepts the following condition values:

```text
clear             partly-cloudy       cloudy              overcast
fog               drizzle             rain                freezing-rain
rain-showers      snow                snow-grains         snow-showers
thunderstorm      thunderstorm-hail
```

Condition names are case-insensitive, and underscores are accepted where the CLI normalizes them to the displayed hyphenated names.

### Clear versus cloudy

`clear` is intentionally not a cloudy condition. The weather model reports `is_cloudy = false`, so the cloud animation is inactive and no cloud glyphs are rendered. During a simulated clear day, the sun layer can render; at night, the sun is hidden and the night layers can take over.

`partly-cloudy`, `cloudy`, and `overcast` report `is_cloudy = true` and activate the moving cloud background layer. Partly cloudy clouds use a lighter grey; cloudy and overcast clouds use dark grey. Cloud movement receives the current wind speed and direction. Rain, snow, and thunderstorms additionally enable their corresponding foreground effects and suppress the daytime sun and bird systems where applicable.

## Birds

Birds are a background animation system. It maintains at most three birds, spawns them occasionally in the upper third of the terminal, moves them horizontally, and alternates the `v` and `-` characters to flap their wings. Every bird is rendered with `Color::White`, independent of the active theme palette.

The bird system is active during the day when rain, thunderstorms, and snow are not active. It can therefore appear over clear or cloud-covered daytime conditions (and is not itself the cloud layer). It is inactive at night and during the excluded precipitation/storm conditions.

## Simulation and useful commands

Simulation constructs a local weather state instead of starting a live weather-provider request. It is useful for checking one scene without network data. Simulated weather uses a temperature of 20 °C, a precipitation value when the condition rains, and stronger wind for thunderstorms; the relevant animation systems continue to animate normally.

Examples:

```sh
# Live weather, with the local-calendar season
weathr

# Offline weather previews
weathr --simulate clear
weathr --simulate clear --season spring
weathr --simulate clear --season summer
weathr --simulate clear --season autumn
weathr --simulate clear --season winter
weathr --simulate snow --night
weathr --simulate thunderstorm
```

The relevant options are:

| Option | Effect |
| --- | --- |
| `-s, --simulate <CONDITION>` | Render one of the supported weather conditions without live forecast data. |
| `-n, --night` | In simulation mode, force night celestial conditions (moon/stars/fireflies can then be exercised). |
| `--season <SEASON>` | Force `spring`, `summer`, `autumn`, or `winter` instead of the local calendar season. |
| `-l, --leaves` | Enable falling leaves when rain, thunderstorms, and snow are not active. |
| `--hide-hud` | Hide the complete HUD status line. |
| `--hide-location` | Keep weather details but omit location information from the HUD. |
| `--auto-location` | Enable location detection through the configured IP-based lookup. |
| `--metric` / `--imperial` | Select metric or imperial display units; the two flags conflict. |
| `--silent` | Suppress non-error startup output. |
| `--help` | Show the complete CLI help, including all simulation values. |

## HUD and keyboard controls

The HUD is shown by default unless configuration or `--hide-hud` disables it. The initial HUD view contains the weather summary (condition, temperature, wind, and precipitation). The location and quit hint are detail text hidden by default.

- **F1** toggles the detail portion of the HUD. It does not hide the weather summary. When details are shown, the HUD can include location information and `Press 'q' to quit` (unless location display is disabled).
- **`q` or `Q`** exits the application.
- **Ctrl+C** exits the application through the normal signal path.

The terminal renderer uses an alternate screen and restores the terminal when the application exits or handles a panic. If the HUD is hidden, F1 still changes the internal detail toggle but no HUD line is displayed.

## Source cross-check

The behavior above is implemented by `src/season.rs`, `src/cli.rs`, `src/main.rs`, `src/app.rs`, `src/app_state.rs`, `src/scene/world/style.rs`, `src/animation_manager.rs`, `src/animation/clouds.rs`, `src/animation/birds.rs`, and the weather condition helpers under `src/weather/`. This page deliberately contains no deployment hostnames, addresses, process IDs, credentials, or other private operational data.
