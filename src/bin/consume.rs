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

fn main() {}
