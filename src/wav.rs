use crate::chunk::{parse_chunks, Chunk, ChunkTag};
use crate::data::Data;
use crate::error::Error;
use crate::fmt::Fmt;
use alloc::vec;
use alloc::vec::Vec;

/// Struct representing a WAV file
pub struct Wav {
    /// Contains data from the fmt chunk / header part of the file
    pub fmt: Fmt,
    /// Contains audio data as samples of a fixed bit depth
    pub data: Data,
    /// Contains raw chunk data that is either unimplemented or unknown
    pub chunks: Vec<Chunk>,
}

impl Wav {
    /// Create new [`Wav`] instance from a slice of bytes
    ///
    /// ```
    /// use std::fs;
    /// use std::path::Path;
    /// use wavv::Wav;
    ///
    /// let bytes = fs::read(Path::new("./test_files/mono_16_48000.wav")).unwrap();
    /// let wav = Wav::from_bytes(&bytes).unwrap();
    ///
    /// assert_eq!(wav.fmt.num_channels, 1);
    /// assert_eq!(wav.fmt.bit_depth, 16);
    /// assert_eq!(wav.fmt.sample_rate, 48_000);
    /// ```
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let parsed_chunks = parse_chunks(bytes)?;

        let fmt = parsed_chunks
            .iter()
            .find(|c| c.id == ChunkTag::Fmt)
            .ok_or(Error::NoFmtChunkFound)
            .and_then(|c| Fmt::from_chunk(&c))?;

        let data = parsed_chunks
            .iter()
            .find(|c| c.id == ChunkTag::Data)
            .ok_or(Error::NoDataChunkFound)
            .and_then(|c| Data::from_chunk(&fmt, &c))?;

        let chunks = parsed_chunks
            .into_iter()
            .filter(|c| c.id != ChunkTag::Data && c.id != ChunkTag::Fmt)
            .collect();

        let wave = Wav { data, fmt, chunks };

        Ok(wave)
    }

    /// Create a [`Wav`] struct from samples.
    ///
    /// ```
    /// use wavv::{Wav, Data};
    ///
    /// let samples = vec![0, 0, 0, 0];
    /// let wav = Wav::from_data(Data::BitDepth24(samples), 44_100, 2);
    ///
    /// assert_eq!(wav.fmt.num_channels, 2);
    /// assert_eq!(wav.fmt.bit_depth, 24);
    /// assert_eq!(wav.fmt.sample_rate, 44_100);
    /// ```
    pub fn from_data(data: Data, sample_rate: usize, num_channels: usize) -> Self {
        let bit_depth = match &data {
            Data::BitDepth8(_) => 8,
            Data::BitDepth16(_) => 16,
            Data::BitDepth24(_) => 24,
        };

        let fmt = Fmt {
            sample_rate: sample_rate as u32,
            num_channels: num_channels as u16,
            bit_depth,
        };

        Wav {
            data,
            fmt,
            chunks: vec![],
        }
    }

    /// Convert a [`Wav`] instance into bytes.
    ///
    /// Useful if you have raw sample data that you want to convert to a .wav file:
    ///
    /// ```
    /// use wavv::{Wav, Data};
    ///
    /// let wav = Wav::from_data(Data::BitDepth16(vec![1, 2, 3, -1]), 48_000, 2);
    ///
    /// let bytes: [u8; 52] = [
    ///     0x52, 0x49, 0x46, 0x46, // RIFF
    ///     0x2c, 0x00, 0x00, 0x00, // chunk size
    ///     0x57, 0x41, 0x56, 0x45, // WAVE
    ///     0x66, 0x6d, 0x74, 0x20, // fmt_
    ///     0x10, 0x00, 0x00, 0x00, // chunk size
    ///     0x01, 0x00, // audio format
    ///     0x02, 0x00, // num channels
    ///     0x80, 0xbb, 0x00, 0x00, // sample rate
    ///     0x00, 0xee, 0x02, 0x00, // byte rate
    ///     0x04, 0x00, // block align
    ///     0x10, 0x00, // bits per sample
    ///     0x64, 0x61, 0x74, 0x61, // data
    ///     0x08, 0x00, 0x00, 0x00, // chunk size
    ///     0x01, 0x00, 0x02, 0x00, // samples
    ///     0x03, 0x00, 0xff, 0xff, // samples
    /// ];
    ///
    /// assert_eq!(wav.to_bytes(), bytes);
    /// ```
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x00, 0x00, 0x00, 0x00, // chunk size (kept empty for later)
            0x57, 0x41, 0x56, 0x45, // WAVE
        ];

        bytes.extend_from_slice(&self.fmt.to_chunk().to_bytes());
        bytes.extend_from_slice(&self.data.to_chunk().to_bytes());

        // Subtract 8 for initial two words
        let chunk_size = (bytes.len() as u32 - 8).to_le_bytes();

        bytes[4] = chunk_size[0];
        bytes[5] = chunk_size[1];
        bytes[6] = chunk_size[2];
        bytes[7] = chunk_size[3];

        bytes
    }
}

