use sysinfo::System;

pub fn calculate_workers(mode: &str, env_mem_bytes: usize) -> usize {
    let num_cpus = num_cpus::get();

    if mode == "single" {
        return 1;
    }

    let mut sys = System::new_all();
    sys.refresh_memory();

    let available_mem = sys.available_memory() as usize;
    let half_mem = available_mem / 2;

    // 1環境のメモリと利用可能メモリ半分の大きい方を採用
    let target_mem = std::cmp::max(env_mem_bytes, half_mem);
    let mem_workers = target_mem / env_mem_bytes;

    match mode {
        "half-memory" => std::cmp::min(num_cpus, std::cmp::max(1, mem_workers)),
        "max" => num_cpus,
        _ => 1,
    }
}