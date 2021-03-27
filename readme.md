# wavv
[![.github/workflows/main.yml](https://github.com/samuelleeuwenburg/wavv/actions/workflows/main.yml/badge.svg)](https://github.com/samuelleeuwenburg/wavv/actions/workflows/main.yml)
[![Crates.io](https://img.shields.io/crates/v/wavv.svg)](https://crates.io/crates/wavv)

Basic `#![no_std]` library for reading wav files

example usage:

```rust
use std::fs;
use std::path::Path;
use wavv::Wave;

fn main() {
    let bytes = fs::read(Path::new("./foo.wav"));
	let wave = Wave::from_bytes(&bytes).unwrap();
}
```
