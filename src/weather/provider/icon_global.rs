use async_trait::async_trait;

use crate::error::WeatherError;
use crate::weather::provider::open_meteo::OpenMeteoProvider;
use crate::weather::provider::{WeatherProvider, WeatherProviderResponse};
use crate::weather::types::{WeatherLocation, WeatherUnits};

const ICON_D2_MODEL: &str = "dwd_icon_d2";
const ICON_D2_ATTRIBUTION: &str = "DWD ICON D2 via Open-Meteo";

/// Weather provider backed by the DWD's ICON-D2 model through Open-Meteo.
pub struct IconGlobalProvider {
    delegate: OpenMeteoProvider,
}

impl IconGlobalProvider {
    pub fn new() -> Self {
        Self {
            delegate: OpenMeteoProvider::for_model(ICON_D2_MODEL, ICON_D2_ATTRIBUTION),
        }
    }
}

impl Default for IconGlobalProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WeatherProvider for IconGlobalProvider {
    fn get_attribution(&self) -> &'static str {
        ICON_D2_ATTRIBUTION
    }

    async fn get_current_weather(
        &self,
        location: &WeatherLocation,
        units: &WeatherUnits,
    ) -> Result<WeatherProviderResponse, WeatherError> {
        self.delegate.get_current_weather(location, units).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uses_dwd_icon_d2_attribution() {
        assert_eq!(
            IconGlobalProvider::new().get_attribution(),
            ICON_D2_ATTRIBUTION
        );
    }
}
