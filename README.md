# WireGuard Manager

A CLI and TUI tool for managing WireGuard VPN configurations, written in Rust.

WireGuard Manager handles the full lifecycle of a WireGuard network: initialize subnets, add and edit servers and clients, automatically allocate IP addresses and generate cryptographic keys, and export ready-to-use `.conf` files.

## Features

- **Network initialization** — define subnet and server IP range with automatic validation
- **Device management** — add, edit, and delete servers and clients
- **Automatic key generation** — Curve25519 keypairs and preshared keys via `x25519-dalek`
- **Automatic IP allocation** — assigns the next available IP within configured ranges
- **Config generation** — produce standard WireGuard `.conf` files (single device or all at once)
- **Interactive TUI** — full-featured terminal UI with four-quadrant layout, syntax-highlighted config preview, and keyboard-driven workflow
- **CLI mode** — scriptable subcommands for automation and pipelines
- **Zero unsafe code** — `unsafe_code = "forbid"` enforced at the compiler level

## Building

Requires [Rust](https://www.rust-lang.org/tools/install) (edition 2024).

```sh
cargo build --release
```

The binary will be at `target/release/wireguard-manager` (or `.exe` on Windows).

## Usage

### Interactive TUI

Run without arguments to launch the terminal interface:

```sh
wireguard-manager
```

**TUI layout:**

| Top-left: Network Overview | Top-right: Device Detail |
|---|---|
| **Bottom-left: Device List** | **Bottom-right: Config Preview** |

Key bindings:

| Key | Action |
|---|---|
| `TAB` / `SHIFT+TAB` | Cycle panels |
| `?` | Help overlay |
| `CTRL+I` | Initialize / reinitialize network |
| `CTRL+E` | Edit network configuration |
| `A` | Add device (server or client) |
| `E` | Edit selected device |
| `D` | Delete selected device |
| `S` | Generate config for selected device |
| `G` | Generate configs for all devices |
| `Q` | Quit |

### CLI

```sh
wireguard-manager <COMMAND>
```

**Initialize a network:**

```sh
wireguard-manager init --subnet 10.0.0.0/24 --server-range [1,10]
```

**View / modify network configuration:**

```sh
wireguard-manager config show
wireguard-manager config set --subnet 10.0.0.0/16 --server-range [1,20]
```

**Add devices:**

```sh
wireguard-manager add server my-server --public-ip 203.0.113.1 --port 51820
wireguard-manager add client my-laptop
```

**Edit devices:**

```sh
wireguard-manager edit server my-server --public-ip 203.0.113.2
wireguard-manager edit client my-laptop --name new-name
```

**Delete devices:**

```sh
wireguard-manager del server my-server
wireguard-manager del client my-laptop
```

**Generate configuration files:**

```sh
wireguard-manager gen my-server
wireguard-manager gen all --output-dir ./configs
```

**List and inspect devices:**

```sh
wireguard-manager list
wireguard-manager show my-server
```

## Project Structure

```
src/
├── main.rs              Entry point: CLI args → cli, no args → TUI
├── cli/mod.rs           CLI argument definitions (clap) and command dispatch
├── manager/mod.rs       Core logic: data persistence, IP allocation, config generation
├── key/mod.rs           Curve25519 key generation and verification
└── tui/
    ├── mod.rs           Terminal setup, event loop
    ├── app/             App state machine, form handling, keyboard handlers
    ├── ui/              Layout rendering, modals, panel widgets
    ├── theme/           Centralized color palette and style factories
    └── input/           TextInput widget with cursor-aware editing
```

Data is stored in `data/data.json`. Generated configs are written to `output/` by default.

## Development

**Run tests:**

```sh
cargo test
```

**Lint:**

```sh
cargo clippy -- -D warnings
```

The project enforces Clippy pedantic + nursery lints and forbids unsafe code. See `Cargo.toml` `[lints]` for details.

## Dependencies

| Crate | Purpose |
|---|---|
| `clap` | CLI argument parsing |
| `serde` + `serde_json` | JSON serialization |
| `x25519-dalek` | WireGuard key generation |
| `base64` | Key encoding |
| `ipnet` | Subnet and IP range handling |
| `anyhow` | Error handling |
| `ratatui` + `crossterm` | Terminal UI framework |
| `chrono` | Timestamps |
| `rand` | Random number generation |
