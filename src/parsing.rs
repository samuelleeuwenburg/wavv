use crate::wave::{Error, Header, Samples, Wave};
use alloc::vec;
use alloc::vec::Vec;
use core::array::TryFromSliceError;
use core::convert::TryInto;

#[derive(Debug, PartialEq)]
pub enum ChunkID {
    RIFF,
    FMT,
    LIST,
    DATA,
    JUNK,
}

pub fn parse_chunk_id(id: [u8; 4]) -> Result<ChunkID, Error> {
    match id {
        [b'R', b'I', b'F', b'F'] => Ok(ChunkID::RIFF),
        [b'f', b'm', b't', b' '] => Ok(ChunkID::FMT),
        [b'L', b'I', b'S', b'T'] => Ok(ChunkID::LIST),
        [b'd', b'a', b't', b'a'] => Ok(ChunkID::DATA),
        [b'J', b'U', b'N', b'K'] => Ok(ChunkID::JUNK),
        _ => Err(Error::UnknownChunkID(id)),
    }
}

fn parse_fmt_chunk(bytes: &[u8]) -> Result<Header, Error> {
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

fn read_samples<T, F: Fn(&[u8]) -> Result<T, TryFromSliceError>>(
    format: &Header,
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

fn parse_data_chunk(format: &Header, bytes: &[u8]) -> Result<Samples, Error> {
    match format.bit_depth {
        8 => {
            let samples = read_samples(&format, bytes, |b| {
                b.try_into().map(|b| u8::from_le_bytes(b))
            });
            samples
                .map_err(|e| Error::CantParseSliceInto(e))
                .map(|s| Samples::BitDepth8(s))
        }
        16 => {
            let samples = read_samples(&format, bytes, |b| {
                b.try_into().map(|b| i16::from_le_bytes(b))
            });
            samples
                .map_err(|e| Error::CantParseSliceInto(e))
                .map(|s| Samples::BitDepth16(s))
        }
        24 => {
            let samples = read_samples(&format, bytes, |b| {
                [0, b[0], b[1], b[2]][0..4]
                    .try_into()
                    .map(|b| i32::from_le_bytes(b))
            });
            samples
                .map_err(|e| Error::CantParseSliceInto(e))
                .map(|s| Samples::BitDepth24(s))
        }
        bit_depth => Err(Error::UnsupportedBitDepth(bit_depth)),
    }
}

pub fn parse_chunks(bytes: &[u8]) -> Result<Wave, Error> {
    let mut pos = 0;

    let mut format = Err(Error::NoFmtChunkFound);
    let mut data = Err(Error::NoDataChunkFound);

    // @TODO: assume fmt chunk as the first chunk in file
    loop {
        if pos + 8 > bytes.len() {
            break;
        }

        let chunk_id = bytes[pos..pos + 4]
            .try_into()
            .map_err(|e| Error::CantParseSliceInto(e))
            .and_then(|b| parse_chunk_id(b))?;

        let chunk_size = bytes[pos + 4..pos + 8]
            .try_into()
            .map_err(|e| Error::CantParseSliceInto(e))
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
        header: format?,
        data: data?,
    };

    Ok(wave)
}
