# Development

You will need:

- The latest [Rust stable](https://rustup.rs/)
- The [MSVC toolchain](https://visualstudio.microsoft.com/vs/features/cplusplus/)
- On Linux:
  - [cargo xwin](https://github.com/rust-cross/cargo-xwin)

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
ER_PATCHES_PATH="C:/Videogames/EldenRingPatches"
```
