fn main() {
    let commands = hope_app::ipc_contract();
    let desktop_commands = hope_app::desktop_invoke_contract();
    println!(
        "Hope app shell skeleton is ready. IPC contract: {:?}. Desktop invoke phase-1 contract: {:?}",
        commands, desktop_commands
    );
}
