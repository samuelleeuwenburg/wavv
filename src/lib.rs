//! Very basic `#![no_std]` library for reading wav files.
//!
//! **NOTE! this library is unfinished, incomplete and still contains bugs!**
//!
//! ```
//! use std::fs;
//! use std::path::Path;
//! use wavv::{Wave, Samples};
//!
//! fn main() {
//!     let bytes = fs::read(Path::new("./test_files/sine_stereo_16_48000.wav")).unwrap();
//! 	let wave = Wave::from_bytes(&bytes).unwrap();
//!
//!     assert_eq!(wave.header.num_channels, 2);
//!     assert_eq!(wave.header.bit_depth, 16);
//!     assert_eq!(wave.header.sample_rate, 48_000);
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

pub use wave::{Error, Header, Samples, Wave};
