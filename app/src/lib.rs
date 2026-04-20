pub mod desktop_bridge;
pub mod ipc;
pub mod runtime;
pub mod state;

pub fn ipc_contract() -> &'static [&'static str] {
    ipc::IPC_COMMANDS
}

pub fn desktop_invoke_contract() -> &'static [&'static str] {
    desktop_bridge::desktop_invoke_contract()
}
