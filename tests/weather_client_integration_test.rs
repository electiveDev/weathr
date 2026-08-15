use std::sync::Arc;
use std::time::Duration;
use weathr::weather::provider::WeatherProvider;
use weathr::weather::{
    IconGlobalProvider, OpenMeteoProvider, WeatherClient, WeatherLocation, WeatherUnits,
};

#[tokio::test]
async fn test_weather_client_integration_cache_behavior() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let weather1 = client
        .get_current_weather(&location, &units, weathr::config::Provider::OpenMeteo)
        .await
        .expect("First fetch should succeed");

    let weather2 = client
        .get_current_weather(&location, &units, weathr::config::Provider::OpenMeteo)
        .await
        .expect("Second fetch should succeed");

    assert_eq!(
        weather1.timestamp, weather2.timestamp,
        "Second fetch should return cached data"
    );
}

#[tokio::test]
async fn test_weather_client_integration_cache_invalidation() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let _weather1 = client
        .get_current_weather(&location, &units, weathr::config::Provider::OpenMeteo)
        .await
        .expect("First fetch should succeed");

    client.invalidate_cache().await;

    let weather2 = client
        .get_current_weather(&location, &units, weathr::config::Provider::OpenMeteo)
        .await
        .expect("Fetch after invalidation should succeed");

    assert!(
        weather2.temperature >= -90.0 && weather2.temperature <= 60.0,
        "Weather data should still be valid after cache invalidation"
    );
}

#[tokio::test]
async fn test_weather_client_integration_realistic_weather_ranges() {
    let provider = Arc::new(OpenMeteoProvider::new());
    let client = WeatherClient::new(provider, Duration::from_secs(60));

    let location = WeatherLocation {
        latitude: 0.0,
        longitude: 0.0,
        elevation: None,
    };

    let units = WeatherUnits::default();

    let weather = client
        .get_current_weather(&location, &units, weathr::config::Provider::OpenMeteo)
        .await
        .expect("Should fetch weather");

    assert!(
        weather.temperature >= -90.0 && weather.temperature <= 60.0,
        "Temperature should be within realistic range"
    );
    assert!(
        weather.wind_speed >= 0.0 && weather.wind_speed <= 500.0,
        "Wind speed should be realistic"
    );
    assert!(
        weather.wind_direction >= 0.0 && weather.wind_direction <= 360.0,
        "Wind direction should be 0-360 degrees"
    );
    assert!(
        weather.precipitation >= 0.0,
        "Precipitation should be non-negative"
    );
}

#[tokio::test]
async fn test_icon_d2_weather_is_plausible_in_germany() {
    let provider = IconGlobalProvider::new();
    let location = WeatherLocation {
        latitude: 52.52,
        longitude: 13.41,
        elevation: None,
    };

    let weather = provider
        .get_current_weather(&location, &WeatherUnits::default())
        .await
        .expect("ICON D2 should return weather for Berlin");

    assert!(
        (-90.0..=60.0).contains(&weather.temperature),
        "Temperature should be realistic: {}",
        weather.temperature
    );
    assert!(
        (0.0..=500.0).contains(&weather.precipitation),
        "Precipitation should be realistic: {}",
        weather.precipitation
    );
    assert!(
        (0.0..=500.0).contains(&weather.wind_speed),
        "Wind speed should be realistic: {}",
        weather.wind_speed
    );
    assert!(
        (0.0..=360.0).contains(&weather.wind_direction),
        "Wind direction should be valid: {}",
        weather.wind_direction
    );
    assert!(!weather.timestamp.is_empty());
    assert_eq!(weather.attribution, "DWD ICON D2 via Open-Meteo");
}
