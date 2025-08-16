use core::simd::prelude::*;

use super::fallback;

#[must_use]
#[inline]
pub fn contains_null_or_utf8_4_byte_char_header(value: &[u8]) -> bool {
    let mut remainder = value;

    macro_rules! process {
        ($simd:ty) => {
            let (array_chunks, array_remainder) = remainder.as_chunks::<{ <$simd>::LEN }>();
            remainder = array_remainder;

            let zero = <$simd>::splat(0x00);
            let mask = <$simd>::splat(0b1111_1000);
            let header = <$simd>::splat(0b1111_0000);

            for &array in array_chunks {
                let simd = <$simd>::from_array(array);

                if simd.simd_eq(zero).any() || (simd & mask).simd_eq(header).any() {
                    return true;
                }
            }
        };
    }

    process!(u8x64);
    process!(u8x32);
    process!(u8x16);
    process!(u8x8);
    process!(u8x4);
    process!(u8x2);

    fallback::contains_null_or_utf8_4_byte_char_header(remainder)
}

#[must_use]
#[inline]
pub fn contains_utf8_4_byte_char_header(value: &[u8]) -> bool {
    let mut remainder = value;

    macro_rules! process {
        ($simd:ty) => {
            let (array_chunks, array_remainder) = remainder.as_chunks::<{ <$simd>::LEN }>();
            remainder = array_remainder;

            let mask = <$simd>::splat(0b1111_1000);
            let header = <$simd>::splat(0b1111_0000);

            for &array in array_chunks {
                let simd = <$simd>::from_array(array);

                if (simd & mask).simd_eq(header).any() {
                    return true;
                }
            }
        };
    }

    process!(u8x64);
    process!(u8x32);
    process!(u8x16);
    process!(u8x8);
    process!(u8x4);
    process!(u8x2);

    fallback::contains_utf8_4_byte_char_header(remainder)
}
