use std::fs;
use std::process::Command;
use std::thread;
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceStats {
    pub rss_bytes: u64,
    pub app_cuda_vram_bytes: u64,
    pub gpu_total_vram_bytes: u64,
}

pub fn spawn_telemetry(ctx: egui::Context) -> Receiver<ResourceStats> {
    let (tx, rx) = unbounded::<ResourceStats>();
    let _ = thread::Builder::new()
        .name("resource-telemetry".to_owned())
        .spawn(move || sample_loop(tx, ctx));
    rx
}

fn sample_loop(tx: Sender<ResourceStats>, ctx: egui::Context) {
    let pid = std::process::id();
    let mut tick: u32 = 0;
    let mut cuda_vram: u64 = 0;
    let gpu_total_vram_bytes = read_gpu_total_vram_bytes();

    thread::sleep(Duration::from_millis(500));
    loop {
        if tick.is_multiple_of(3) {
            cuda_vram = read_process_cuda_vram_bytes(pid);
        }
        tick = tick.wrapping_add(1);

        let stats = ResourceStats {
            rss_bytes: read_proc_rss_bytes(),
            app_cuda_vram_bytes: cuda_vram,
            gpu_total_vram_bytes,
        };
        if tx.send(stats).is_err() {
            break;
        }
        ctx.request_repaint();
        thread::sleep(Duration::from_secs(1));
    }
}

fn read_proc_rss_bytes() -> u64 {
    let Ok(text) = fs::read_to_string("/proc/self/status") else {
        return 0;
    };
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("VmRSS:") {
            let kb: u64 = rest
                .split_whitespace()
                .next()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0);
            return kb.saturating_mul(1024);
        }
    }
    0
}

fn read_process_cuda_vram_bytes(my_pid: u32) -> u64 {
    let Ok(out) = Command::new("nvidia-smi")
        .args([
            "--query-compute-apps=pid,used_memory",
            "--format=csv,noheader,nounits",
        ])
        .output()
    else {
        return 0;
    };
    if !out.status.success() {
        return 0;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut total_mb: u64 = 0;
    for line in text.lines() {
        let mut parts = line.split(',');
        let pid = parts.next().and_then(|s| s.trim().parse::<u32>().ok());
        let mb = parts.next().and_then(|s| s.trim().parse::<u64>().ok());
        if pid == Some(my_pid) {
            total_mb = total_mb.saturating_add(mb.unwrap_or(0));
        }
    }
    total_mb.saturating_mul(1024 * 1024)
}

fn read_gpu_total_vram_bytes() -> u64 {
    let Ok(out) = Command::new("nvidia-smi")
        .args(["--query-gpu=memory.total", "--format=csv,noheader,nounits"])
        .output()
    else {
        return 0;
    };
    if !out.status.success() {
        return 0;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mb: u64 = text
        .lines()
        .next()
        .and_then(|l| l.trim().parse().ok())
        .unwrap_or(0);
    mb.saturating_mul(1024 * 1024)
}
