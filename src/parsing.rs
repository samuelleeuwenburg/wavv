use crate::wave::{Error, Header, Samples};
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryInto;

// @TODO: https://exiftool.org/TagNames/RIFF.html#Info
#[derive(Debug, PartialEq, Clone)]
pub enum ChunkId {
    RIFF,
    FMT,
    LIST,
    DATA,
    JUNK,
    WAVE,
    INFO,
    Other([u8; 4]),
}

impl ChunkId {
    pub fn from_bytes(bytes: &[u8; 4]) -> ChunkId {
        match bytes {
            [b'R', b'I', b'F', b'F'] => ChunkId::RIFF,
            [b'f', b'm', b't', b' '] => ChunkId::FMT,
            [b'L', b'I', b'S', b'T'] => ChunkId::LIST,
            [b'd', b'a', b't', b'a'] => ChunkId::DATA,
            [b'J', b'U', b'N', b'K'] => ChunkId::JUNK,
            [b'W', b'A', b'V', b'E'] => ChunkId::WAVE,
            [b'I', b'N', b'F', b'O'] => ChunkId::INFO,
            _ => ChunkId::Other(bytes.clone()),
        }
    }
}

#[derive(Debug)]
pub enum Chunk {
    FMT(Header),
    DATA(Samples),
    LIST((ChunkId, Vec<Chunk>)),
    Unknown(ChunkId, Vec<u8>),
}

impl Chunk {
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

                let chunk = parse_chunks(header, &bytes[4..])?;

                Chunk::LIST((chunk_id, chunk))
            }
            _ => Chunk::Unknown(id.clone(), bytes.into()),
        };

        Ok(chunk)
    }

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
