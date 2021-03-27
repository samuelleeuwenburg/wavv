#![no_std]

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;
use core::array::TryFromSliceError;
use core::convert::TryInto;

#[derive(Debug, Clone)]
pub enum Error {
    UnknownChunkID([u8; 4]),
    CantParseSamples(TryFromSliceError),
    CantReadChunkID,
    CantReadChunkSize,
    CantReadNumChannels,
    CantReadSampleRate,
    CantReadBitDepth,
    NoRiffChunkFound,
    NoDataChunkFound,
    NoFmtChunkFound,
    UnsupportedBitDepth,
}

#[derive(Debug, PartialEq)]
enum ChunkID {
    RIFF,
    FMT,
    LIST,
    DATA,
    JUNK,
}

#[derive(Debug, PartialEq)]
pub enum Samples {
    BitDepth8(Vec<u8>),
    BitDepth16(Vec<i16>),
    BitDepth24(Vec<i32>),
}

#[derive(Debug, Clone)]
pub struct WaveFormat {
    pub sample_rate: u32,
    pub num_channels: u16,
    pub bit_depth: u16,
}

#[derive(Debug)]
pub struct Wave {
    pub format: WaveFormat,
    pub data: Samples,
}

impl Wave {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let riff = bytes[0..4]
            .try_into()
            .map_err(|_| Error::CantReadChunkID)
            .and_then(|b| parse_chunk_id(b))?;

        let file_size = bytes[4..8]
            .try_into()
            .map_err(|_| Error::CantReadChunkSize)
            .map(|b| u32::from_le_bytes(b))?;

        if riff != ChunkID::RIFF {
            return Err(Error::NoRiffChunkFound);
        }

        parse_chunks(&bytes[12..file_size as usize + 8])
    }
}

fn parse_chunk_id(id: [u8; 4]) -> Result<ChunkID, Error> {
    match id {
        [b'R', b'I', b'F', b'F'] => Ok(ChunkID::RIFF),
        [b'f', b'm', b't', b' '] => Ok(ChunkID::FMT),
        [b'L', b'I', b'S', b'T'] => Ok(ChunkID::LIST),
        [b'd', b'a', b't', b'a'] => Ok(ChunkID::DATA),
        [b'J', b'U', b'N', b'K'] => Ok(ChunkID::JUNK),
        _ => Err(Error::UnknownChunkID(id)),
    }
}

fn parse_fmt_chunk(bytes: &[u8]) -> Result<WaveFormat, Error> {
    let num_channels = bytes[2..4]
        .try_into()
        .map_err(|_| Error::CantReadNumChannels)
        .map(|b| u16::from_le_bytes(b))?;

    let sample_rate = bytes[4..8]
        .try_into()
        .map_err(|_| Error::CantReadSampleRate)
        .map(|b| u32::from_le_bytes(b))?;

    let bit_depth = bytes[14..16]
        .try_into()
        .map_err(|_| Error::CantReadBitDepth)
        .map(|b| u16::from_le_bytes(b))?;

    Ok(WaveFormat {
        num_channels,
        sample_rate,
        bit_depth,
    })
}

fn read_samples<T, F: Fn(&[u8]) -> Result<T, TryFromSliceError>>(
    format: &WaveFormat,
    bytes: &[u8],
    read_sample: F,
) -> Result<Vec<T>, TryFromSliceError> {
    let num_bytes = (format.bit_depth / 8) as usize;
    let mut pos = 0;
    let mut samples = vec![];

    loop {
        if pos + num_bytes > bytes.len() {
            break;
        }

        samples.push(read_sample(&bytes[pos..pos + num_bytes])?);

        pos += num_bytes;
    }

    Ok(samples)
}

fn parse_data_chunk(format: &WaveFormat, bytes: &[u8]) -> Result<Samples, Error> {
    match format.bit_depth {
        8 => {
            let samples = read_samples(&format, bytes, |b| {
                b.try_into().map(|b| u8::from_le_bytes(b))
            });
            samples
                .map_err(|e| Error::CantParseSamples(e))
                .map(|s| Samples::BitDepth8(s))
        }
        16 => {
            let samples = read_samples(&format, bytes, |b| {
                b.try_into().map(|b| i16::from_le_bytes(b))
            });
            samples
                .map_err(|e| Error::CantParseSamples(e))
                .map(|s| Samples::BitDepth16(s))
        }
        24 => {
            let samples = read_samples(&format, bytes, |b| {
                [0, b[0], b[1], b[2]][0..4]
                    .try_into()
                    .map(|b| i32::from_le_bytes(b))
            });
            samples
                .map_err(|e| Error::CantParseSamples(e))
                .map(|s| Samples::BitDepth24(s))
        }
        _ => Err(Error::UnsupportedBitDepth),
    }
}

