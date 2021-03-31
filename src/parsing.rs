use crate::wave::{Error, Header, Samples};
use alloc::vec;
use alloc::vec::Vec;
use core::convert::TryInto;

// @TODO: https://exiftool.org/TagNames/RIFF.html#Info
#[derive(Debug, PartialEq, Clone)]
pub enum ChunkID {
    RIFF,
    FMT,
    LIST,
    DATA,
    JUNK,
    WAVE,
}

impl ChunkID {
    fn from_bytes(bytes: &[u8; 4]) -> Result<ChunkID, Error> {
        match bytes {
            [b'R', b'I', b'F', b'F'] => Ok(ChunkID::RIFF),
            [b'f', b'm', b't', b' '] => Ok(ChunkID::FMT),
            [b'L', b'I', b'S', b'T'] => Ok(ChunkID::LIST),
            [b'd', b'a', b't', b'a'] => Ok(ChunkID::DATA),
            [b'J', b'U', b'N', b'K'] => Ok(ChunkID::JUNK),
            [b'W', b'A', b'V', b'E'] => Ok(ChunkID::WAVE),
            _ => Err(Error::UnknownChunkID(bytes.clone())),
        }
    }
}

#[derive(Debug)]
pub enum Chunk {
    FMT(Header),
    DATA(Samples),
    Unknown(ChunkID, Vec<u8>),
}

impl Chunk {
    fn from_bytes_with_id_and_header(
        header: &Header,
        id: &ChunkID,
        bytes: &[u8],
    ) -> Result<Option<Chunk>, Error> {
        let chunk = match id {
            ChunkID::DATA => {
                let samples = Samples::from_bytes(header, bytes)?;
                Some(Chunk::DATA(samples))
            }
            _ => None,
        };

        Ok(chunk)
    }

    fn from_bytes_with_id(id: &ChunkID, bytes: &[u8]) -> Result<Option<Chunk>, Error> {
        let chunk = match id {
            ChunkID::FMT => {
                let header = Header::from_bytes(bytes)?;
                Some(Chunk::FMT(header))
            }
            _ => None,
        };

        Ok(chunk)
    }
}

fn parse_chunk(bytes: &[u8]) -> Result<(ChunkID, &[u8], &[u8]), Error> {
    let chunk_id = bytes[0..4]
        .try_into()
        .map_err(|e| Error::CantParseSliceInto(e))
        .and_then(ChunkID::from_bytes)?;

    let chunk_size = bytes[4..8]
        .try_into()
        .map_err(|e| Error::CantParseSliceInto(e))
        .map(|b| u32::from_le_bytes(b))?;

    let (start, end) = match chunk_id {
        ChunkID::RIFF => (12, 8 + chunk_size as usize),
        _ => (8, 8 + chunk_size as usize),
    };

    Ok((chunk_id, &bytes[start..end], &bytes[end..]))
}

pub fn parse_chunks(bytes: &[u8]) -> Result<(Vec<Chunk>, Option<Vec<Chunk>>), Error> {
    let (chunk_id, wave_data, _) = parse_chunk(bytes)?;

    if chunk_id != ChunkID::RIFF {
        return Err(Error::NoRiffChunkFound);
    }

    let (chunk_id, fmt_data, tail) = parse_chunk(wave_data)?;

    let first_chunk = Chunk::from_bytes_with_id(&chunk_id, fmt_data)?
        .ok_or(Error::CantParseChunk(chunk_id.clone()))?;

    let header = match &first_chunk {
        Chunk::FMT(header) => Ok(header.clone()),
        _ => return Err(Error::NoRiffChunkFound),
    }?;

    let mut chunks = vec![first_chunk];
    let mut unknown_chunks: Option<Vec<Chunk>> = None;
    let mut tail = tail;

    loop {
        if tail.len() == 0 {
            break;
        }

        let (chunk_id, chunk_bytes, new_tail) = parse_chunk(tail)?;
        tail = new_tail;

        match Chunk::from_bytes_with_id_and_header(&header, &chunk_id, chunk_bytes)? {
            Some(chunk) => chunks.push(chunk),
            None => {
                let chunk = Chunk::Unknown(chunk_id.clone(), chunk_bytes.into());
                match unknown_chunks.as_mut() {
                    Some(v) => v.push(chunk),
                    None => {
                        unknown_chunks.replace(vec![chunk]);
                    }
                }
            }
        }
    }

    Ok((chunks, unknown_chunks))
}
