use devclean_core::disk::disk_info;
use devclean_core::memory::memory_info;
use devclean_core::processes::top_processes;
use devclean_core::{area_sizes, docker, human, Context};

use crate::style::{bold, dim, flag};

const HEAVY_PROCESS: u64 = 2 * 1024 * 1024 * 1024;

pub fn print_report(ctx: &Context) -> Result<(), devclean_core::CoreError> {
    print_memory()?;
    print_disk_and_caches(ctx)
}

fn print_memory() -> Result<(), devclean_core::CoreError> {
    let memory = memory_info();
    let free = match memory.free_percent {
        Some(free) => format!("{free}%"),
        None => "?%".to_string(),
    };
    println!("{}", bold("MEMORY"));
    println!(
        "  total {}   free {}   swap used {}",
        human(memory.total),
        flag(&free, memory.is_low()),
        human(memory.swap_used)
    );
    for process in top_processes(8)? {
        let size = format!("{:>9}", human(process.bytes));
        let name: String = process.name.chars().take(60).collect();
        println!("  {}  {name}", flag(&size, process.bytes > HEAVY_PROCESS));
    }
    if docker::is_running() {
        println!("  {}", dim("docker containers"));
        for container in docker::container_memory()?.into_iter().take(8) {
            println!("    {:>10}  {}", container.usage, container.name);
        }
    }
    Ok(())
}

fn print_disk_and_caches(ctx: &Context) -> Result<(), devclean_core::CoreError> {
    let disk = disk_info(&ctx.home)?;
    println!("\n{}", bold("DISK"));
    println!(
        "  free {} of {} ({}%)",
        flag(&human(disk.free), disk.is_low()),
        human(disk.total),
        disk.free_percent
    );
    println!("\n{}  {}", bold("CACHES"), dim("(devclean clean <area>)"));
    for size in area_sizes(ctx) {
        let safe = if size.safe { "safe" } else { "ask " };
        println!(
            "  {:>9}  {:<11} [{safe}] {}",
            human(size.bytes),
            size.area.name(),
            size.label
        );
    }
    Ok(())
}
