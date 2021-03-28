//! Very basic `#![no_std]` library for reading wav files.
//!
//! **NOTE! this library is unfinished, incomplete and still contains bugs!**
//!
//! ```rust
//! use std::fs;
//! use std::path::Path;
//! use wavv::{Wave, Samples};
//!
//! fn main() {
//!     let bytes = fs::read(Path::new("./test_files/sine_mono.wav")).unwrap();
//! 	let wave = Wave::from_bytes(&bytes).unwrap();
//!
//!     println!(
//!         "sample rate: {}, channels: {}, bit depth: {}",
//!         wave.format.sample_rate, wave.format.bit_depth, wave.format.num_channels
//!     );
//!
//!     match wave.data {
//!         Samples::BitDepth8(samples) => println!("{:?}", samples),
//!         Samples::BitDepth16(samples) => println!("{:?}", samples),
//!         Samples::BitDepth24(samples) => println!("{:?}", samples),
//!     }
//! }
//! ```

#![no_std]
#![warn(missing_docs)]

extern crate alloc;

mod parsing;
mod wave;

pub use wave::{Format, Samples, Wave};
