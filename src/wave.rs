use crate::parsing::{parse_chunk_id, parse_chunks, ChunkID};
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
pub enum Samples {
    BitDepth8(Vec<u8>),
    BitDepth16(Vec<i16>),
    BitDepth24(Vec<i32>),
}

#[derive(Debug, Clone)]
pub struct Format {
    pub sample_rate: u32,
    pub num_channels: u16,
    pub bit_depth: u16,
}

#[derive(Debug)]
pub struct Wave {
    pub format: Format,
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

#[cfg(test)]
mod tests {
    #![allow(overflowing_literals)]
    use super::*;
    use alloc::vec;

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
