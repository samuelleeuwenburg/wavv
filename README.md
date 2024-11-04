# wavv
[![.github/workflows/main.yml](https://github.com/samuelleeuwenburg/wavv/actions/workflows/main.yml/badge.svg)](https://github.com/samuelleeuwenburg/wavv/actions/workflows/main.yml)
[![Crates.io](https://img.shields.io/crates/v/wavv.svg)](https://crates.io/crates/wavv)
[![docs.rs](https://docs.rs/wavv/badge.svg)](https://docs.rs/wavv/)


Basic `no_std` library for parsing and creating WAV files.

Reading a WAV file:
```
use std::fs;
use std::path::Path;
use wavv::{Wav, Data};

fn main() {
    let bytes = fs::read(Path::new("./test_files/stereo_16_48000.wav")).unwrap();
	let wav = Wav::from_bytes(&bytes).unwrap();

    assert_eq!(wav.fmt.num_channels, 2);
    assert_eq!(wav.fmt.bit_depth, 16);
    assert_eq!(wav.fmt.sample_rate, 48_000);

    match wav.data {
        Data::BitDepth8(samples) => println!("{:?}", samples),
        Data::BitDepth16(samples) => println!("{:?}", samples),
        Data::BitDepth24(samples) => println!("{:?}", samples),
    }
}
```

Writing a WAV file:
```
use std::fs::File;
use std::io::Write;
use std::path::Path;
use wavv::{Wav, Data};

fn main() {
    let data = Data::BitDepth16(vec![0, 0, 0, 0, 0, 0]);
	let wav = Wav::from_data(data, 48_000, 2);

    let path = Path::new("output.wav");
    let mut file = File::create(&path).unwrap();
    file.write_all(&wav.to_bytes()).unwrap();
}
```
