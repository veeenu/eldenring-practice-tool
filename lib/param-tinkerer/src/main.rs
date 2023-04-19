use hudhook::inject::Process;
use hudhook::tracing::trace;
use tracing_subscriber::filter::LevelFilter;

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
    trace!("Injecting {:?}", dll_path);

    Process::by_title("ELDEN RING™")
        .map_err(|e| e.to_string())?
        .inject(dll_path)
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_ansi(true)
        .init();
    perform_injection().unwrap();
}
