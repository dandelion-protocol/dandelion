use alloc::{
    fmt::{Result, Write},
    string::String,
    vec::Vec,
};
use core::str::from_utf8;

pub trait Printable {
    fn print(&self, writer: &mut dyn Write) -> Result;

    fn as_printed(&self) -> String {
        let mut result = String::new();
        self.print(&mut result).unwrap();
        result
    }
}

impl<T: Printable> Printable for [T] {
    fn print(&self, writer: &mut dyn Write) -> Result {
        if self.is_empty() {
            writer.write_str("[]")
        } else {
            writer.write_str("[ ")?;
            self[0].print(writer)?;
            let count = self.len();
            for index in 1..count {
                writer.write_str(", ")?;
                self[index].print(writer)?;
            }
            writer.write_str(" ]")
        }
    }
}

impl<T: Printable> Printable for Vec<T> {
    fn print(&self, writer: &mut dyn Write) -> Result {
        <Vec<T> as AsRef<[T]>>::as_ref(self).print(writer)
    }
}

pub fn print_public_bytes(writer: &mut dyn Write, data: &[u8]) -> Result {
    const SIZE_LIMIT: usize = 96;
    if data.len() >= SIZE_LIMIT {
        return write!(writer, "... {} bytes omitted", data.len());
    }
    writer.write_char('"')?;
    print_base64(writer, data)?;
    writer.write_char('"')
}

pub fn print_secret_bytes(writer: &mut dyn Write) -> Result {
    writer.write_str("<redacted>")
}

const BASE64_DIGITS: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const BASE64_PAD: u8 = b'=';

pub fn print_base64(writer: &mut dyn Write, data: &[u8]) -> Result {
    for chunk in data.chunks(3) {
        let tmp = make_base64(chunk);
        writer.write_str(from_utf8(&tmp).unwrap())?;
    }
    Ok(())
}

fn make_base64(chunk: &[u8]) -> [u8; 4] {
    match chunk.len() {
        3 => make_base64_full(chunk[0], chunk[1], chunk[2]),
        2 => {
            let mut tmp = make_base64_full(chunk[0], chunk[1], 0);
            tmp[3] = BASE64_PAD;
            tmp
        },
        1 => {
            let mut tmp = make_base64_full(chunk[0], 0, 0);
            tmp[3] = BASE64_PAD;
            tmp[2] = BASE64_PAD;
            tmp
        },
        _ => unreachable!(),
    }
}

fn make_base64_full(a: u8, b: u8, c: u8) -> [u8; 4] {
    let num = u32::from_be_bytes([0, a, b, c]);
    let ch0 = BASE64_DIGITS[((num >> 18) & 63) as usize];
    let ch1 = BASE64_DIGITS[((num >> 12) & 63) as usize];
    let ch2 = BASE64_DIGITS[((num >> 6) & 63) as usize];
    let ch3 = BASE64_DIGITS[((num >> 0) & 63) as usize];
    [ch0, ch1, ch2, ch3]
}
