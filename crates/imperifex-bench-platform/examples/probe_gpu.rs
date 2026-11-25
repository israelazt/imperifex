//! Probe 1.2: how the GPU engine counters name their instances and what reading
//! them costs. Read-only.
//!
//! `cargo run --release -p imperifex-bench-platform --example probe_gpu`

#[cfg(windows)]
#[path = "common/processes.rs"]
mod processes;

#[cfg(windows)]
#[path = "common/stats.rs"]
mod stats;

#[cfg(windows)]
#[path = "probe_gpu/pdh.rs"]
mod pdh;

#[cfg(windows)]
fn main() -> Result<(), String> {
    const ROUNDS: usize = 10;
    let found: Vec<_> = processes::processes()?
        .into_iter()
        .filter(|p| p.is("warp.exe"))
        .collect();
    for process in &found {
        println!("warp.exe pid {} (padre {})", process.pid, process.parent);
    }
    let warp: Vec<u32> = found.iter().map(|p| p.pid).collect();

    let query = pdh::Query::open(pdh::GPU_ENGINES)?;
    query.collect()?;
    let (mut collects, mut reads) = (Vec::new(), Vec::new());
    let mut last = Vec::new();
    let mut totals_3d = Vec::new();
    for _ in 0..ROUNDS {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let before = stats::ticks();
        query.collect()?;
        let collected = stats::ticks();
        last = query.values()?;
        collects.push(stats::ms(before, collected));
        reads.push(stats::ms(collected, stats::ticks()));
        totals_3d.push(
            of_pids(&last, &warp)
                .filter(|(name, _)| name.ends_with("engtype_3D"))
                .map(|(_, value)| value)
                .sum::<f64>(),
        );
    }

    println!("instancias en total: {}", last.len());
    println!("instancias de warp en la última lectura:");
    for (name, value) in of_pids(&last, &warp) {
        println!("  {name} = {value:.3} %");
    }
    println!("suma 3D de warp por segundo (%): {totals_3d:.2?}");
    println!("PdhCollectQueryData: {}", stats::spread(collects));
    println!("leer el arreglo: {}", stats::spread(reads));
    Ok(())
}

#[cfg(windows)]
fn of_pids<'a>(
    values: &'a [(String, f64)],
    pids: &'a [u32],
) -> impl Iterator<Item = &'a (String, f64)> {
    values.iter().filter(|(name, _)| {
        pids.iter()
            .any(|pid| name.starts_with(&format!("pid_{pid}_")))
    })
}

#[cfg(not(windows))]
fn main() {}
