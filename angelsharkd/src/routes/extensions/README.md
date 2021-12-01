# Angelshark Daemon Extensions

This module aims to provide a simple way of extending Angelshark's basic
functionality (running commands on the ACM) with additional HTTP endpoints that
run one or more commands to achieve a basic business task.

This functionality may not be desirable for all end users, and therefore is
completely opt-in with feature flags. For example, at compile time, you can add
`--features simple_search` to enable a given extension called `simple_search`.

To add additional features, read `mod.rs` and `Cargo.toml` for `angelsharkd` to
see how to conditionally incorporate your own warp HTTP filters into the
project.
