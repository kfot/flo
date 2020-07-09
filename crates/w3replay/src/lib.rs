use bytes::buf::ext::Reader;
use bytes::Buf;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::Path;

mod block;
mod constants;
mod header;
mod records;

pub mod error;
use block::Blocks;
pub use constants::*;
use error::*;
pub use header::Header;
pub use records::*;

#[derive(Debug)]
pub struct W3Replay<R> {
  header: Header,
  blocks: Blocks<R>,
}

impl W3Replay<BufReader<File>> {
  pub fn open<P: AsRef<Path>>(path: P) -> Result<W3Replay<BufReader<File>>> {
    use flo_util::binary::BinDecode;

    let f = File::open(path)?;
    let len = f.metadata()?.len() as usize;
    let mut r = BufReader::new(f);
    let mut buf: [u8; Header::MIN_SIZE] = [0; Header::MIN_SIZE];
    r.read_exact(&mut buf).map_err(Error::ReadHeader)?;
    let mut buf_slice = &buf[..];
    let header = Header::decode(&mut buf_slice).map_err(|e| e.context("header"))?;
    Ok(W3Replay {
      blocks: Blocks::new(r, header.num_blocks as usize, len - Header::MIN_SIZE),
      header,
    })
  }
}

impl<B> W3Replay<Reader<B>>
where
  B: Buf,
{
  pub fn from_buf(mut buf: B) -> Result<W3Replay<Reader<B>>> {
    use flo_util::binary::BinDecode;
    let header = Header::decode(&mut buf).map_err(|e| e.context("header"))?;
    Ok(W3Replay {
      blocks: Blocks::from_buf(buf, header.num_blocks as usize),
      header,
    })
  }
}

impl<R> W3Replay<R> {
  pub fn into_records(self) -> RecordIter<R> {
    RecordIter::new(self.blocks)
  }
}

#[test]
fn test_open() {
  let path = flo_util::sample_path!("replay", "16k.w3g");
  for r in W3Replay::open(&path).unwrap().into_records() {
    let r = r.unwrap();
    if r.type_id() == RecordTypeId::GameInfo {
      dbg!(r);
      break;
    }
  }
}