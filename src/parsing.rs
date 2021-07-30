use crate::wave::{Error, Header, Samples};
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryInto;

/// 4 byte chunk IDs used in RIFF files to describe various sections of data
#[derive(Debug, PartialEq, Clone)]
pub enum ChunkId {
    /// RIFF file header
    RIFF,
    /// FMT header containing wave file data
    FMT,
    /// RIFF list chunk containing an id and list of chunks
    LIST,
    /// Audio data
    DATA,
    /// Junk data
    JUNK,
    /// WAVE data
    WAVE,
    /// Info list, containing metadata
    INFO,
    /// Track title
    INAM,
    /// Artist
    IART,
    /// Album title
    IPRD,
    /// Software used to create the file
    ISFT,
    /// Creation date (YYYY-MM-DD or YYYY)
    ITCH,
    /// Genre
    IGNR,
    /// Secondary genre
    ISGN,
    /// Copywright information
    ICOP,
    /// Track number
    TRCK,
    /// Length
    TLEN,
    /// Unimplemented or unknow chunk id
    Other([u8; 4]),
}

impl ChunkId {
    /// Parse raw bytes into [`ChunkId`]
    pub fn from_bytes(bytes: &[u8; 4]) -> ChunkId {
        match bytes {
            [b'R', b'I', b'F', b'F'] => ChunkId::RIFF,
            [b'f', b'm', b't', b' '] => ChunkId::FMT,
            [b'L', b'I', b'S', b'T'] => ChunkId::LIST,
            [b'd', b'a', b't', b'a'] => ChunkId::DATA,
            [b'J', b'U', b'N', b'K'] => ChunkId::JUNK,
            [b'W', b'A', b'V', b'E'] => ChunkId::WAVE,
            [b'I', b'N', b'F', b'O'] => ChunkId::INFO,
            [b'I', b'N', b'A', b'M'] => ChunkId::INAM,
            [b'I', b'A', b'R', b'T'] => ChunkId::INAM,
            [b'I', b'P', b'R', b'D'] => ChunkId::IPRD,
            [b'I', b'S', b'F', b'T'] => ChunkId::ISFT,
            _ => ChunkId::Other(bytes.clone()),
        }
    }
}

/// Enum describing various .wav file data relationships
#[derive(Debug)]
pub enum Chunk {
    /// [`ChunkId::FMT`] containing header data
    FMT(Header),
    /// [`ChunkId::DATA`] containing sample data
    DATA(Samples),
    /// [`ChunkId::LIST`] containing other chunks
    LIST(ChunkId, Vec<Chunk>),
    /// [`ChunkId::INFO`] containing pairs of [`ChunkId`] combined with string data
    INFO(Vec<(ChunkId, String)>),
    /// Unkown or (most likely) unimplemented [`ChunkId`], containing raw bytes
    Unknown(ChunkId, Vec<u8>),
}

impl Chunk {
    /// attempt to parse bytes into valid [`Chunk`] based on [`ChunkId`] and [`Header`]
    pub fn from_bytes_with_id_and_header(
        header: &Header,
        id: &ChunkId,
        bytes: &[u8],
    ) -> Result<Chunk, Error> {
        let chunk = match id {
            ChunkId::DATA => {
                let samples = Samples::from_bytes(header, bytes)?;
                Chunk::DATA(samples)
            }
            ChunkId::LIST => {
                let chunk_id = bytes[0..4]
                    .try_into()
                    .map_err(|e| Error::CantParseSliceInto(e))
                    .map(ChunkId::from_bytes)?;

                let chunks = parse_chunks(header, &bytes[4..])?;

                Chunk::LIST(chunk_id, chunks)
            }

            _ => Chunk::Unknown(id.clone(), bytes.into()),
        };

        Ok(chunk)
    }

    /// attempt to parse bytes into valid [`Chunk`] based on [`ChunkId`]
    pub fn from_bytes_with_id(id: &ChunkId, bytes: &[u8]) -> Result<Option<Chunk>, Error> {
        let chunk = match id {
            ChunkId::FMT => {
                let header = Header::from_bytes(bytes)?;
                Some(Chunk::FMT(header))
            }
            _ => None,
        };

        Ok(chunk)
    }
}

pub fn parse_chunks(header: &Header, bytes: &[u8]) -> Result<Vec<Chunk>, Error> {
    let mut chunks = vec![];
    let mut tail = bytes;

    loop {
        if tail.len() == 0 {
            break;
        }

        let (chunk_id, chunk_bytes, new_tail) = parse_chunk(tail)?;
        tail = new_tail;

        let chunk = Chunk::from_bytes_with_id_and_header(&header, &chunk_id, chunk_bytes)?;
        chunks.push(chunk);
    }

    Ok(chunks)
}

pub fn parse_chunk(bytes: &[u8]) -> Result<(ChunkId, &[u8], &[u8]), Error> {
    let chunk_id = bytes[0..4]
        .try_into()
        .map_err(|e| Error::CantParseSliceInto(e))
        .map(ChunkId::from_bytes)?;

    let chunk_size = bytes[4..8]
        .try_into()
        .map_err(|e| Error::CantParseSliceInto(e))
        .map(|b| u32::from_le_bytes(b))?;

    let start = 8;
    let end = 8 + chunk_size as usize;

    Ok((chunk_id, &bytes[start..end], &bytes[end..]))
}
