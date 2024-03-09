use std::path::PathBuf;

use hudhook::inject::Process;

fn main() {
    let mut args = std::env::args();
    args.next().unwrap();

    let dll: PathBuf = args.next().unwrap().into();

    let process = Process::by_name("eldenring.exe").expect("Could not find process");
    process.inject(dll).expect("Could not inject DLL");
}
