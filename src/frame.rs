use bytes::{Buf, Bytes};
use std::io::Cursor;

#[derive(Debug, Clone)]
pub enum Frame {
    Simple(String),
    Bulk(Bytes),
    Integer(i64),
    Array(Vec<Frame>),
}

impl Frame {
    pub fn parse(buf: &mut Cursor<&[u8]>) -> Frame {
        // first byte (u8) represents structure of the full request
        let repr_char = buf.get_u8();
        match repr_char {
            b'+' => {
                let bytes_vec = get_line(buf).to_vec();
                let simple_str =
                    String::from_utf8(bytes_vec).expect("failed to parse UTF-8 encoded strings ");

                return Frame::Simple(simple_str);
            }
            b':' => return Frame::Integer(get_u64(buf) as i64),
            b'$' => {
                let len = get_u64(buf);
                let data = Bytes::copy_from_slice(get_bytes(buf, len as usize));
                buf.advance((len + 2) as usize);
                return Frame::Bulk(data);
            }
            b'*' => {
                let len = get_u64(buf);
                let mut out = Vec::new();

                for _ in 0..len {
                    out.push(Frame::parse(buf));
                }

                return Frame::Array(out);
            }
            _ => unimplemented!(),
        }
    }

    pub fn encode(self) -> String {
        match &self {
            Frame::Simple(s) => format!("+{}\r\n", s.as_str()),
            Frame::Bulk(s) => {
                let s = String::from_utf8(s.to_vec()).unwrap();
                format!("${}\r\n{}\r\n", s.chars().count(), s)
            }
            _ => panic!("value encode not implemented for: {:?}", self),
        }
    }

    pub fn unwrap_bulk(&self) -> Bytes {
        if let Frame::Bulk(b) = self {
            return b.clone();
        }
        panic!("Trying to unwrap_bulk on a Frame that doesn't conform to Frame::Bulk");
    }

    pub fn unwrap_bulk_as_string(&self) -> String {
        let b = self.unwrap_bulk();
        let value_str = String::from_utf8(b.to_vec()).unwrap();
        return value_str;
    }

    pub fn unwrap_array(self) -> Vec<Frame> {
        if let Frame::Array(vf) = self {
            return vf;
        }
        panic!("Trying to unwrap_bulk on a Frame that doesn't conform to Frame::Bulk");
    }
}

fn get_bytes<'a>(buf: &mut Cursor<&'a [u8]>, n: usize) -> &'a [u8] {
    // n is the length of the bulk string
    let start = buf.position() as usize;
    let end = start + n;
    return &buf.get_ref()[start..end];
}

fn get_line<'a>(buf: &mut Cursor<&'a [u8]>) -> &'a [u8] {
    // scan the buffer directly
    // search from start, scan through max_end, stop if you find CRLF
    let start = buf.position() as usize;
    let max_end = buf.get_ref().len() - 1;

    for i in start..max_end {
        if buf.get_ref()[i] == b'\r' && buf.get_ref()[i + 1] == b'\n' {
            // advance the pointer manually to after CRLF characters
            buf.set_position((i + 2) as u64);
            return &buf.get_ref()[start..i];
        }
    }

    panic!("corrupted data !")
}

fn get_u64(buf: &mut Cursor<&[u8]>) -> u64 {
    let bytes_vec = get_line(buf).to_vec();
    let u64_str = String::from_utf8(bytes_vec).expect("failed to parse UTF-8 encoded strings ");

    let decimal = u64_str
        .parse::<u64>()
        .expect("failed to parse string to u64 ");

    return decimal;
}
