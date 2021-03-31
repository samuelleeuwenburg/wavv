use crate::parsing::{parse_chunk, parse_chunks, Chunk, ChunkId};
use alloc::vec;
use alloc::vec::Vec;
use core::array::TryFromSliceError;
use core::convert::TryInto;

/// Error type for different parsing failures
#[derive(Debug, Clone)]
pub enum Error {
    /// Unknown or unsupported Chunk ID
    UnknownChunkID([u8; 4]),
    /// Failed parsing slice into specific bytes
    CantParseSliceInto(TryFromSliceError),
    /// Failed parsing chunk with given id
    CantParseChunk(ChunkId),
    /// no riff chunk found in file, probably not a valid RIFF file
    NoRiffChunkFound,
    /// no wave id found in file, probably not a valid WAV file
    NoWaveIdFound,
    /// no data chunk found in file
    NoDataChunkFound,
    /// no fmt/header chunk found in file
    NoFmtChunkFound,
    /// unsupported bit depth
    UnsupportedBitDepth(u16),
}

/// Struct representing the header section of a .wav file
///
/// for more information see [`here`]
///
/// [`here`]: http://soundfile.sapp.org/doc/WaveFormat/
#[derive(Debug, Clone)]
pub struct Header {
    /// sample rate, typical values are `44_100`, `48_000` or `96_000`
    pub sample_rate: u32,
    /// number of audio channels in the sample data, channels are interleaved
    pub num_channels: u16,
    /// bit depth for each sample, typical values are `16` or `24`
    pub bit_depth: u16,
}

impl Header {
    /// Create new [`Header`] instance from a slice of bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use wavv::Header;
    ///
    /// fn main() {
    ///     let bytes = [
    ///         0x01, 0x00, // audio format
    ///         0x01, 0x00, // num channels
    ///         0x44, 0xac, 0x00, 0x00, // sample rate
    ///         0x88, 0x58, 0x01, 0x00, // byte rate
    ///         0x04, 0x00, // block align
    ///         0x18, 0x00, // bits per sample
    ///     ];
    ///
    ///     let header = Header::from_bytes(&bytes).unwrap();
    ///
    ///     assert_eq!(header.num_channels, 1);
    ///     assert_eq!(header.bit_depth, 24);
    ///     assert_eq!(header.sample_rate, 44_100);
    /// }
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let num_channels = bytes[2..4]
            .try_into()
            .map_err(|e| Error::CantParseSliceInto(e))
            .map(|b| u16::from_le_bytes(b))?;

        let sample_rate = bytes[4..8]
            .try_into()
            .map_err(|e| Error::CantParseSliceInto(e))
            .map(|b| u32::from_le_bytes(b))?;

        let bit_depth = bytes[14..16]
            .try_into()
            .map_err(|e| Error::CantParseSliceInto(e))
            .map(|b| u16::from_le_bytes(b))?;

        Ok(Header {
            num_channels,
            sample_rate,
            bit_depth,
        })
    }
}

/// Enum to hold samples for different bit depth
#[derive(Debug, PartialEq)]
pub enum Samples {
    /// 8 bit audio
    BitDepth8(Vec<u8>),
    /// 16 bit audio
    BitDepth16(Vec<i16>),
    /// 24 bit audio
    BitDepth24(Vec<i32>),
}

