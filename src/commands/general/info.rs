use sysinfo::{Disks, Pid, System};

use crate::commands::cmd::{COMMANDS};
use crate::cmd;
use std::collections::HashSet;

fn get_memory_usage(sys: &System) -> String {
    let pid = Pid::from(std::process::id() as usize);
    if let Some(process) = sys.process(pid) {
        let rss = process.memory()/1024/1024;
        format!("{rss}MB").to_string()
    } else {
        "Unknown".to_string()
    }
}

fn get_total_mem(sys: &System) -> String {
    let total_mem = sys.total_memory()/1024/1024;
    let used_mem = sys.used_memory()/1024/1024;
    format!("{used_mem}MB / {total_mem}MB").to_string()
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

fn get_disk_info() -> String {
    let disks = Disks::new_with_refreshed_list();
    let mut total = 0;
    let mut used = 0; 
    for disk in disks.list() {
        total = disk.total_space()/1024/1024/1024;
        used = total - disk.available_space()/1024/1024/1024;
    }
    format!("{}GB / {}GB", used, total).to_string()
}

cmd!(
    Info,
    name: "info",
    aliases: ["i", "inf"],
    category: "general",
    execute: |ctx| {
        let app_name = env!("CARGO_PKG_NAME");
        let lib_version = get_lib_version();
        let compiler_version = env!("RUSTC_VERSION");
        let os_name = get_os_name();
        let allocator = "mimalloc";

        let mut categories = HashSet::new();
        for cmd in COMMANDS.iter() {
            categories.insert(cmd.category());
        }
        let total_cmds = COMMANDS.len();
        let total_cats = categories.len();

        let mode = &ctx.state.config.mode;
        let prefix = &ctx.state.config.prefixes;


        let mut sys = System::new_all();
        sys.refresh_all();
        let mem_usage = get_memory_usage(&sys);
        let cpu = sys.cpus()[0].brand();
        let physical_cores = System::physical_core_count().unwrap_or(0);
        let logical_cores = sys.cpus().len();
        let total_mem = get_total_mem(&sys);
        let free_mem = sys.total_memory()/1024/1024-sys.used_memory()/1024/1024;
        let uptime_secs = ctx.state.start_time.elapsed().as_secs();
        let hours = uptime_secs / 3600;
        let minutes = (uptime_secs % 3600) / 60;
        let seconds = uptime_secs % 60;
        let uptime = format!("{:02}h {:02}m {:02}s", hours, minutes, seconds);
        let disk = get_disk_info();
        let response = format!(
"```INFORMATION
-----------
App: {}
Library: whatsapp-rust v{}
Compiler: {}
OS: {}
Memory Usage: {}
Allocator: {}

STATISTICS
----------
Total Commands: {}
Total Categories: {}
Current Mode: {}
Active Prefix: {:?}
Uptime: {}

RESOURCES
---------
CPU: {}
Physical Cores: {}
Logical Cores: {}
Total Memory: {}
Free Memory: {}MB
Disk: {}
```",
            app_name,
            lib_version,
            compiler_version,
            os_name,
            mem_usage,
            allocator,
            total_cmds,
            total_cats,
            mode,
            prefix,
            uptime,
            cpu,
            physical_cores,
            logical_cores,
            total_mem,
            free_mem,
            disk
        );

        ctx.reply(&response).await?;
    }
);