use alloc::vec::Vec;
use alloc::String;
use error::{Error};
use cbor_no_std::{from_bytes, to_bytes, Value};
use core::mem;
use core::slice;

pub const LENGTH_BYTE_COUNT: usize = 4;

pub type Pointer = *const u8;

fn u32_to_u8_vec(x:u32, big_endian: bool) -> Vec<u8> {
    let b1 : u8 = ((x >> 24) & 0xff) as u8;
    let b2 : u8 = ((x >> 16) & 0xff) as u8;
    let b3 : u8 = ((x >> 8) & 0xff) as u8;
    let b4 : u8 = (x & 0xff) as u8;

    if big_endian {
        vec![b1, b2, b3, b4]
    } else {
        vec![b4, b3, b2, b1]
    }
}

fn u64_to_u8_vec(x: u64, big_endian: bool) -> Vec<u8> {
    let b1 : u8 = ((x >> 56) & 0xff) as u8;
    let b2 : u8 = ((x >> 48) & 0xff) as u8;
    let b3 : u8 = ((x >> 40) & 0xff) as u8;
    let b4 : u8 = ((x >> 32) & 0xff) as u8;
    let b5 : u8 = ((x >> 24) & 0xff) as u8;
    let b6 : u8 = ((x >> 16) & 0xff) as u8;
    let b7 : u8 = ((x >> 8) & 0xff) as u8;
    let b8 : u8 = (x & 0xff) as u8;

    if big_endian {
        vec![b1, b2, b3, b4, b5, b6, b7, b8]
    } else {
        vec![b8, b7, b6, b5, b4, b3, b2, b1]
    }
}

fn u8_vec_to_u32(x: Vec<u8>, big_endian: bool) -> u32 {
    if big_endian {
        (x[0] as u32) << 24 |
            ((x[1] as u32) & 0xff) << 16 |
            ((x[2] as u32) & 0xff) << 8 |
            ((x[3] as u32) & 0xff)
    } else {
        (x[3] as u32) << 24 |
            ((x[2] as u32) & 0xff) << 16 |
            ((x[1] as u32) & 0xff) << 8 |
            ((x[0] as u32) & 0xff)
    }
}

fn u8_vec_to_u64(x: Vec<u8>, big_endian: bool) -> u64 {
    if big_endian {
        (x[7] as u64) << 56 |
            ((x[6] as u64) & 0xff) << 48 |
            ((x[5] as u64) & 0xff) << 40 |
            ((x[4] as u64) & 0xff) << 32 |
            ((x[3] as u64) & 0xff) << 24 |
            ((x[2] as u64) & 0xff) << 16 |
            ((x[1] as u64) & 0xff) << 8 |
            ((x[0] as u64) & 0xff)
    } else {
        (x[0] as u64) << 56 |
            ((x[1] as u64) & 0xff) << 48 |
            ((x[2] as u64) & 0xff) << 40 |
            ((x[3] as u64) & 0xff) << 32 |
            ((x[4] as u64) & 0xff) << 24 |
            ((x[5] as u64) & 0xff) << 16 |
            ((x[6] as u64) & 0xff) << 8 |
            ((x[7] as u64) & 0xff)
    }
}

pub unsafe trait Dereferenceable {
    fn as_raw_bytes(&self) -> Vec<u8>;
    fn to_bytes(&self) -> Vec<u8>;
    fn to_string(&self) -> String;
    fn to_array(&self) -> Vec<Value>;
}

unsafe impl Dereferenceable for Pointer {
    fn as_raw_bytes(&self) -> Vec<u8> {
        let length_slice = unsafe { slice::from_raw_parts(self.offset(0) as *const u8, LENGTH_BYTE_COUNT as usize) };
        let length = u8_vec_to_u32(length_slice.to_vec(), true);

        unsafe {
            slice::from_raw_parts(self.offset(LENGTH_BYTE_COUNT as isize) as *const u8, length as usize).to_vec()
        }
    }

    fn to_bytes(&self) -> Vec<u8> {
        from_bytes(self.as_raw_bytes()).as_bytes().unwrap().to_vec()
    }

    fn to_string(&self) -> String {
        let name = from_bytes(self.as_raw_bytes());
        name.as_string().unwrap().clone()
    }

    fn to_array(&self) -> Vec<Value> {
        let name = from_bytes(self.as_raw_bytes());
        name.as_array().unwrap().clone()
    }
}

pub unsafe trait Referenceable {
    fn as_pointer(&self) -> Pointer;
}

unsafe impl Referenceable for Vec<u8> {
    fn as_pointer(&self) -> Pointer {
        let mut value = self.clone();
        let mut value_and_length = u32_to_u8_vec(value.len() as u32, true);
        value_and_length.append(&mut value);
        let value_and_length_ptr = value_and_length.as_ptr();
        mem::forget(value_and_length);
        value_and_length_ptr
    }
}

unsafe impl Referenceable for String {
    fn as_pointer(&self) -> Pointer {
        self.as_bytes().to_vec().as_pointer()
    }
}

pub trait ReturnValue {
    fn to_return_value(self) -> Pointer;
}


impl ReturnValue for Result<Value, Error> {
    fn to_return_value(self: Result<Value, Error>) -> Pointer {
        let error_code = match self {
            Err(error) => error.code,
            _ => 0,
        };

        let mut return_value = match self {
            Ok(Value::Null) => vec![],
            Ok(value) => to_bytes(value),
            Err(error) => to_bytes(Value::String(error.message.into())),
        };

        let mut error_code_bytes = u32_to_u8_vec(error_code, true);

        error_code_bytes.append(&mut return_value);
        error_code_bytes.as_pointer()
    }
}

impl ReturnValue for Result<(), Error> {
    fn to_return_value(self) -> Pointer {
        match self {
            Ok(value) => (Ok::<Value, Error>(value.into())).to_return_value(),
            Err(error) => (Err::<Value, Error>(error)).to_return_value()
        }
    }
}

impl ReturnValue for Result<u32, Error> {
    fn to_return_value(self) -> Pointer {
        match self {
            Ok(value) => (Ok::<Value, Error>(value.into())).to_return_value(),
            Err(error) => (Err::<Value, Error>(error)).to_return_value()
        }
    }
}

impl ReturnValue for Result<Vec<u8>, Error> {
    fn to_return_value(self) -> Pointer {
        match self {
            Ok(value) => (Ok::<Value, Error>(value.into())).to_return_value(),
            Err(error) => (Err::<Value, Error>(error)).to_return_value()
        }
    }
}

pub trait FromBytes {}

impl FromBytes {
    pub fn from_u64(value: u64) -> Vec<u8>{
        u64_to_u8_vec(value, true)
    }
}

pub trait Bytes<T> {
    fn value(&self) -> T;
}

impl Bytes<u64> for Vec<u8> {
    fn value(&self) -> u64 {
        if self.len() == 8 {
            let mut slice: [u8; 8] = [0; 8];
            slice.copy_from_slice(&self[..]);
            u8_vec_to_u64(slice.to_vec(), false)
        } else {
            0
        }
    }
}

impl Bytes<u32> for Vec<u8> {
    fn value(&self) -> u32 {
        let mut slice: [u8; 4] = [0; 4];
        slice.copy_from_slice(&self[..]);
        u8_vec_to_u32(slice.to_vec(), true)
    }
}
