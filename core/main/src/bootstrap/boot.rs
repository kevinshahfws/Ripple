use ripple_sdk::framework::bootstrap::Bootstrap;

use crate::state::bootstrap_state::BootstrapState;

use super::{
    setup_extn_client_step::SetupExtnClientStep, start_fbgateway_step::FireboltGatewayStep,
    start_ws_step::StartWsStep,
};
/// Starts up Ripple uses `PlatformState` to manage State
/// # Arguments
/// * `platform_state` - PlatformState
///
/// # Panics
///
/// Bootstrap panics are fatal in nature and it could happen due to bad configuration or device state. Logs should provide more information on which step the failure had occurred.
///
/// # Steps
///
/// 1. [SetupExtnClientStep] - Initializes the extn client to start the Inter process communication backbone
/// 2. [StartWsStep] - Starts the Websocket to accept external and internal connections
/// 3. [FireboltGatewayStep] - Starts the firebolt gateway and blocks the thread to keep it alive till interruption.
///
pub async fn boot(state: BootstrapState) {
    let bootstrap = &Bootstrap::new(state);
    bootstrap
        .step(SetupExtnClientStep)
        .await
        .expect("Extn Client setup failure")
        .step(StartWsStep)
        .await
        .expect("Websocket startup failure")
        .step(FireboltGatewayStep)
        .await
        .expect("Firebolt Gateway failure");
    // -- capability Manager
    // -- App event manager
    // -- User grant manager
    // -- permissions manager
    // -- app manager
    // Start Launcher
    // Start Dpab
    // Start Launcher
}
