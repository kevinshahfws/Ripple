// Copyright 2023 Comcast Cable Communications Management, LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0
//

use jsonrpsee::{
    core::{async_trait, RpcResult},
    proc_macros::rpc,
    tracing::error,
    RpcModule,
};
use ripple_sdk::{
    api::{
        device::{
            device_info_request::DeviceInfoRequest,
            device_peristence::SetStringProperty,
            device_request::{LanguageProperty, TimezoneProperty},
        },
        firebolt::fb_general::{ListenRequest, ListenerResponse},
        gateway::rpc_gateway_api::CallContext,
        storage_property::{StorageProperty, EVENT_TIMEZONE_CHANGED, KEY_POSTAL_CODE},
    },
    extn::extn_client_message::ExtnResponse,
};
use std::collections::HashMap;

use crate::utils::rpc_utils::{rpc_add_event_listener, rpc_err};
use crate::{
    firebolt::rpc::RippleRPCProvider, processor::storage::storage_manager::StorageManager,
    service::apps::provider_broker::ProviderBroker, state::platform_state::PlatformState,
};

#[rpc(server)]
pub trait Localization {
    #[method(name = "localization.locality")]
    async fn locality(&self, ctx: CallContext) -> RpcResult<String>;
    #[method(name = "localization.setLocality")]
    async fn locality_set(&self, ctx: CallContext, set_request: SetStringProperty)
        -> RpcResult<()>;
    #[method(name = "localization.onLocalityChanged")]
    async fn on_locality_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;
    #[method(name = "localization.countryCode")]
    async fn country_code(&self, ctx: CallContext) -> RpcResult<String>;
    #[method(name = "localization.setCountryCode")]
    async fn country_code_set(
        &self,
        ctx: CallContext,
        set_request: SetStringProperty,
    ) -> RpcResult<()>;
    #[method(name = "localization.onCountryCodeChanged")]
    async fn on_country_code_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;
    #[method(name = "localization.language")]
    async fn language(&self, ctx: CallContext) -> RpcResult<String>;
    #[method(name = "localization.setLanguage")]
    async fn language_set(&self, ctx: CallContext, set_request: LanguageProperty) -> RpcResult<()>;
    #[method(name = "localization.onLanguageChanged")]
    async fn on_language_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;
    #[method(name = "localization.postalCode")]
    async fn postal_code(&self, _ctx: CallContext) -> RpcResult<String>;
    #[method(name = "localization.setPostalCode")]
    async fn postal_code_set(
        &self,
        ctx: CallContext,
        set_request: SetStringProperty,
    ) -> RpcResult<()>;
    #[method(name = "localization.onPostalCodeChanged")]
    async fn on_postal_code_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;
    #[method(name = "localization.locale")]
    async fn locale(&self, _ctx: CallContext) -> RpcResult<String>;
    #[method(name = "localization.setLocale")]
    async fn locale_set(&self, ctx: CallContext, set_request: SetStringProperty) -> RpcResult<()>;
    #[method(name = "localization.onLocaleChanged")]
    async fn on_locale_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;
    #[method(name = "localization.latlon")]
    async fn latlon(&self, _ctx: CallContext) -> RpcResult<String>;
    #[method(name = "localization.setLatlon")]
    async fn latlon_set(&self, ctx: CallContext, set_request: SetStringProperty) -> RpcResult<()>;
    #[method(name = "localization.onLatlonChanged")]
    async fn on_latlon_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;
    #[method(name = "localization.additionalInfo")]
    async fn additional_info(&self, _ctx: CallContext) -> RpcResult<HashMap<String, String>>;
    #[method(name = "localization.setAdditionalInfo")]
    async fn additional_info_set(
        &self,
        ctx: CallContext,
        set_request: SetStringProperty,
    ) -> RpcResult<()>;
    #[method(name = "localization.onAdditionalInfoChanged")]
    async fn on_additional_info_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;
    #[method(name = "localization.setTimeZone")]
    async fn timezone_set(&self, ctx: CallContext, set_request: TimezoneProperty) -> RpcResult<()>;
    #[method(name = "localization.timeZone")]
    async fn timezone(&self, ctx: CallContext) -> RpcResult<String>;
    #[method(name = "localization.onTimeZoneChanged")]
    async fn on_timezone_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse>;
}

