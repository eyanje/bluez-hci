use libc::{c_int};
use std::io::{Read, Result, Write};

// Traits for reading events

pub trait ReadFrom: Sized {
    fn read_from<R: Read>(r: R) -> Result<(Self, usize)>;
}


pub trait ReadAs {
    fn read_as<V: ReadFrom>(&mut self) -> Result<(V, usize)>;
}

impl <R: Read> ReadAs for R {
    fn read_as<V: ReadFrom>(&mut self) -> Result<(V, usize)> {
        V::read_from(self)
    }
}

macro_rules! impl_read_from_int {
    ($t:ty) => {
        impl ReadFrom for $t {
            fn read_from<R: Read>(mut r: R) -> Result<(Self, usize)> {
                let mut bytes = [0u8; size_of::<$t>()];
                let size = r.read(&mut bytes)?;
                Ok((<$t>::from_le_bytes(bytes), size))
            }
        }
    }
}

impl_read_from_int!(u8);
impl_read_from_int!(u16);
impl_read_from_int!(u32);
impl_read_from_int!(u64);




pub trait WriteTo: Sized {
    fn write_to<W: Write>(self, w: &mut W) -> Result<usize>;

    fn bytes(self) -> Result<Box<[u8]>> {
        let mut buf = Vec::new();
        self.write_to(&mut buf)?;
        Ok(buf.into_boxed_slice())
    }
}

pub trait WriteAs {
    fn write_as<T: WriteTo>(self, value: T) -> Result<usize>;
}

// Automatic implementation of WriteAs for any Write.
impl <W: Write> WriteAs for W {
    fn write_as<T: WriteTo>(mut self, value: T) -> Result<usize> {
        value.write_to(&mut self)
    }
}

macro_rules! impl_write_as_int {
    ($t:ty) => {
        impl WriteTo for $t {
            fn write_to<W: Write>(self, w: &mut W) -> Result<usize> {
                w.write(&self.to_le_bytes())
            }
        }
    }
}

impl_write_as_int!(u8);
impl_write_as_int!(u16);
impl_write_as_int!(u32);
impl_write_as_int!(u64);
impl_write_as_int!(u128);
impl_write_as_int!(c_int);

