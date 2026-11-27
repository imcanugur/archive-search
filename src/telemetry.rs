use std::fs;
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};

use crossbeam_channel::{unbounded, Receiver, Sender};

#[derive(Debug, Clone, Copy, Default)]
pub struct ResourceStats {
    pub rss_bytes: u64,
    pub cpu_percent: f32,
    pub app_cuda_vram_bytes: u64,
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
    let mut prev_cpu = read_proc_cpu_ticks();
    let mut prev_time = Instant::now();
    let mut tick: u32 = 0;
    let mut cuda_vram: u64 = 0;

    thread::sleep(Duration::from_millis(500));
    loop {
        let rss_bytes = read_proc_rss_bytes();
        let now = Instant::now();
        let cur_cpu = read_proc_cpu_ticks();
        let dt = now.duration_since(prev_time).as_secs_f64().max(0.001);
        let cpu_percent = compute_cpu_percent(prev_cpu, cur_cpu, dt);
        prev_cpu = cur_cpu;
        prev_time = now;

        if tick.is_multiple_of(3) {
            cuda_vram = read_process_cuda_vram_bytes(pid);
        }
        tick = tick.wrapping_add(1);

        let stats = ResourceStats {
            rss_bytes,
            cpu_percent,
            app_cuda_vram_bytes: cuda_vram,
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

fn read_proc_cpu_ticks() -> u64 {
    let Ok(text) = fs::read_to_string("/proc/self/stat") else {
        return 0;
    };
    let Some((_, after_comm)) = text.rsplit_once(')') else {
        return 0;
    };
    let fields: Vec<&str> = after_comm.split_whitespace().collect();
    let utime: u64 = fields.get(11).and_then(|s| s.parse().ok()).unwrap_or(0);
    let stime: u64 = fields.get(12).and_then(|s| s.parse().ok()).unwrap_or(0);
    utime.saturating_add(stime)
}

fn compute_cpu_percent(prev: u64, cur: u64, dt_secs: f64) -> f32 {
    let delta_ticks = cur.saturating_sub(prev) as f64;
    let cpu_secs = delta_ticks / 100.0;
    let cores = thread::available_parallelism()
        .map(|n| n.get() as f64)
        .unwrap_or(1.0);
    ((cpu_secs / (dt_secs * cores)) * 100.0).clamp(0.0, 100.0) as f32
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