#[derive(Debug)]
pub struct LocalizationImpl {
    pub platform_state: PlatformState,
}

impl LocalizationImpl {
    pub async fn postal_code(state: &PlatformState, app_id: String) -> Option<String> {
        match StorageManager::get_string(state, StorageProperty::PostalCode).await {
            Ok(resp) => Some(resp),
            Err(_) => {
                match StorageManager::get_string_from_namespace(state, app_id, KEY_POSTAL_CODE)
                    .await
                {
                    Ok(resp) => Some(resp.as_value()),
                    Err(_) => None,
                }
            }
        }
    }

    pub async fn on_request_app_event(
        &self,
        ctx: CallContext,
        request: ListenRequest,
        method: &'static str,
        event_name: &'static str,
    ) -> RpcResult<ListenerResponse> {
        let listen = request.listen;
        ProviderBroker::register_or_unregister_provider(
            &self.platform_state,
            // TODO update with Firebolt Cap in later effort
            "xrn:firebolt:capability:localization:locale".into(),
            method.into(),
            event_name,
            ctx,
            request,
        )
        .await;

        Ok(ListenerResponse {
            listening: listen,
            event: event_name.into(),
        })
    }
}

#[async_trait]
impl LocalizationServer for LocalizationImpl {
    async fn locality(&self, _ctx: CallContext) -> RpcResult<String> {
        StorageManager::get_string(&self.platform_state, StorageProperty::Locality).await
    }

    async fn locality_set(
        &self,
        _ctx: CallContext,
        set_request: SetStringProperty,
    ) -> RpcResult<()> {
        StorageManager::set_string(
            &self.platform_state,
            StorageProperty::Locality,
            set_request.value,
            None,
        )
        .await
    }

