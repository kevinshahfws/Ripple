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
#[serde(rename_all = "lowercase")]
pub struct BrowserProps {
    pub user_agent: Option<String>,
    pub http_cookie_accept_policy: Option<String>,
    pub local_storage_enabled: Option<bool>,
    pub languages: Option<String>,
    pub headers: Option<String>,
}

impl BrowserProps {
    pub fn is_local_storage_enabled(&self) -> bool {
        if self.local_storage_enabled.is_some() {
            return self.local_storage_enabled.unwrap();
        }
        false
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserLaunchParams {
    pub uri: String,
    pub browser_name: String,
    #[serde(rename = "type")]
    pub _type: String,
    pub visible: bool,
    pub suspend: bool,
    pub focused: bool,
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
    pub properties: Option<BrowserProps>,
}

impl BrowserLaunchParams {
    pub fn is_local_storage_enabled(&self) -> bool {
        if self.properties.is_some() {
            return self.properties.clone().unwrap().is_local_storage_enabled();
        }
        false
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserDestroyParams {
    pub browser_name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserNameRequestParams {
    pub runtime: String,
    pub name: String,
    pub instances: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum BrowserRequest {
    Start(BrowserLaunchParams),
    Destroy(BrowserDestroyParams),
    GetBrowserName(BrowserNameRequestParams),
}

impl ExtnPayloadProvider for BrowserRequest {
    fn get_extn_payload(&self) -> ExtnPayload {
        ExtnPayload::Request(ExtnRequest::Device(DeviceRequest::Browser(self.clone())))
    }

    fn get_from_payload(payload: ExtnPayload) -> Option<Self> {
        match payload {
            ExtnPayload::Request(request) => match request {
                ExtnRequest::Device(r) => match r {
                    DeviceRequest::Browser(d) => return Some(d),
                    _ => {}
                },
                _ => {}
            },
            _ => {}
        }
        None
    }

    fn contract() -> RippleContract {
        RippleContract::Browser
    }
}