impl Samples {
    /// Create new [`Samples`] instance from a slice of bytes
    /// this requires a [`Header`] instance to be passed to determine
    /// the sample size and channel data etc.
    ///
    /// # Examples
    ///
    /// ```
    /// use wavv::{Header, Samples};
    ///
    /// fn main() {
    ///     let bytes = [
    ///         0x01, 0x00, // audio format
    ///         0x01, 0x00, // num channels
    ///         0x44, 0xac, 0x00, 0x00, // sample rate
    ///         0x88, 0x58, 0x01, 0x00, // byte rate
    ///         0x04, 0x00, // block align
    ///         0x18, 0x00, // bits per sample
    ///     ];
    ///
    ///     let header = Header::from_bytes(&bytes).unwrap();
    ///
    ///     assert_eq!(header.num_channels, 1);
    ///     assert_eq!(header.bit_depth, 24);
    ///     assert_eq!(header.sample_rate, 44_100);
    ///
    ///     let bytes = [
    ///         0x00, 0x00, 0x00, // sample 1
    ///         0x00, 0x24, 0x17, // sample 2
    ///         0x1e, 0xf3, 0x3c, // sample 3
    ///         0x13, 0x3c, 0x14, // sample 4
    ///     ];
    ///
    ///     let samples = Samples::from_bytes(&header, &bytes).unwrap();
    ///
    ///     assert_eq!(
    ///         samples,
    ///         Samples::BitDepth24(vec![
    ///     	    0x00000000, // sample 1
    ///     	    0x17240000, // sample 2
    ///     	    0x3cf31e00, // sample 3
    ///     	    0x143c1300, // sample 4
    ///         ])
    ///     )
    /// }
    /// ```
    pub fn from_bytes(header: &Header, bytes: &[u8]) -> Result<Self, Error> {
        let mut samples = match header.bit_depth {
            8 => Ok(Samples::BitDepth8(vec![])),
            16 => Ok(Samples::BitDepth16(vec![])),
            24 => Ok(Samples::BitDepth24(vec![])),
            _ => Err(Error::UnsupportedBitDepth(header.bit_depth)),
        }?;

        let num_bytes = (header.bit_depth / 8) as usize;
        let mut pos = 0;

        loop {
            if pos + num_bytes > bytes.len() {
                break;
            }
            let slice = &bytes[pos..pos + num_bytes];

            match samples {
                Samples::BitDepth8(ref mut v) => {
                    let sample = slice
                        .try_into()
                        .map_err(|e| Error::CantParseSliceInto(e))
                        .map(u8::from_le_bytes)?;

                    v.push(sample)
                }
                Samples::BitDepth16(ref mut v) => {
                    let sample = slice
                        .try_into()
                        .map_err(|e| Error::CantParseSliceInto(e))
                        .map(i16::from_le_bytes)?;

                    v.push(sample)
                }
                Samples::BitDepth24(ref mut v) => {
                    let sample = [0, slice[0], slice[1], slice[2]][0..4]
                        .try_into()
                        .map_err(|e| Error::CantParseSliceInto(e))
                        .map(i32::from_le_bytes)?;

                    v.push(sample)
                }
            }

            pos += num_bytes;
        }

        Ok(samples)
    }
}

/// Struct representing a .wav file
#[derive(Debug)]
pub struct Wave {
    /// Contains data from the fmt chunk / header part of the file
    pub header: Header,
    /// Contains audio data as samples of a fixed bit depth
    pub data: Samples,
}

impl Wave {
    /// Create new [`Wave`] instance from a slice of bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs;
    /// use std::path::Path;
    /// use wavv::Wave;
    ///
    /// fn main() {
    ///     let bytes = fs::read(Path::new("./test_files/sine_mono_16_44100.wav")).unwrap();
    ///     let wave = Wave::from_bytes(&bytes).unwrap();
    ///
    ///     assert_eq!(wave.header.num_channels, 1);
    ///     assert_eq!(wave.header.bit_depth, 16);
    ///     assert_eq!(wave.header.sample_rate, 44_100);
    /// }
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let (chunk_id, wave_data, _) = parse_chunk(bytes)?;

        let wave_chunk_id = wave_data[0..4]
            .try_into()
            .map_err(|e| Error::CantParseSliceInto(e))
            .map(ChunkId::from_bytes)?;

        if chunk_id != ChunkId::RIFF {
            return Err(Error::NoRiffChunkFound);
        }

        if wave_chunk_id != ChunkId::WAVE {
            return Err(Error::NoWaveIdFound);
        }

        let (chunk_id, fmt_data, tail) = parse_chunk(&wave_data[4..])?;

        let header_chunk = Chunk::from_bytes_with_id(&chunk_id, fmt_data)?
            .ok_or(Error::CantParseChunk(chunk_id.clone()))?;

        let header = match &header_chunk {
            Chunk::FMT(header) => Ok(header.clone()),
            _ => return Err(Error::NoRiffChunkFound),
        }?;

        let mut chunks = parse_chunks(&header, &tail)?;
        chunks.push(header_chunk);

        let mut data = Err(Error::NoDataChunkFound);
        let mut header = Err(Error::NoFmtChunkFound);

        for chunk in chunks {
            match chunk {
                Chunk::FMT(h) => header = Ok(h),
                Chunk::DATA(d) => data = Ok(d),
                Chunk::LIST(_) => (),
                // ignore unknown chunks
                Chunk::Unknown(_, _) => (),
            }
        }

        let wave = Wave {
            data: data?,
            header: header?,
        };

        Ok(wave)
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

        assert_eq!(wave.header.sample_rate, 22050);
        assert_eq!(wave.header.bit_depth, 16);
        assert_eq!(wave.header.num_channels, 2);

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

        assert_eq!(wave.header.sample_rate, 44100);
        assert_eq!(wave.header.bit_depth, 24);
        assert_eq!(wave.header.num_channels, 1);

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
