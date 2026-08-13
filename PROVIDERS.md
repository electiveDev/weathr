# weathr data providers

How to use different providers & how to implement them

## Using Providers

There are currently 3 available providers:
- IconGlobal (default): DWD's global ICON model through Open-Meteo
- OpenMeteo: Open-Meteo's automatic model selection
- MetOffice

Currently the only way to change the provider is done via the config.

### IconGlobal Provider

The default provider requests `models=icon_global` from Open-Meteo's forecast API.
This selects the DWD global ICON model worldwide, including locations in Germany.
No API key is required.

To make the default explicit in your config:
```
[provider.IconGlobal]
```

### OpenMeteo Provider

The existing Open-Meteo provider remains available as an explicit fallback. It uses
Open-Meteo's automatic model selection rather than forcing a specific model:
```
[provider.OpenMeteo]
```

### [MetOffice](metoffice.gov.uk) Provider
This is the [UK Government Met Office](metoffice.gov.uk) weather provider
#### Getting your API key
[https://login.auth.metoffice.cloud/](https://login.auth.metoffice.cloud/)

#### Enabling
To enable simply add to your config once you have your API Key
```
[provider.MetOffice]
# Met Office API key
api_key = "YOUR MET OFFICE API KEY"
```

## Supplementary Providers
Currently there is 1 Sup-provider which is the US Government Astronomical Applications Department

These types of providers are meant to be small & suppliment other providers data in the event they are missing data, an example is the MetOffice provider doesn't provide any atronomical data, instead the provider will make another request to get that data

## Guide: Adding providers

### Creating the provider
There are 2 types of providers a `WeatherProvider` and a `SupplementaryWeatherProvider`, a provider can be both a supplementary provider and a "primary" provider

#### Where to place your provider
`src/weather/provider`
#### Must Haves
Your new provider must use a trait, it can be either `WeatherProvider` or `SupplementaryWeatherProvider`

### Making the primary provider useful
Add in `src/config.rs` the `Provider` enum the provider name

Then `src/app.rs` in `App::new` a match at line 139 to map the `Provider` enum to a provider, there is where you add your provider's initialisation 

#### Provider Configs
A providers config there is no format a provider config will look, only expected fields, currently if a field is missing panic with a nice message, an example would be the `MetOffice` provider

### Making the supplementary provider useful
Currently supplementary are ad-hoc, the trait is useful for further improvements