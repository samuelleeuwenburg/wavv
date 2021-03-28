# wavv
[![.github/workflows/main.yml](https://github.com/samuelleeuwenburg/wavv/actions/workflows/main.yml/badge.svg)](https://github.com/samuelleeuwenburg/wavv/actions/workflows/main.yml)
[![Crates.io](https://img.shields.io/crates/v/wavv.svg)](https://crates.io/crates/wavv)
[![docs.rs](https://docs.rs/wavv/badge.svg)](https://docs.rs/wavv/)

Very basic `#![no_std]` library for reading wav files.

**NOTE! this library is unfinished, incomplete and still contains bugs!**

```rust
use std::fs;
use std::path::Path;
use wavv::{Wave, Samples};

fn main() {
    let bytes = fs::read(Path::new("foo.wav")).unwrap();
	let wave = Wave::from_bytes(&bytes).unwrap();

    println!(
        "sample rate: {}, channels: {}, bit depth: {}",
        wave.format.sample_rate, wave.format.bit_depth, wave.format.num_channels
    );

    match wave.data {
        Samples::BitDepth8(samples) => println!("{:?}", samples),
        Samples::BitDepth16(samples) => println!("{:?}", samples),
        Samples::BitDepth24(samples) => println!("{:?}", samples),
    }
}
```
