# eldenring-practice-tool

A tool for practicing speedruns. It is compatible with all Elden Ring patches.
Made with ❤️ by [johndisandonato](https://twitch.tv/johndisandonato) .

To run the tool, extract all files from the zip archive and double-click the
`.exe` file he tool will automatically appear over the game, and it can be
toggled by pressing `0`.

You can download the latest release [here](https://github.com/veeenu/eldenring-practice-tool/releases).

If you need help, please read the [FAQ](#troubleshooting--faq) section for
solutions or ways to get in touch.

## Troubleshooting / FAQ

### I found a bug. What do I do?

- Set the `log_level = "DEBUG"` option in `jdsd_er_practice_tool.toml`.
- Reproduce the steps that cause your bug.
- Go [here](https://github.com/veeenu/eldenring-practice-tool/issues/new)
  and submit a new issue, explaining the problem and attaching the
  `jdsd_er_practice_tool.log` file.

I'll do my best to get back to you and fix the bug.

### Where are all the key bindings?

You can customize the default ones or add your own by editing
`jdsd_er_practice_tool.toml` with your favorite text editor.

The bundled file contains all possible settings with predefined hotkeys and is
mostly self-explanatory.

You can find a list of supported hotkey codes [here](https://github.com/veeenu/darksoulsiii-practice-tool/blob/7aa6ac33c6f155d35d0fa99ab100c8caa13913f9/practice-tool/src/util/vk.rs#L15-L186).

### What versions of the game are supported?

All of them! When new patches come out, a new release with compatibility will be drafted as soon as
possible.

### I want to talk to you!

You can contact me on Discord at `johndisandonato#4484` or on [Twitter](https://twitter.com/johndisandonato).

### I want to watch your speedruns!

Sure! See you over here 👉 [https://twitch.tv/johndisandonato](https://twitch.tv/johndisandonato)!

## Credits

- The [Soulsmodding community](http://soulsmodding.wikidot.com/) for the
  [Param definitions](https://github.com/soulsmods/Paramdex) and the
  Cheat Engine table maintained by Pav.
- Pav and wasted for technical help in figuring out pointers.
- [curiouspeanut](https://www.twitch.tv/curiouspeanut), [Weider96](https://www.twitch.tv/weider96),
  [Siegbruh](https://twitch.tv/siegbruh), [catalystz](https://www.twitch.tv/catalystz) for beta
  testing the tool.

## Development

You will need:

- A recent (Rust nightly)[https://rustup.rs/]
- The (MSVC toolchain)[https://visualstudio.microsoft.com/vs/features/cplusplus/]

Most building functions are exposed by the (xtasks)[https://github.com/matklad/cargo-xtask].

### Run the tool

```
cargo xtask run
```

This task will compile and run the practice tool from the repo.

### Distribution artifacts

```
cargo xtask dist
```

This task will create release artifacts in `target/dist/jdsd_er_practice_tool.zip`.

### Codegen

```
cargo xtask codegen
```

This task is responsible for generating Rust code from various external sources.
Examples: params from [Paramdex](https://github.com/soulsmods/Paramdex), base pointers for
array-of-byte scans from the Elden Ring executables.

### Environment

Some tasks require you to have environment variables defined that are dependent on your system.
You can put all your task-specific environment variables in a `.env` file in the top level directory
of the project. Complete example:

```
$ cat .env
ERPT_PATCHES_PATH="C:/Videogames/EldenRingPatches"
```
