use hudhook::inject::Process;
use hudhook::log;
use simplelog::*;

fn err_to_string<T: std::fmt::Display>(e: T) -> String {
    format!("Error: {}", e)
}

fn perform_injection() -> Result<(), String> {
    let mut dll_path = std::env::current_exe().unwrap();
    dll_path.pop();
    dll_path.push("jdsd_er_param_tinkerer.dll");

    if !dll_path.exists() {
        dll_path.pop();
        dll_path.push("libjdsd_er_param_tinkerer.dll");
        dll_path.set_extension("dll");
    }

    let dll_path = dll_path.canonicalize().map_err(err_to_string)?;
    log::trace!("Injecting {:?}", dll_path);

    Process::by_title("ELDEN RING™")
        .map_err(|e| e.to_string())?
        .inject(dll_path)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    CombinedLogger::init(vec![TermLogger::new(
        LevelFilter::Trace,
        ConfigBuilder::new().build(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )])
    .ok();
    log::info!("test");
    perform_injection().unwrap();
}
