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

use std::{
    collections::HashMap,
    str::FromStr,
    sync::{Arc, RwLock},
};

use ripple_sdk::{
    api::{
        apps::{AppEvent, AppEventRequest},
        device::{
            device_events::{DeviceEvent, DeviceEventCallback},
            device_operator::DeviceSubscribeRequest,
            device_request::{
                AudioProfile, NetworkResponse, NetworkState, NetworkType, PowerState,
                SystemPowerState,
            },
        },
    },
    extn::extn_client_message::ExtnEvent,
    log::{error, trace},
    serde_json::{self, Value},
    utils::error::RippleError,
};
use serde::{Deserialize, Serialize};

use crate::{thunder_state::ThunderState, utils::get_audio_profile_from_value};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveInputThunderEvent {
    pub active_input: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ResolutionChangedEvent {
    pub width: i32,
    pub height: i32,
    pub video_display_type: String,
    pub resolution: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceGuidanceEvent {
    pub state: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub enum ThunderEventMessage {
    ActiveInput(ActiveInputThunderEvent),
    Resolution(ResolutionChangedEvent),
    Network(NetworkResponse),
    PowerState(SystemPowerState),
    VoiceGuidance(VoiceGuidanceEvent),
    Audio(HashMap<AudioProfile, bool>),
}
impl ThunderEventMessage {
    pub fn get(event: &str, value: &Value) -> Option<Self> {
        if let Ok(device_event) = DeviceEvent::from_str(event) {
            match device_event {
                DeviceEvent::InputChanged | DeviceEvent::HdrChanged => {
                    if let Ok(v) = serde_json::from_value(value.clone()) {
                        return Some(ThunderEventMessage::ActiveInput(v));
                    }
                }
                DeviceEvent::ScreenResolutionChanged | DeviceEvent::VideoResolutionChanged => {
                    if let Ok(v) = serde_json::from_value(value.clone()) {
                        return Some(ThunderEventMessage::Resolution(v));
                    }
                }
                DeviceEvent::NetworkChanged => {
                    if let Some(v) = value["interface"].as_str() {
                        if let Ok(network_type) = NetworkType::from_str(v) {
                            if let Some(v) = value["status"].as_str() {
                                if let Ok(network_status) = NetworkState::from_str(v) {
                                    return Some(ThunderEventMessage::Network(NetworkResponse {
                                        _type: network_type,
                                        state: network_status,
                                    }));
                                }
                            }
                        }
                    }
                }
                DeviceEvent::SystemPowerStateChanged => {
                    if let Some(v) = value["powerState"].as_str() {
                        if let Ok(power_state) = PowerState::from_str(v) {
                            if let Some(v) = value["currentPowerState"].as_str() {
                                if let Ok(current_power_state) = PowerState::from_str(v) {
                                    return Some(ThunderEventMessage::PowerState(
                                        SystemPowerState {
                                            power_state,
                                            current_power_state,
                                        },
                                    ));
                                }
                            }
                        }
                    }
                }
                DeviceEvent::VoiceGuidanceChanged => {
                    if let Ok(v) = serde_json::from_value(value.clone()) {
                        return Some(ThunderEventMessage::VoiceGuidance(v));
                    }
                }
                DeviceEvent::AudioChanged => {
                    return Some(ThunderEventMessage::Audio(get_audio_profile_from_value(
                        value.clone(),
                    )))
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct ThunderEventHandler {
    pub request: DeviceSubscribeRequest,
    pub handle:
        fn(state: ThunderState, value: ThunderEventMessage, callback_type: DeviceEventCallback),
    pub is_valid: fn(event: ThunderEventMessage) -> bool,
    pub listeners: Vec<String>,
    pub id: String,
    pub callback_type: DeviceEventCallback,
}

pub trait ThunderEventHandlerProvider {
    type EVENT: Serialize;

    fn provide(id: String, callback_type: DeviceEventCallback) -> ThunderEventHandler;
    fn module() -> String;
    fn event_name() -> String;
    fn get_mapped_event() -> String;
    fn get_id(&self) -> String {
        Self::get_mapped_event()
    }
    fn get_device_request() -> DeviceSubscribeRequest {
        DeviceSubscribeRequest {
            module: Self::module(),
            event_name: Self::event_name(),
            params: None,
            sub_id: Some(Self::get_mapped_event()),
        }
    }

    fn get_extn_event(
        r: Self::EVENT,
        callback_type: DeviceEventCallback,
    ) -> Result<ExtnEvent, RippleError> {
        let result = serde_json::to_value(r).unwrap();
        match callback_type {
            DeviceEventCallback::FireboltAppEvent => {
                Ok(ExtnEvent::AppEvent(AppEventRequest::Emit(AppEvent {
                    event_name: Self::get_mapped_event(),
                    context: None,
                    result,
                    app_id: None,
                })))
            }
            DeviceEventCallback::ExtnEvent => Ok(ExtnEvent::Value(result)),
        }
    }
}

impl ThunderEventHandler {
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    pub fn add_listener(&mut self, id: String) {
        self.listeners.push(id);
    }

    pub fn remove_listener(&mut self, id: String) -> bool {
        self.listeners.retain(|x| !x.eq(&id));
        self.listeners.len() == 0
    }

    pub fn process(
        &self,
        state: ThunderState,
        event_id: &str,
        value: Value,
        callback_type: DeviceEventCallback,
    ) {
        if let Some(event_message) = ThunderEventMessage::get(event_id, &value) {
            if (self.is_valid)(event_message.clone()) {
                (self.handle)(state, event_message, callback_type)
            }
        }
    }

    pub fn callback_device_event(state: ThunderState, event_name: String, event: ExtnEvent) {
        if state.event_processor.check_last_event(&event_name, &event) {
            state.event_processor.add_last_event(&event_name, &event);
            if let Err(_) = match event {
                ExtnEvent::AppEvent(a) => state.get_client().request_transient(a),
                ExtnEvent::PowerState(p) => state.get_client().request_transient(p),
                _ => Err(RippleError::InvalidOutput),
            } {
                error!("Error while forwarding app event");
            }
        } else {
            trace!("Already sent")
        }
    }
}

pub trait DeviceSubscribeRequestProvider {
    fn get_subscribe_request(&self) -> DeviceSubscribeRequest;
    fn get_handler(&self) -> fn(state: ThunderState, value: Value);
}

#[derive(Debug, Clone)]
pub struct ThunderEventProcessor {
    event_map: Arc<RwLock<HashMap<String, ThunderEventHandler>>>,
    last_event: Arc<RwLock<HashMap<String, Value>>>,
}

impl ThunderEventProcessor {
    pub fn new() -> ThunderEventProcessor {
        ThunderEventProcessor {
            event_map: Arc::new(RwLock::new(HashMap::new())),
            last_event: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn get_handler(&self, event: &str) -> Option<ThunderEventHandler> {
        let event_map = self.event_map.read().unwrap();
        event_map.get(event).cloned()
    }

    pub fn handle_listener(
        &self,
        listen: bool,
        app_id: String,
        handler: ThunderEventHandler,
    ) -> bool {
        if listen {
            self.add_event_listener(app_id, handler)
        } else {
            self.remove_event_listener(handler.get_id(), app_id)
        }
    }

    pub fn add_event_listener(&self, app_id: String, handler: ThunderEventHandler) -> bool {
        let event_name = handler.get_id();
        let mut event_map = self.event_map.write().unwrap();
        if let Some(entry) = event_map.get_mut(&event_name) {
            entry.add_listener(app_id);
            return false;
        } else {
            event_map.insert(event_name, handler);
        }
        true
    }

    pub fn remove_event_listener(&self, event_name: String, app_id: String) -> bool {
        let mut event_map = self.event_map.write().unwrap();
        if let Some(entry) = event_map.get_mut(&event_name) {
            if !entry.remove_listener(app_id) {
                return false;
            }
        }
        event_map.remove(&event_name);
        true
    }

    pub fn add_last_event(&self, event_name: &str, value: &ExtnEvent) {
        let mut last_event_map = self.last_event.write().unwrap();
        last_event_map.insert(
            event_name.to_string(),
            serde_json::to_value(value.clone()).unwrap(),
        );
    }

    pub fn check_last_event(&self, event_name: &str, value: &ExtnEvent) -> bool {
        let ref_value = serde_json::to_value(value.clone()).unwrap();
        let last_event_map = self.last_event.read().unwrap();
        if let Some(last_event) = last_event_map.get(event_name) {
            return last_event.eq(&ref_value);
        }
        false
    }
}
