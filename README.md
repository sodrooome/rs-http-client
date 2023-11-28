## rust-http-client

A simple HTTP client wrapped around [reqwest](https://docs.rs/reqwest/latest/reqwest/) but with the requests python's looks-like

## Features

- exponential backoff and retry
- prepared HTTP request before make a calls
- built-in logging function

## Usage

Simply move it over to the `examples` directory, and feel free to customize it using the existing APIs. Afterwards, run the code with

```bash
cargo run --package rs-http-request --example main
```

For running all the curated tests, you can run it with the

```bash
cargo test
```