#[cfg(test)]
mod tests {
    #![allow(overflowing_literals)]
    use super::*;
    use alloc::vec;
    use std::fs;
    use std::path::Path;

    #[test]
    fn parse_wav_16_bit_stereo() {
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
            0x00, 0x00, 0x01, 0x00, // sample 1 L+R
            0x02, 0x00, 0x03, 0x00, // sample 2 L+R
            0x04, 0x00, 0x05, 0x00, // sample 3 L+R
            0x06, 0x00, 0x07, 0x00, // sample 4 L+R
        ];

        let wav = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wav.fmt.sample_rate, 22050);
        assert_eq!(wav.fmt.bit_depth, 16);
        assert_eq!(wav.fmt.num_channels, 2);

        assert_eq!(
            wav.data,
            Data::BitDepth16(vec![
                0, 1, // sample 1 L+R
                2, 3, // sample 2 L+R
                4, 5, // sample 3 L+R
                6, 7, // sample 4 L+R
            ])
        );
    }

    #[test]
    fn parse_wav_24_bit_mono() {
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

        let wav = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wav.fmt.sample_rate, 44100);
        assert_eq!(wav.fmt.bit_depth, 24);
        assert_eq!(wav.fmt.num_channels, 1);

        assert_eq!(
            wav.data,
            Data::BitDepth24(vec![
                0x00000000, // sample 1
                0x00172400, // sample 2
                0x003cf31e, // sample 3
                0x00143c13, // sample 4
            ])
        );
    }

    #[test]
    fn parse_wav_24_bit_with_padding_byte() {
        let bytes: [u8; 48] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x28, 0x00, 0x00, 0x00, // chunk size
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
            0x03, 0x00, 0x00, 0x00, // chunk size
            0xff, 0xff, 0xff, // sample 1
            0x00, // padding byte
        ];

        let wav = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wav.fmt.sample_rate, 44100);
        assert_eq!(wav.fmt.bit_depth, 24);
        assert_eq!(wav.fmt.num_channels, 1);

        assert_eq!(wav.data, Data::BitDepth24(vec![-1]));
    }

    #[test]
    fn parse_wav_from_and_to_bytes_stereo() {
        let bytes: [u8; 60] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x34, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x45, // WAVE
            0x66, 0x6d, 0x74, 0x20, // fmt_
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x01, 0x00, // audio format
            0x02, 0x00, // num channels
            0x80, 0xbb, 0x00, 0x00, // sample rate
            0x00, 0xee, 0x02, 0x00, // byte rate
            0x04, 0x00, // block align
            0x10, 0x00, // bits per sample
            0x64, 0x61, 0x74, 0x61, // data
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x00, 0x00, 0x00, 0x00, // sample 1 L+R
            0x24, 0x17, 0x1e, 0xf3, // sample 2 L+R
            0x3c, 0x13, 0x3c, 0x14, // sample 3 L+R
            0x16, 0xf9, 0x18, 0xf9, // sample 4 L+R
        ];

        let wave = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wave.to_bytes(), bytes);
    }

    #[test]
    fn parse_wav_from_and_to_bytes_mono() {
        let bytes: [u8; 56] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x30, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x45, // WAVE
            0x66, 0x6d, 0x74, 0x20, // fmt_
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x01, 0x00, // audio format
            0x01, 0x00, // num channels
            0x80, 0xbb, 0x00, 0x00, // sample rate
            0x80, 0x32, 0x02, 0x00, // byte rate
            0x03, 0x00, // block align
            0x18, 0x00, // bits per sample
            0x64, 0x61, 0x74, 0x61, // data
            0x0c, 0x00, 0x00, 0x00, // chunk size
            0x00, 0x00, 0x00, // sample 1
            0x00, 0x00, 0x00, // sample 2
            0x00, 0x00, 0x00, // sample 3
            0x00, 0x00, 0x00, // sample 4
        ];

        let wave = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wave.to_bytes(), bytes);
    }

    #[test]
    fn parse_files() {
        let bytes = fs::read(Path::new("./test_files/mono_16_48000.wav")).unwrap();
        let wav = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wav.fmt.num_channels, 1);
        assert_eq!(wav.fmt.bit_depth, 16);
        assert_eq!(wav.fmt.sample_rate, 48_000);

        let bytes = fs::read(Path::new("./test_files/mono_24_48000.wav")).unwrap();
        let wav = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wav.fmt.num_channels, 1);
        assert_eq!(wav.fmt.bit_depth, 24);
        assert_eq!(wav.fmt.sample_rate, 48_000);

        let bytes = fs::read(Path::new("./test_files/stereo_16_48000.wav")).unwrap();
        let wav = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wav.fmt.num_channels, 2);
        assert_eq!(wav.fmt.bit_depth, 16);
        assert_eq!(wav.fmt.sample_rate, 48_000);

        let bytes = fs::read(Path::new("./test_files/stereo_24_48000.wav")).unwrap();
        let wav = Wav::from_bytes(&bytes).unwrap();

        assert_eq!(wav.fmt.num_channels, 2);
        assert_eq!(wav.fmt.bit_depth, 24);
        assert_eq!(wav.fmt.sample_rate, 48_000);
    }
}
