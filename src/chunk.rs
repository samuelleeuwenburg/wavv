use crate::error::Error;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryInto;

/// RIFF chunks are tagged with 4 byte identifiers.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ChunkTag {
    /// Root level "chunk"
    Riff,
    /// Mandatory chunk for WAV files, contains data such as the sample rate, bit depth, and number of channels.
    Fmt,
    /// Mandatory chunk for WAV files, contains the (interleaved) samples.
    Data,
    /// File identifier, should be located right after the RIFF tag and chunk size
    Wave,
    /// Unkown/unhandled chunk tag, useful for parsing [`Chunk`] bytes.
    Unknown([u8; 4]),
}

impl ChunkTag {
    fn from_bytes(bytes: &[u8; 4]) -> Self {
        match bytes {
            [b'R', b'I', b'F', b'F'] => ChunkTag::Riff,
            [b'f', b'm', b't', b' '] => ChunkTag::Fmt,
            [b'd', b'a', b't', b'a'] => ChunkTag::Data,
            [b'W', b'A', b'V', b'E'] => ChunkTag::Wave,
            _ => ChunkTag::Unknown(*bytes),
        }
    }

    fn to_bytes(self) -> [u8; 4] {
        match self {
            ChunkTag::Riff => [b'R', b'I', b'F', b'F'],
            ChunkTag::Fmt => [b'f', b'm', b't', b' '],
            ChunkTag::Data => [b'd', b'a', b't', b'a'],
            ChunkTag::Wave => [b'W', b'A', b'V', b'E'],
            ChunkTag::Unknown(bytes) => bytes,
        }
    }
}

/// Resource Interchange File Format (RIFF) tagged chunk.
#[derive(Debug)]
pub struct Chunk {
    /// Chunk tag
    pub id: ChunkTag,
    /// Raw bytes for parsing based on the associated tag.
    pub bytes: Vec<u8>,
}

impl Chunk {
    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let id = bytes[0..4]
            .try_into()
            .map_err(|_| Error::CantParseSliceInto)
            .map(ChunkTag::from_bytes)?;

        let size = bytes[4..8]
            .try_into()
            .map_err(|_| Error::CantParseSliceInto)
            .map(|b| u32::from_le_bytes(b))?;

        let start = 8;
        let end = 8 + size as usize;
        let bytes: Vec<u8> = bytes[start..end].to_vec();

        Ok(Chunk { id, bytes })
    }

    pub(crate) fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];

        bytes.extend_from_slice(&self.id.to_bytes());
        bytes.extend_from_slice(&(self.bytes.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.bytes);

        bytes
    }
}

pub fn parse_chunks(bytes: &[u8]) -> Result<Vec<Chunk>, Error> {
    let mut chunks: Vec<Chunk> = vec![];

    let riff = Chunk::from_bytes(bytes)?;

    if riff.id != ChunkTag::Riff {
        return Err(Error::NoRiffChunkFound);
    }

    let tag: [u8; 4] = riff.bytes[0..4].try_into().unwrap();

    if tag != ChunkTag::Wave.to_bytes() {
        return Err(Error::NoWaveTagFound);
    }

    let mut index = 4;

    while index < riff.bytes.len() {
        let chunk = Chunk::from_bytes(&riff.bytes[index..])?;

        // Chunks should always have an even number of bytes,
        // if it is odd there is an empty padding byte at the end
        let chunk_length = chunk.bytes.len();
        let padding_byte = (chunk_length & 1) * 8;

        index += 8 + chunk_length + padding_byte;

        chunks.push(chunk);
    }

    Ok(chunks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;

    #[test]
    fn should_parse_chunks() {
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

        let chunks = parse_chunks(&bytes).unwrap();

        assert_eq!(chunks.len(), 2);
        assert!(chunks.iter().find(|c| c.id == ChunkTag::Fmt).is_some());
        assert!(chunks.iter().find(|c| c.id == ChunkTag::Data).is_some());
    }

    #[test]
    fn should_fail_on_non_wave_files() {
        let bytes: [u8; 60] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x34, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x56, // WAVV
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

        assert_eq!(parse_chunks(&bytes).unwrap_err(), Error::NoWaveTagFound);
    }

    #[test]
    fn should_not_fail_with_random_chunks_added() {
        let bytes: [u8; 88] = [
            0x52, 0x49, 0x46, 0x46, // RIFF
            0x34, 0x00, 0x00, 0x00, // chunk size
            0x57, 0x41, 0x56, 0x56, // WAVV
            0x66, 0x6d, 0x74, 0x20, // fmt_
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x01, 0x00, // audio format
            0x02, 0x00, // num channels
            0x22, 0x56, 0x00, 0x00, // sample rate
            0x88, 0x58, 0x01, 0x00, // byte rate
            0x04, 0x00, // block align
            0x10, 0x00, // bits per sample
            0x72, 0x6e, 0x64, 0x6d, // rndm
            0x04, 0x00, 0x00, 0x00, // chunk size
            0xaa, 0xaa, 0xaa, 0xaa, // ...
            0x8b, 0xad, 0xf0, 0x0d, // 8badfood
            0x08, 0x00, 0x00, 0x00, // chunk size
            0xaa, 0xff, 0xaa, 0xff, // ...
            0xff, 0xaa, 0xff, 0xaa, // ...
            0x64, 0x61, 0x74, 0x61, // data
            0x10, 0x00, 0x00, 0x00, // chunk size
            0x00, 0x00, 0x00, 0x00, // sample 1 L+R
            0x24, 0x17, 0x1e, 0xf3, // sample 2 L+R
            0x3c, 0x13, 0x3c, 0x14, // sample 3 L+R
            0x16, 0xf9, 0x18, 0xf9, // sample 4 L+R
        ];

        assert_eq!(parse_chunks(&bytes).unwrap_err(), Error::NoWaveTagFound);
    }
}
