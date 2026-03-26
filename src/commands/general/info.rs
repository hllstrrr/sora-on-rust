use crate::commands::cmd::{COMMANDS};
use crate::cmd;
use std::collections::HashSet;

fn get_memory_usage() -> String {
    if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
        for line in status.lines() {
            if line.starts_with("RssAnon:") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    if let Ok(kb) = parts[1].parse::<f64>() {
                        return format!("{:.2} MB", kb / 1024.0);
                    }
                }
            }
        }
    }
    "Unknown".to_string()
}
fn get_os_name() -> String {
    if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
        for line in os_release.lines() {
            if line.starts_with("PRETTY_NAME=") {
                return line.replace("PRETTY_NAME=", "").replace("\"", "");
            }
        }
    }
    std::env::consts::OS.to_string()
}

fn get_lib_version() -> String {
    let cargo_toml = include_str!("../../../Cargo.toml");
    for line in cargo_toml.lines() {
        if line.starts_with("whatsapp-rust") {
            let parts: Vec<&str> = line.split('"').collect();
            if parts.len() >= 3 {
                return parts[1].to_string();
            }
        }
    }
    "Unknown".to_string()
}

cmd!(
    InfoCommand,
    name: "info",
    aliases: ["i", "inf"],
    category: "general",
    execute: |ctx| {
        let app_name = env!("CARGO_PKG_NAME");
        let lib_version = get_lib_version();
        let compiler_version = env!("RUSTC_VERSION");
        let os_name = get_os_name();

        let mut categories = HashSet::new();
        for cmd in COMMANDS.iter() {
            categories.insert(cmd.category());
        }
        let total_cmds = COMMANDS.len();
        let total_cats = categories.len();

        let mode = &ctx.state.config.mode;
        let prefix = &ctx.state.config.prefixes;

        let mem_usage = get_memory_usage();
        
        let uptime_secs = ctx.state.start_time.elapsed().as_secs();
        let hours = uptime_secs / 3600;
        let minutes = (uptime_secs % 3600) / 60;
        let seconds = uptime_secs % 60;
        let uptime = format!("{:02}h {:02}m {:02}s", hours, minutes, seconds);

        let response = format!(
"```INFORMATION
------------------------
App: {}
Library: whatsapp-rust v{}
Compiler: {}
OS: {}

STATISTICS
----------
Total Commands: {}
Total Categories: {}
Current Mode: {}
Active Prefix: {:?}

RESOURCES
---------
Memory Usage: {}
Uptime: {}```",
            app_name,
            lib_version,
            compiler_version,
            os_name,
            total_cmds,
            total_cats,
            mode,
            prefix,
            mem_usage,
            uptime
        );

        ctx.reply(&response).await?;
    }
);