fn parse_chunks(bytes: &[u8]) -> Result<Wave, Error> {
    let mut pos = 0;

    let mut format = Err(Error::NoFmtChunkFound);
    let mut data = Err(Error::NoDataChunkFound);

    loop {
        if pos + 8 > bytes.len() {
            break;
        }

        let chunk_id = bytes[pos..pos + 4]
            .try_into()
            .map_err(|_| Error::CantReadChunkID)
            .and_then(|b| parse_chunk_id(b))?;

        let chunk_size = bytes[pos + 4..pos + 8]
            .try_into()
            .map_err(|_| Error::CantReadChunkSize)
            .map(|b| u32::from_le_bytes(b))?;

        let start = pos + 8;
        let end = pos + 8 + chunk_size as usize;

        match chunk_id {
            ChunkID::FMT => format = parse_fmt_chunk(&bytes[start..end]),
            ChunkID::DATA => data = parse_data_chunk(&format.clone()?, &bytes[start..end]),
            _ => (),
        };

        pos += chunk_size as usize + 8;
    }

    let wave = Wave {
        format: format?,
        data: data?,
    };

    Ok(wave)
}

#[cfg(test)]
mod tests {
    #![allow(overflowing_literals)]
    use super::*;

    #[test]
    fn test_parse_wave_16_bit_stereo() {
        let bytes: [u8; 60] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x34, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x45, // WAVE
            0x66, 0x6d, 0x74, 0x20, // fmt_
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x01, 0x00, // audio format
            0x02, 0x00, // num channels
            0x22, 0x56, 0x00, 0x00, // sample rate
            0x88, 0x58, 0x01, 0x00, // byte rate
            0x04, 0x00, // block align
            0x10, 0x00, // bits per sample
            0x64, 0x61, 0x74, 0x61, // data
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x00, 0x00, 0x00, 0x00, // sample 1 L+R
            0x24, 0x17, 0x1e, 0xf3, // sample 2 L+R
            0x3c, 0x13, 0x3c, 0x14, // sample 3 L+R
            0x16, 0xf9, 0x18, 0xf9, // sample 4 L+R
        ];

        let wave = Wave::from_bytes(&bytes).unwrap();

        assert_eq!(wave.format.sample_rate, 22050);
        assert_eq!(wave.format.bit_depth, 16);
        assert_eq!(wave.format.num_channels, 2);

        assert_eq!(
            wave.data,
            Samples::BitDepth16(vec![
                0x0000, 0x0000, // sample 1 L+R
                0x1724, 0xf31e, // sample 2 L+R
                0x133c, 0x143c, // sample 3 L+R
                0xf916, 0xf918, // sample 4 L+R
            ])
        );
    }

    #[test]
    fn test_parse_wave_24_bit_mono() {
        let bytes: [u8; 56] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x30, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x45, // WAVE
            0x66, 0x6d, 0x74, 0x20, // fmt_
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x01, 0x00, // audio format
            0x01, 0x00, // num channels
            0x44, 0xac, 0x00, 0x00, // sample rate
            0x88, 0x58, 0x01, 0x00, // byte rate
            0x04, 0x00, // block align
            0x18, 0x00, // bits per sample
            0x64, 0x61, 0x74, 0x61, // data
            0x0c, 0x00, 0x00, 0x00, // chunk size
            0x00, 0x00, 0x00, // sample 1
            0x00, 0x24, 0x17, // sample 2
            0x1e, 0xf3, 0x3c, // sample 3
            0x13, 0x3c, 0x14, // sample 4
        ];

        let wave = Wave::from_bytes(&bytes).unwrap();

        assert_eq!(wave.format.sample_rate, 44100);
        assert_eq!(wave.format.bit_depth, 24);
        assert_eq!(wave.format.num_channels, 1);

        assert_eq!(
            wave.data,
            Samples::BitDepth24(vec![
                0x00000000, // sample 1
                0x17240000, // sample 2
                0x3cf31e00, // sample 3
                0x143c1300, // sample 4
            ])
        );
    }
}
