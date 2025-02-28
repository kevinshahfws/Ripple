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

use crate::{
    extn::extn_client_message::{ExtnPayload, ExtnPayloadProvider, ExtnRequest},
    framework::ripple_contract::RippleContract,
};

use super::device_request::DeviceRequest;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum WifiSecurityMode {
    None,
    Wep64,
    Wep128,
    WpaPskTkip,
    WpaPskAes,
    Wpa2PskTkip,
    Wpa2PskAes,
    WpaEnterpriseTkip,
    WpaEnterpriseAes,
    Wpa2EnterpriseTkip,
    Wpa2EnterpriseAes,
    Wpa2Psk,
    Wpa2Enterprise,
    Wpa3PskAes,
    Wpa3Sae,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessPointRequest {
    pub ssid: String,
    pub passphrase: String,
    pub security: WifiSecurityMode,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum WifiRequest {
    Scan(u64),
    Connect(AccessPointRequest),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessPoint {
    pub ssid: String,
    pub security_mode: WifiSecurityMode,
    pub signal_strength: i32,
    pub frequency: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccessPointList {
    pub list: Vec<AccessPoint>,
}

impl ExtnPayloadProvider for WifiRequest {
    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Request(ExtnRequest::Device(DeviceRequest::Wifi(self.clone())))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        match payload {
            ExtnPayload::Request(request) => match request {
                ExtnRequest::Device(r) => match r {
                    DeviceRequest::Wifi(d) => return Some(d),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        None
    }

    fn contract() -> RippleContract {
        RippleContract::Wifi
    }
}
