use crate::chunk::{Chunk, ChunkTag};
use crate::error::Error;
use crate::fmt::Fmt;
use alloc::vec;
use alloc::vec::Vec;

/// Enum to hold samples for different bit depths
#[derive(Debug, PartialEq)]
pub enum Data {
    /// 8 bit audio
    BitDepth8(Vec<u8>),
    /// 16 bit audio
    BitDepth16(Vec<i16>),
    /// 24 bit audio
    BitDepth24(Vec<i32>),
}

impl Data {
    pub(crate) fn from_chunk(fmt: &Fmt, chunk: &Chunk) -> Result<Self, Error> {
        let mut samples = match fmt.bit_depth {
            8 => Ok(Data::BitDepth8(vec![])),
            16 => Ok(Data::BitDepth16(vec![])),
            24 => Ok(Data::BitDepth24(vec![])),
            _ => Err(Error::UnsupportedBitDepth(fmt.bit_depth)),
        }?;

        let num_bytes = (fmt.bit_depth / 8) as usize;
        let mut pos = 0;

        while pos < chunk.bytes.len() {
            match &mut samples {
                Data::BitDepth8(s) => {
                    s.push(chunk.bytes[pos]);
                }
                Data::BitDepth16(s) => {
                    let sample = i16::from_le_bytes([chunk.bytes[pos], chunk.bytes[pos + 1]]);
                    s.push(sample);
                }
                Data::BitDepth24(s) => {
                    let sign = chunk.bytes[pos + 2] >> 7;
                    let sign_byte = if sign == 1 { 0xff } else { 0x0 };

                    let sample = i32::from_le_bytes([
                        chunk.bytes[pos],
                        chunk.bytes[pos + 1],
                        chunk.bytes[pos + 2],
                        sign_byte,
                    ]);

                    s.push(sample);
                }
            }

            pos += num_bytes;
        }

        Ok(samples)
    }

    pub(crate) fn to_chunk(&self) -> Chunk {
        let mut bytes = vec![];

        match self {
            Data::BitDepth8(samples) => {
                for s in samples {
                    bytes.push(*s);
                }
            }
            Data::BitDepth16(samples) => {
                for s in samples {
                    bytes.extend_from_slice(&s.to_le_bytes());
                }
            }
            Data::BitDepth24(samples) => {
                for s in samples {
                    let b = s.to_le_bytes();
                    bytes.extend_from_slice(&[b[0], b[1], b[2]]);
                }
            }
        }

        Chunk {
            id: ChunkTag::Data,
            bytes,
        }
    }

    /// Get the length of the internal sample Vec.
    pub fn len(&self) -> usize {
        match self {
            Data::BitDepth8(s) => s.len(),
            Data::BitDepth16(s) => s.len(),
            Data::BitDepth24(s) => s.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(overflowing_literals)]
    use super::*;
    use alloc::vec;

    #[test]
    fn to_8_bit() {
        let data = Data::BitDepth8(vec![1, 2, 3, 4]);
        assert_eq!(data.to_chunk().bytes, &[1, 2, 3, 4]);
    }

    #[test]
    fn to_16_bit() {
        let data = Data::BitDepth16(vec![1, 2, 3, 4]);
        assert_eq!(data.to_chunk().bytes, &[1, 0, 2, 0, 3, 0, 4, 0]);
    }

    #[test]
    fn to_24_bit() {
        let data = Data::BitDepth24(vec![1, 2, 3, 4]);
        assert_eq!(data.to_chunk().bytes, &[1, 0, 0, 2, 0, 0, 3, 0, 0, 4, 0, 0]);
    }

    #[test]
    fn from_8_bit() {
        let fmt = Fmt {
            bit_depth: 8,
            sample_rate: 48_000,
            num_channels: 1,
        };

        let bytes = [
            0x64, 0x61, 0x74, 0x61, // data
            0x04, 0x00, 0x00, 0x00, // chunk size
            0xff, 0xc0, 0xaa, 0x40, // sample 1, 2, 3, 4
        ];

        let data = Data::from_chunk(&fmt, &Chunk::from_bytes(&bytes).unwrap()).unwrap();

        assert_eq!(data, Data::BitDepth8(vec![255, 192, 170, 64]));
    }
    #[test]
    fn from_16_bit() {
        let fmt = Fmt {
            bit_depth: 16,
            sample_rate: 48_000,
            num_channels: 1,
        };

        let bytes = [
            0x64, 0x61, 0x74, 0x61, // data
            0x08, 0x00, 0x00, 0x00, // chunk size
            0xff, 0x7f, 0x00, 0x80, // sample 1, 2
            0xff, 0xff, 0x01, 0x00, // sample 3, 4
        ];

        let data = Data::from_chunk(&fmt, &Chunk::from_bytes(&bytes).unwrap()).unwrap();

        assert_eq!(data, Data::BitDepth16(vec![32767, -32768, -1, 1]));
    }

    #[test]
    fn from_24_bit() {
        let fmt = Fmt {
            bit_depth: 24,
            sample_rate: 48_000,
            num_channels: 1,
        };

        let bytes = [
            0x64, 0x61, 0x74, 0x61, // data
            0x0c, 0x00, 0x00, 0x00, // chunk size
            0xff, 0xff, 0x7f, // sample 1
            0x00, 0x00, 0x80, // sample 2
            0x01, 0x00, 0x00, // sample 3
            0xff, 0xff, 0xff, // sample 4
        ];

        let data = Data::from_chunk(&fmt, &Chunk::from_bytes(&bytes).unwrap()).unwrap();

        assert_eq!(data, Data::BitDepth24(vec![8_388_607, -8_388_608, 1, -1]));
    }
}
