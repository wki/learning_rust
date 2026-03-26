# Learning Rust - Project Guide

A collection of Rust learning projects exploring async networking, CLI tooling, actor patterns, HTTP, and embedded systems.

## Project Structure

Each subdirectory is an independent Cargo workspace:

| Crate | Purpose | Key Deps |
|---|---|---|
| `tcp_listener` | Synchronous TCP echo server (stdlib only) | — |
| `concurrent_tcp_listener` | Async TCP server with in-memory key-value store, MPSC channels, graceful shutdown | `tokio` |
| `actor_demo` | Actor-based concurrency patterns | `tokio` |
| `cmdline` | CLI argument parsing demo | `clap` |
| `http_client` | Async HTTP client fetching weather data from wttr.in | `reqwest`, `tokio` |
| `http_server` | HTTP server with dynamic routing | `actix-web`, `tokio` |
| `pico_w_blink` | Raspberry Pi Pico W embedded app: LED, BLE advertising, joystick ADC, OLED display | `embassy-*`, `cyw43`, `trouble-host` |

## Build & Run

Each project is built independently from its own directory:

```sh
cd <crate_name>
cargo run
```

The `pico_w_blink` crate targets `thumbv6m-none-eabi` and requires the embedded toolchain.

## Conventions

- Rust edition 2024 (except `pico_w_blink` which uses 2021)
- Async runtime: Tokio (full features) for networking crates
- Servers bind to port 8000 by default
