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

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

use crate::{
    api::session::AccountSession,
    extn::extn_client_message::{ExtnPayload, ExtnPayloadProvider, ExtnRequest, ExtnResponse},
    framework::ripple_contract::RippleContract,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdvertisingRequest {
    GetAdInitObject(AdInitObjectRequestParams),
    GetAdIdObject(AdIdRequestParams),
    ResetAdIdentifier(AccountSession),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdInitObjectRequestParams {
    pub privacy_data: HashMap<String, String>,
    pub environment: String,
    pub durable_app_id: String,
    pub app_version: String,
    pub distributor_app_id: String,
    pub device_ad_attributes: HashMap<String, String>,
    pub coppa: bool,
    pub authentication_entity: String,
    pub dist_session: AccountSession,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdInitObjectResponse {
    pub ad_server_url: String,
    pub ad_server_url_template: String,
    pub ad_network_id: String,
    pub ad_profile_id: String,
    pub ad_site_section_id: String,
    pub ad_opt_out: bool,
    pub privacy_data: String,
    pub ifa_value: String,
    pub ifa: String,
    pub app_name: String,
    pub app_bundle_id: String,
    pub app_version: String,
    pub distributor_app_id: String,
    pub device_ad_attributes: String,
    pub coppa: String,
    pub authentication_entity: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdIdRequestParams {
    pub privacy_data: HashMap<String, String>,
    pub app_id: String,
    pub dist_session: AccountSession,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdIdResponse {
    pub ifa: String,
    pub ifa_type: String,
    pub lmt: String,
}

impl ExtnPayloadProvider for AdvertisingRequest {
    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Request(ExtnRequest::Advertising(self.clone()))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<AdvertisingRequest> {
        match payload {
            ExtnPayload::Request(request) => match request {
                ExtnRequest::Advertising(r) => return Some(r),
                _ => {}
            },
            _ => {}
        }
        None
    }

    fn contract() -> RippleContract {
        RippleContract::Advertising
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AdvertisingResponse {
    None,
    AdInitObject(AdInitObjectResponse),
    AdIdObject(AdIdResponse),
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdvertisingFrameworkConfig {
    pub ad_server_url: String,
    pub ad_server_url_template: String,
    pub ad_network_id: String,
    pub ad_profile_id: String,
    pub ad_site_section_id: String,
    pub ad_opt_out: bool,
    pub privacy_data: String,
    pub ifa_value: String,
    pub ifa: String,
    pub app_name: String,
    pub app_bundle_id: String,
    pub distributor_app_id: String,
    pub device_ad_attributes: String,
    pub coppa: u32,
    pub authentication_entity: String,
}

impl ExtnPayloadProvider for AdvertisingResponse {
    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Response(ExtnResponse::Advertising(self.clone()))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        match payload {
            ExtnPayload::Response(r) => match r {
                ExtnResponse::Advertising(v) => return Some(v),
                _ => {}
            },
            _ => {}
        }

        None
    }

    fn contract() -> RippleContract {
        RippleContract::Advertising
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetAdConfig {
    pub options: AdConfig,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    Prod,
    Test,
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Environment::Prod => write!(f, "prod"),
            Environment::Test => write!(f, "test"),
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment::Prod
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AdConfig {
    #[serde(default)]
    pub environment: Environment,
    // COPPA stands for Children's Online Privacy Protection Act.
    pub coppa: Option<bool>,
    pub authentication_entity: Option<String>,
}

impl Default for GetAdConfig {
    fn default() -> Self {
        GetAdConfig {
            options: AdConfig {
                environment: Environment::default(),
                coppa: Some(false),
                authentication_entity: Some("".to_owned()),
            },
        }
    }
}
