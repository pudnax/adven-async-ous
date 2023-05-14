#![allow(unused_macros, unused_imports)]
use std::convert::TryInto;

macro_rules! consume_long {
    ($ptr:expr, $ty:ty) => {{
        const TYSZ: usize = core::mem::size_of::<$ty>();

        let mut tmp = [0u8; TYSZ];

        $ptr.get(0..TYSZ).map(|x| {
            tmp.copy_from_slice(x);

            $ptr = &$ptr[TYSZ..];

            Some(<$ty>::from_le_bytes(tmp))
                .ok_or_else(|| format!("Failed to consume {} bytes, {} remain", TYSZ, $ptr.len()))
        })
    }};
}

macro_rules! consume_short {
    ($ptr:expr, $ty:ty) => {{
        const TYSZ: usize = core::mem::size_of::<$ty>();

        $ptr[..TYSZ]
            .try_into()
            .map(|x| {
                $ptr = &$ptr[TYSZ..];
                let ret = <$ty>::from_le_bytes(x);
                ret
            })
            .map_err(|_| format!("Failed to consume {} bytes, {} remain", TYSZ, $ptr.len()))
    }};
}

macro_rules! consume_reader {
    ($reader:expr, $ty:ty) => {{
        use std::io::Read;

        const TYSZ: usize = core::mem::size_of::<$ty>();
        let mut bytes = [0u8; TYSZ];

        $reader
            .read_exact(&mut bytes)
            .map(|_| <$ty>::from_le_bytes(bytes))
    }};
}
            
#[macro_export]
macro_rules! consume2 {
    ($buf:expr, $ty:ty) => {{
        // Slice up the buffer to the size we need
        $buf.get(..std::mem::size_of::<$ty>()).map(|x| {
            // Get the value
            let val = <$ty>::from_le_bytes(x.try_into().unwrap());

            // Advance the buffer
            $buf = &$buf[std::mem::size_of::<$ty>()..];

            // Return the value!
            val
        })
    }}
}
            
            /// Consume values which implement `from_le_bytes` from a buffer, advancing
/// the buffer beyond the bytes that were consumed
#[macro_export]
macro_rules! consumee {
    ($buf:expr, $ty:ty) => {{
        consume!($buf, $ty,).map(|x| x.0)
    }};

    ($buf:expr, $($ty:ty),*$(,)?) => {{
        /// Total size we need to consume for all values combined
        const TOTAL_SIZE: usize = $(
            core::mem::size_of::<$ty>() +
        )* 0;

        // Slice up the buffer to the size we need
        $buf.get(..TOTAL_SIZE).map(|mut _x| {
            // Advance the buffer
            $buf = &$buf[TOTAL_SIZE..];

            // Return the values!
            ($(
                {
                    // Get the value
                    let val = <$ty>::from_le_bytes(
                        _x[..core::mem::size_of::<$ty>()].try_into().unwrap());

                    // Advance pointer
                    _x = &_x[core::mem::size_of::<$ty>()..];

                    // Return value
                    val
                },
            )*)
        })
    }};
}

pub fn safe_transmute(mut data: &[u8]) -> Option<(u32, u32)> {
    consume!(data, u8, u8).map(|(a, b)| (a as _, b as _))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_consume_raw() {
        let ptr: &[u8] = &[231u8, 3, 0, 0, 231u8, 3, 0, 0];
        const SIZE: usize = std::mem::size_of::<u32>();

        let val = u32::from_le_bytes(ptr[..SIZE].try_into().unwrap());
        let val2 = u32::from_le_bytes(ptr[..SIZE].try_into().unwrap());

        assert_eq!(999, val);
        assert_eq!(999, val2);
    }

    #[test]
    fn test_consume_long() {
        let mut ptr: &[u8] = &[231u8, 3, 0, 0, 5, 6, 7];

        let val = consume_long!(ptr, u32).unwrap().unwrap();

        assert_eq!(999, val);
    }

    #[test]
    fn test_consume_short() {
        let mut ptr: &[u8] = &[231u8, 3, 0, 0, 5, 6, 7];

        let val = consume_short!(ptr, u32).unwrap();

        assert_eq!(999, val);
    }

    #[test]
    fn test_consume_reader() {
        let ptr: &[u8] = &[231u8, 3, 0, 0, 5, 6, 7];
        let mut reader = std::io::BufReader::new(ptr);

        let val = consume_reader!(reader, u32).unwrap();
        assert_eq!(999, val);
    }
}
            
/*
#![feature(split_array)]

/// Consume a value which implements `from_le_bytes` from a buffer, advancing
/// the buffer beyond the bytes that were consumed
#[macro_export]
macro_rules! consume {
    ($buf:expr, $ty:ty) => {{
        const SIZE: usize = std::mem::size_of::<$ty>();

        // check that we have enough bytes to extract a $ty
        // + 1 instead of >= because >= confuses llvm so it 
        // refuses to fuse multiple checks with multiple consume! calls
        if $buf.len() + 1 > SIZE {
            // split into &[u8; SIZE] and &[u8]
            let (x, rest) = $buf.split_array_ref::<SIZE>();

            // get the val
            let val = <$ty>::from_le_bytes(*x);

            // advance the buffer
            $buf = rest;
            Some(val)
        } else {
            None
        }
    }}
}

pub fn parse(mut ptr: &[u8]) -> Option<(u32, u64, u32, u32)> {
    Some((
        consume!(ptr, u32)?,
        consume!(ptr, u64)?,
        consume!(ptr, u32)?,
        consume!(ptr, u32)?,
    ))
}
*/

fn main() {}
