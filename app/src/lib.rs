pub mod ipc;
pub mod runtime;
pub mod state;

pub fn ipc_contract() -> &'static [&'static str] {
    ipc::IPC_COMMANDS
}