    async fn on_locality_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        self.on_request_app_event(
            ctx,
            request,
            "LocalizationLocalityChanged",
            "localization.onLocalityChanged",
        )
        .await
    }

    async fn country_code(&self, _ctx: CallContext) -> RpcResult<String> {
        StorageManager::get_string(&self.platform_state, StorageProperty::CountryCode).await
    }

    async fn country_code_set(
        &self,
        _ctx: CallContext,
        set_request: SetStringProperty,
    ) -> RpcResult<()> {
        StorageManager::set_string(
            &self.platform_state,
            StorageProperty::CountryCode,
            set_request.value,
            None,
        )
        .await
    }

    async fn on_country_code_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        self.on_request_app_event(
            ctx,
            request,
            "LocalizationCountryCodeChanged",
            "localization.onCountryCodeChanged",
        )
        .await
    }

    async fn language(&self, _ctx: CallContext) -> RpcResult<String> {
        StorageManager::get_string(&self.platform_state, StorageProperty::Language).await
    }

    async fn language_set(
        &self,
        _ctx: CallContext,
        set_request: LanguageProperty,
    ) -> RpcResult<()> {
        StorageManager::set_string(
            &self.platform_state,
            StorageProperty::Language,
            set_request.value,
            None,
        )
        .await
    }

    async fn on_language_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        self.on_request_app_event(
            ctx,
            request,
            "LocalizationLanguageChanged",
            "localization.onLanguageChanged",
        )
        .await
    }

    async fn postal_code(&self, ctx: CallContext) -> RpcResult<String> {
        match LocalizationImpl::postal_code(&self.platform_state, ctx.app_id).await {
            Some(postal_code) => Ok(postal_code),
            None => Err(StorageManager::get_firebolt_error(
                &StorageProperty::PostalCode,
            )),
        }
    }

    async fn postal_code_set(
        &self,
        _ctx: CallContext,
        set_request: SetStringProperty,
    ) -> RpcResult<()> {
        StorageManager::set_string(
            &self.platform_state,
            StorageProperty::PostalCode,
            set_request.value,
            None,
        )
        .await
    }

    async fn on_postal_code_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        self.on_request_app_event(
            ctx,
            request,
            "LocalizationPostalCodeChanged",
            "localization.onPostalCodeChanged",
        )
        .await
    }

    async fn locale(&self, _ctx: CallContext) -> RpcResult<String> {
        StorageManager::get_string(&self.platform_state, StorageProperty::Locale).await
    }

    async fn locale_set(&self, _ctx: CallContext, set_request: SetStringProperty) -> RpcResult<()> {
        StorageManager::set_string(
            &self.platform_state,
            StorageProperty::Locale,
            set_request.value,
            None,
        )
        .await
    }

    async fn on_locale_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        self.on_request_app_event(
            ctx,
            request,
            "LocalizationLocaleChanged",
            "localization.onLocaleChanged",
        )
        .await
    }

    async fn latlon(&self, _ctx: CallContext) -> RpcResult<String> {
        StorageManager::get_string(&self.platform_state, StorageProperty::LatLon).await
    }

    async fn latlon_set(&self, _ctx: CallContext, set_request: SetStringProperty) -> RpcResult<()> {
        StorageManager::set_string(
            &self.platform_state,
            StorageProperty::LatLon,
            set_request.value,
            None,
        )
        .await
    }

    async fn on_latlon_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        self.on_request_app_event(
            ctx,
            request,
            "LocalizationLatlonChanged",
            "localization.onLatlonChanged",
        )
        .await
    }

    async fn additional_info(&self, _ctx: CallContext) -> RpcResult<HashMap<String, String>> {
        let json_str =
            StorageManager::get_string(&self.platform_state, StorageProperty::AdditionalInfo).await;
        match json_str {
            Ok(s) => {
                let deserialized = serde_json::from_str::<HashMap<String, String>>(&s).unwrap();
                return Ok(deserialized);
            }
            Err(_e) => return Ok(HashMap::new()),
        }
    }

    async fn additional_info_set(
        &self,
        _ctx: CallContext,
        set_request: SetStringProperty,
    ) -> RpcResult<()> {
        StorageManager::set_string(
            &self.platform_state,
            StorageProperty::AdditionalInfo,
            set_request.value,
            None,
        )
        .await
    }

    async fn on_additional_info_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        self.on_request_app_event(
            ctx,
            request,
            "LocalizationAdditionalInfoChanged",
            "localization.onAdditionalInfoChanged",
        )
        .await
    }

    async fn timezone_set(&self, ctx: CallContext, set_request: TimezoneProperty) -> RpcResult<()> {
        let resp = self
            .platform_state
            .get_client()
            .send_extn_request(DeviceInfoRequest::GetAvailableTimezones)
            .await;

        if let Err(_e) = resp {
            return Err(jsonrpsee::core::Error::Custom(String::from(
                "timezone_set: error response TBD",
            )));
        }
        if let Some(ExtnResponse::AvailableTimezones(timezones)) = resp.unwrap().payload.extract() {
            if !timezones.contains(&set_request.value) {
                error!(
                    "timezone_set: Unsupported timezone: tz={}",
                    set_request.value
                );
                return Err(jsonrpsee::core::Error::Custom(String::from(
                    "timezone_set: error response TBD",
                )));
            }
        } else {
            return Err(jsonrpsee::core::Error::Custom(String::from(
                "timezone_set: error response TBD",
            )));
        }

        if let Ok(_response) = self
            .platform_state
            .get_client()
            .send_extn_request(DeviceInfoRequest::SetTimezone(set_request.value.clone()))
            .await
        {
            rpc_add_event_listener(
                &self.platform_state,
                ctx,
                ListenRequest { listen: true },
                EVENT_TIMEZONE_CHANGED,
            )
            .await
            .ok();
            return Ok(());
        }
        Err(rpc_err("timezone: error response TBD"))
    }

    async fn timezone(&self, _ctx: CallContext) -> RpcResult<String> {
        if let Ok(response) = self
            .platform_state
            .get_client()
            .send_extn_request(DeviceInfoRequest::GetTimezone)
            .await
        {
            if let Some(ExtnResponse::String(v)) = response.payload.clone().extract() {
                return Ok(v);
            }
        }
        Err(rpc_err("timezone: error response TBD"))
    }

    async fn on_timezone_changed(
        &self,
        ctx: CallContext,
        request: ListenRequest,
    ) -> RpcResult<ListenerResponse> {
        rpc_add_event_listener(&self.platform_state, ctx, request, EVENT_TIMEZONE_CHANGED).await
    }
}

pub struct LocalizationRPCProvider;

impl RippleRPCProvider<LocalizationImpl> for LocalizationRPCProvider {
    fn provide(platform_state: PlatformState) -> RpcModule<LocalizationImpl> {
        (LocalizationImpl { platform_state }).into_rpc()
    }
}
