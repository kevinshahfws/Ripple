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

use futures::{future::join_all, StreamExt};
use jsonrpsee::{
    core::{
        server::{
            helpers::MethodSink,
            resource_limiting::Resources,
            rpc_module::{MethodKind, Methods},
        },
        TEN_MB_SIZE_BYTES,
    },
    types::{error::ErrorCode, Id, Params},
};
use ripple_sdk::{
    api::{
        apps::EffectiveTransport,
        gateway::rpc_gateway_api::{ApiMessage, JsonRpcApiResponse, RpcRequest},
    },
    extn::extn_client_message::{ExtnMessage, ExtnResponse},
    log::{error, info},
    serde_json::{self, Result as SResult},
    tokio::{self},
    utils::error::RippleError,
};
use std::sync::{Arc, RwLock};

use crate::state::{platform_state::PlatformState, session_state::Session};

pub struct RpcRouter;

#[derive(Debug, Clone)]
pub struct RouterState {
    methods: Arc<RwLock<Methods>>,
    resources: Resources,
}

impl RouterState {
    pub fn new() -> RouterState {
        RouterState {
            methods: Arc::new(RwLock::new(Methods::new())),
            resources: Resources::default(),
        }
    }

    pub fn update_methods(&self, methods: Methods) {
        let mut methods_state = self.methods.write().unwrap();
        let _ = methods_state.merge(methods.initialize_resources(&self.resources).unwrap());
    }

    fn get_methods(&self) -> Methods {
        self.methods.read().unwrap().clone()
    }
}

async fn resolve_route(
    methods: Methods,
    resources: Resources,
    req: RpcRequest,
) -> Result<ApiMessage, RippleError> {
    info!("Routing {}", req.method);
    let id = Id::Number(req.ctx.call_id);
    let (sink_tx, mut sink_rx) = futures_channel::mpsc::unbounded::<String>();
    let sink = MethodSink::new_with_limit(sink_tx, TEN_MB_SIZE_BYTES);
    let mut method_executors = Vec::new();
    let params = Params::new(Some(req.params_json.as_str()));
    match methods.method_with_name(&req.method) {
        None => {
            sink.send_error(id, ErrorCode::MethodNotFound.into());
        }
        Some((name, method)) => match &method.inner() {
            MethodKind::Sync(callback) => match method.claim(name, &resources) {
                Ok(_guard) => {
                    (callback)(id, params, &sink);
                }
                Err(_) => {
                    sink.send_error(id, ErrorCode::MethodNotFound.into());
                }
            },
            MethodKind::Async(callback) => match method.claim(name, &resources) {
                Ok(guard) => {
                    let sink = sink.clone();
                    let id = id.into_owned();
                    let params = params.into_owned();

                    let fut = async move {
                        (callback)(id, params, sink, 1, Some(guard)).await;
                    };
                    method_executors.push(fut);
                }
                Err(e) => {
                    error!("{:?}", e);
                    sink.send_error(id, ErrorCode::MethodNotFound.into());
                }
            },
            _ => {
                error!("Unsupported method call");
            }
        },
    }

    join_all(method_executors).await;
    if let Some(r) = sink_rx.next().await {
        return Ok(ApiMessage::new(req.ctx.protocol, r, req.ctx.request_id));
    }
    Err(RippleError::InvalidOutput)
}

impl RpcRouter {
    pub async fn route(state: PlatformState, req: RpcRequest, session: Session) {
        let methods = state.router_state.get_methods();
        let resources = state.router_state.resources.clone();
        tokio::spawn(async move {
            let session_id = req.ctx.session_id.clone();
            if let Ok(msg) = resolve_route(methods, resources, req).await {
                info!(
                    "Sending Firebolt response to {} {}",
                    session_id, msg.jsonrpc_msg
                );
                match session.get_transport() {
                    EffectiveTransport::Websocket => {
                        if let Err(e) = session.send_json_rpc(msg).await {
                            error!("Error while responding back message {:?}", e)
                        }
                    }
                    EffectiveTransport::Bridge(container_id) => {
                        if let Err(e) = state.send_to_bridge(container_id, msg).await {
                            error!("Error sending event to bridge {:?}", e);
                        }
                    }
                }
            }
        });
    }

    pub async fn route_extn_protocol(
        state: &PlatformState,
        req: RpcRequest,
        extn_msg: ExtnMessage,
    ) {
        if extn_msg.callback.is_none() {
            // The caller of this function already checks this adding it here none the less.
            error!("No valid callbacks")
        }

        let callback = extn_msg.clone().callback.unwrap();
        let methods = state.router_state.get_methods();
        let resources = state.router_state.resources.clone();
        tokio::spawn(async move {
            if let Ok(msg) = resolve_route(methods, resources, req).await {
                let r: SResult<JsonRpcApiResponse> = serde_json::from_str(&msg.jsonrpc_msg);

                if let Ok(resp) = r {
                    let response_value = if resp.result.is_some() {
                        resp.result.unwrap()
                    } else {
                        if resp.error.is_some() {
                            resp.error.unwrap()
                        } else {
                            serde_json::to_value(RippleError::InvalidOutput).unwrap()
                        }
                    };
                    let return_value = ExtnResponse::Value(response_value);
                    if let Ok(response) = extn_msg.get_response(return_value) {
                        if let Err(e) = callback.send(response.into()) {
                            error!("Error while sending back rpc request for extn {:?}", e);
                        }
                    } else {
                        error!("Not a Request object {:?}", extn_msg);
                    }
                }
            }
        });
    }
}
