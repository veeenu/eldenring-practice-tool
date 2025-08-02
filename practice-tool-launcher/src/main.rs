// johndisandonato's Elden Ring Practice Tool
// Copyright (C) 2022  johndisandonato <https://github.com/veeenu>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published
// by the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use anyhow::Result;
use practice_tool_launcher::LauncherConfig;

fn launcher() -> Result<()> {
    let title = "Elden Ring Practice Tool Launcher";
    let launcher_config = LauncherConfig {
        game_appid: 1245620,
        game_exe_name: "eldenring.exe",
        game_exe_subpath: "Game/eldenring.exe",
        config_file_name: "jdsd_er_practice_tool.toml",
        tool_exe_path: "jdsd_er_practice_tool.exe",
    };

    if let Err(e) = practice_tool_launcher::run(title, launcher_config) {
        eprintln!("Eframe error: {e:?}");
    }

    Ok(())
}

#[cfg(windows)]
mod injector;

#[cfg(windows)]
fn main() -> Result<()> {
    let mut argv = std::env::args();
    let _ = argv.next();

    if let Some(arg) = argv.next()
        && arg == "--inject"
    {
        injector::run()
    } else {
        launcher()
    }
}

#[cfg(not(windows))]
fn main() -> Result<()> {
    launcher()
}
