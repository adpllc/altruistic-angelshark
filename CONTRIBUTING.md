# Contributing

## Guidelines

- This project uses
  [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/)
- This project uses [semantic versioning](https://semver.org/)
- Please fork, work on a feature branch, and then submit a PR
- Please prefer preserving backwards compatibility

## Getting Started

- A text editor; any will do, but if you use VSCode, it will ask if you want the
  recommended extensions for this project
- [The Rust Toolchain and `cargo`](https://rustup.rs)
- GNU Make (optional but helpful)

To compile:

`$ cargo build`

To run:

`$ cargo run --bin angelsharkcli`

or

`$ cargo run --bin angelsharkd`

To open the library documentation:

`$ cargo doc --no-deps --open`

## Finding Work To Do

- Check the project issue tracker for fixes and feature requests
- Search for `todo!()` macros in the code itself; this typically indicates an
  unimplemented function or method that needs work.
- Search for `TODO:` comments in the code itself
