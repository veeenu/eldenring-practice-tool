# Elden Ring Practice Tool

[![build](https://github.com/veeenu/eldenring-practice-tool/actions/workflows/build.yml/badge.svg)](https://github.com/veeenu/eldenring-practice-tool/actions)
[![GitHub all releases](https://img.shields.io/github/downloads/veeenu/eldenring-practice-tool/total)](https://github.com/veeenu/eldenring-practice-tool/releases/latest)
[![GitHub](https://img.shields.io/github/license/veeenu/eldenring-practice-tool)](https://github.com/veeenu/eldenring-practice-tool/blob/main/LICENSE)

A tool for practicing speedruns. It is compatible with all Elden Ring patches.

Made with ❤️ by [johndisandonato](https://twitch.tv/johndisandonato).

To run the tool, extract all files from the zip archive and double-click the
`.exe` file he tool will automatically appear over the game, and it can be
toggled by pressing `0`.

You can download the latest release [here](https://github.com/veeenu/eldenring-practice-tool/releases/latest).

If you need help, **please first read** the [Known Issues](#known-issues) and [FAQ](#troubleshooting--faq) sections for
solutions, or ways to get in touch.

# Known issues

## Fullscreen freeze (Issue: https://github.com/veeenu/eldenring-practice-tool/issues/23)

The tool presently freezes the game if it is run in fullscreen mode. This is a high priority bug but chances are
it will take a while to study and fix. In the meantime, you can run the game in Windowed or Borderless Windowed mode.

## Stake of Marika instant quitouts (Issue: https://github.com/veeenu/eldenring-practice-tool/issues/6)

If you use the *instant quitout* feature in a Stake of Marika area while dying, the usual choice dialog will pop up.
If you choose "Stake of Marika", you will spawn dead. If you choose "Last visited Grace", you will
instantly quitout instead, as requested.

Always choose the Grace. Hopefully, in the future, a workaround will be found.

## Character rotation in teleportation function (Issue: https://github.com/veeenu/eldenring-practice-tool/issues/15)

When saving/loading position, the character doesn't retain rotation appropriately as the rotation
assignment algorithm is rather enigmatic. Loading the position many times will make the rotation
converge to the intended one.

## Spawning on horseback deathcam oddity (Issue: https://github.com/veeenu/eldenring-practice-tool/issues/5)

If you spawn on horseback, the `deathcam` flag will not work properly at first.
It will just lock the camera in place. It is enough to get off the horse and then the
flag should work as intended again. If it doesn't work, a quitout will most likely fix it.

# Troubleshooting / FAQ

## I found an issue. What do I do?

- Set the `log_level = "DEBUG"` option in `jdsd_er_practice_tool.toml`.
- Reproduce the steps that cause your bug.
- Go [here](https://github.com/veeenu/eldenring-practice-tool/issues/new)
  and submit a new issue, explaining the problem and attaching the
  `jdsd_er_practice_tool.log` file.

I'll do my best to get back to you and fix the bug.

## Where are all the key bindings?

You can customize the default ones or add your own by editing
`jdsd_er_practice_tool.toml` with your favorite text editor.

The bundled file contains all possible settings with predefined hotkeys and is
mostly self-explanatory.

You can find a list of supported hotkey codes [here](https://github.com/veeenu/darksoulsiii-practice-tool/blob/7aa6ac33c6f155d35d0fa99ab100c8caa13913f9/practice-tool/src/util/vk.rs#L15-L186).

## What versions of the game are supported?

All of them! When new patches come out, a new release with compatibility will be drafted as soon as possible.

## Will I get banned if I use this online?

Use at your own risk. Bans are unlikely, but in doubt, make backups of your savefiles and only use the tool offline.
By using the tool, you agree that I will not be held liable for any bans or unintended side effects resulting from the usage of the tool.

## My game crashes / the tool doesn't start

- Always start with a clean zip of the latest release.
- EAC needs to be [bypassed](https://wiki.speedsouls.com/eldenring:EAC_Bypass).
- When in doubt, wait for the main menu of the game to appear before launching the tool.
- If you are running in [fullscreen](https://github.com/veeenu/eldenring-practice-tool/issues/23), try borderless or windowed mode.
- Antivirus software and old Windows versions will interact poorly with the tool, as it
  employs some techniques that are usually typical of malware. Don't worry, the tool is
  safe! The source code is fully available and auditable in this repository.
- If all else fails, [submit an issue](#i-found-an-issue-what-do-i-do).

## I want to talk to you!

You can contact me on Discord at `johndisandonato#4484` or on [Twitter](https://twitter.com/johndisandonato).

## I want to watch your speedruns!

Sure! See you over here 👉 [https://twitch.tv/johndisandonato](https://twitch.tv/johndisandonato)!

# Credits

- The [Soulsmodding community](http://soulsmodding.wikidot.com/) for the
  [Param definitions](https://github.com/soulsmods/Paramdex) and the
  Cheat Engine table maintained by Pav.
- Pav, wasted, jamesq7 for technical help in figuring things out.
- [curiouspeanut](https://twitch.tv/curiouspeanut), [Weider96](https://twitch.tv/weider96),
  [Siegbruh](https://twitch.tv/siegbruh), [catalystz](https://twitch.tv/catalystz),
  [danisangb](https://twitch.tv/danisangb), [GiantCookieJar](https://twitch.tv/GiantCookieJar),
  [Ahady](https://twitch.tv/ahady), [Gin](https://twitch.tv/g1nnz),
  [Nemz38](https://twitch.tv/nemz38), [Noobest](https://twitch.tv/noobest),
  for beta testing the tool.

# Development

You will need:

- A recent [Rust nightly](https://rustup.rs/)
- The [MSVC toolchain](https://visualstudio.microsoft.com/vs/features/cplusplus/)

Most building functions are exposed by the [xtasks](https://github.com/matklad/cargo-xtask).

## Run the tool

```
cargo xtask run
```

This task will compile and run the practice tool from the repo.

## Distribution artifacts

```
cargo xtask dist
```

This task will create release artifacts in `target/dist/jdsd_er_practice_tool.zip`.

## Code generation

```
cargo xtask codegen
```

This task is responsible for generating Rust code from various external sources.
Examples: params from [Paramdex](https://github.com/soulsmods/Paramdex), base pointers for
array-of-byte scans from the Elden Ring executables.

## Environment

Some tasks require you to have environment variables defined that are dependent on your system.
You can put all your task-specific environment variables in a `.env` file in the top level directory
of the project. Complete example:

```
$ cat .env
ERPT_PATCHES_PATH="C:/Videogames/EldenRingPatches"
```
