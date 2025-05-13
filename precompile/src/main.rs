use std::{fs, io::Read, path::PathBuf};

use wasmtime::{Config, Engine};
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about=None)]
struct Cli {
    file: PathBuf,
    #[arg(short, long)]
    output: Option<PathBuf>
}

fn main() {
    let cli = Cli::parse();
    let file = fs::read(&cli.file).unwrap();

    
    let mut config = Config::default();
    config.target("x86_64-unknown-none").unwrap();
    // config.collector(wasmtime::Collector::DeferredReferenceCounting);
    config.memory_init_cow(false);
    config.memory_reservation(0);
    config.memory_reservation_for_growth(0);
    config.memory_guard_size(0);
    config.signals_based_traps(false);
    config.debug_info(false);
    config.memory_may_move(false);
    config.guard_before_linear_memory(false);
    config.table_lazy_init(false);
    config.wasm_backtrace(false);
    config.wasm_bulk_memory(false);
    config.cranelift_nan_canonicalization(false);
    // config.wasm_gc(false);


    let engine = Engine::new(&config).unwrap();
    
    let module = engine.precompile_module(&file).unwrap();
    fs::write(cli.output.unwrap_or(cli.file.with_extension("cwasm")), module).unwrap();
}
