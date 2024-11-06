//! Basic `no_std` library for parsing and creating WAV files.
//!
//! Reading a WAV file:
//! ```
//! use std::fs;
//! use std::path::Path;
//! use wavv::{Wav, Data};
//!
//! fn main() {
//!     let bytes = fs::read(Path::new("./test_files/stereo_16_48000.wav")).unwrap();
//! 	let wav = Wav::from_bytes(&bytes).unwrap();
//!
//!     assert_eq!(wav.fmt.num_channels, 2);
//!     assert_eq!(wav.fmt.bit_depth, 16);
//!     assert_eq!(wav.fmt.sample_rate, 48_000);
//!
//!     match wav.data {
//!         Data::BitDepth8(samples) => println!("{:?}", samples),
//!         Data::BitDepth16(samples) => println!("{:?}", samples),
//!         Data::BitDepth24(samples) => println!("{:?}", samples),
//!     }
//! }
//! ```
//!
//! Writing a WAV file:
//! ```
//! use std::fs::File;
//! use std::io::Write;
//! use std::path::Path;
//! use wavv::{Wav, Data};
//!
//! fn main() {
//!     // Enjoy the silence
//!     let data = Data::BitDepth16(vec![0; 480_000]);
//! 	let wav = Wav::from_data(data, 48_000, 2);
//!
//!     let path = Path::new("output.wav");
//!     let mut file = File::create(&path).unwrap();
//!     file.write_all(&wav.to_bytes()).unwrap();
//! }
//! ```

#![cfg_attr(not(test), no_std)]
#![warn(missing_docs)]

extern crate alloc;

mod chunk;
mod data;
mod error;
mod fmt;
mod wav;

pub use chunk::{Chunk, ChunkTag};
pub use data::Data;
pub use error::Error;
pub use fmt::Fmt;
pub use wav::Wav;
