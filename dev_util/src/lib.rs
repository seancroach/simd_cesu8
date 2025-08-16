//! A set of utilities for generating data for benchmarks.

// NOTE: The assertions in this `crate` are more like debug assertions, but
// because they are used in benchmarks, typical debug assertions would never
// trigger.
#![allow(clippy::missing_panics_doc)]
#![feature(iter_intersperse)]

extern crate alloc;

use alloc::borrow::Cow;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::iter;

use criterion::Throughput;
use rand::distributions::Uniform;
use rand::prelude::*;
use rand_xoshiro::Xoshiro256StarStar;

/// A bucket of data that can be used in benchmarks.
#[derive(Debug, Clone)]
pub struct Bucket<T> {
    name: Cow<'static, str>,
    values: Vec<T>,
}

impl<T> Bucket<T> {
    const SIZE: usize = 1000;

    /// The size of the values in the bucket in bytes. This number was chosen
    /// because it was the closest number to 16 KiB that was divisible by 3. We
    /// want to use a number that is divisible by 3 because we want to generate
    /// UTF-8 strings that contain only characters of a certain width, and one
    /// of those widths is 3 bytes.
    pub const VALUE_SIZE: usize = 16_380;

    /// The throughput of the bucket in bytes.
    pub const THROUGHPUT: Throughput = Throughput::Bytes(Self::VALUE_SIZE as u64);

    /// Returns the name of the bucket.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Creates a new bucket of data.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            values: &self.values,
            index: 0,
        }
    }
}

impl Bucket<String> {
    #[must_use]
    fn new_string<T>(name: T, values: Vec<String>) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        let name = name.into();

        assert!(
            name.ends_with("_strings"),
            "{name:?} does not end with \"_strings\""
        );

        assert_eq!(values.len(), Self::SIZE, "{name:?} has an incorrect size");

        for value in &values {
            assert_eq!(
                value.len(),
                Self::VALUE_SIZE,
                "{name:?} has an element of incorrect size"
            );
        }

        eprintln!("initialized {name:?} bucket");

        Self { name, values }
    }

    /// Generates a bucket of strings purely composed of null characters.
    #[must_use]
    pub fn null() -> Self {
        let value = "\0".repeat(Self::VALUE_SIZE);
        let values = iter::repeat_n(value, Self::SIZE).collect::<Vec<_>>();
        Self::new_string("null_strings", values)
    }

    /// Generates a bucket of random ASCII strings that don't contain the null
    /// byte.
    #[must_use]
    pub fn ascii_non_null() -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(1);
        let ascii_dist = Uniform::new_inclusive(0x01, 0x7f);

        let values = (0..Self::SIZE)
            .map(|_| {
                let bytes = (&mut rng)
                    .sample_iter(&ascii_dist)
                    .take(Self::VALUE_SIZE)
                    .collect::<Vec<u8>>();
                String::from_utf8(bytes).unwrap()
            })
            .collect::<Vec<_>>();

        Self::new_string("ascii_non_null_strings", values)
    }

    /// Generates a bucket of ASCII strings where every other byte is a null
    /// byte.
    #[must_use]
    pub fn ascii_null_alternating() -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(1);
        let ascii_dist = Uniform::new_inclusive(0x01, 0x7f);

        let values = (0..Self::SIZE)
            .map(|_| {
                let bytes = (&mut rng)
                    .sample_iter(&ascii_dist)
                    .intersperse(0x00)
                    .take(Self::VALUE_SIZE)
                    .collect::<Vec<u8>>();
                String::from_utf8(bytes).unwrap()
            })
            .collect::<Vec<_>>();

        Self::new_string("ascii_null_alternating_strings", values)
    }

    /// Generates a bucket of UTF-8 strings that only contain characters that
    /// are of a certain width.
    #[must_use]
    pub fn utf8_clamped_width(width: usize) -> Self {
        assert!(width != 0 && width <= 4);
        let mut rng = Xoshiro256StarStar::seed_from_u64(1);

        let values = (0..Self::SIZE)
            .map(|_| {
                Utf8ClampedGen::new(&mut rng, width)
                    .take(Self::VALUE_SIZE / width)
                    .collect::<String>()
            })
            .collect::<Vec<_>>();

        Self::new_string(format!("utf8_clamped_width_{width}_strings"), values)
    }

    /// Generates a bucket of UTF-8 strings that contain a uniform spread of
    /// characters of different widths.
    ///
    /// The distribution of characters is as follows:
    ///
    /// 1. A non-null ASCII character (1 byte)
    /// 2. A 2-byte character
    /// 3. A 3-byte character
    /// 4. A 4-byte character
    /// 5. A null byte
    /// 6. A 2-byte character
    /// 7. A 3-byte character
    /// 8. A 4-byte character
    ///
    /// This pattern repeats until the string is filled ([`Self::VALUE_SIZE`]).
    #[must_use]
    pub fn interspersed() -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(1);
        let ascii_dist = Uniform::new_inclusive(0x01, 0x7f);

        let passes = Self::VALUE_SIZE / (4 + 3 + 2 + 1);

        let mut values = Vec::with_capacity(Self::SIZE);

        for _ in 0..Self::SIZE {
            let clamped_1 = (0..passes)
                .map(|i| {
                    if i % 2 == 0 {
                        '\0'
                    } else {
                        let n: u8 = rng.sample(ascii_dist);
                        char::from(n)
                    }
                })
                .collect::<String>();
            let clamped_2 = Utf8ClampedGen::new(&mut rng, 2)
                .take(passes)
                .collect::<String>();
            let clamped_3 = Utf8ClampedGen::new(&mut rng, 3)
                .take(passes)
                .collect::<String>();
            let clamped_4 = Utf8ClampedGen::new(&mut rng, 4)
                .take(passes)
                .collect::<String>();

            let mut clamped_1_chars = clamped_1.chars();
            let mut clamped_2_chars = clamped_2.chars();
            let mut clamped_3_chars = clamped_3.chars();
            let mut clamped_4_chars = clamped_4.chars();

            let mut string = String::with_capacity(Self::VALUE_SIZE);

            for _ in 0..passes {
                string.push(clamped_1_chars.next().unwrap());
                string.push(clamped_2_chars.next().unwrap());
                string.push(clamped_3_chars.next().unwrap());
                string.push(clamped_4_chars.next().unwrap());
            }

            assert_eq!(string.len(), Self::VALUE_SIZE);
            values.push(string);
        }

        Self::new_string("interspersed_strings", values)
    }

    /// Converts the bucket of strings into a bucket of bytes.
    #[must_use]
    pub fn into_bytes(self) -> Bucket<Vec<u8>> {
        let values = self.values.into_iter().map(String::into_bytes).collect();
        let name = self.name.into_owned().replace("_strings", "_bytes");
        Bucket::new_bytes(name, values)
    }
}

impl Bucket<Vec<u8>> {
    #[must_use]
    fn new_bytes<T>(name: T, values: Vec<Vec<u8>>) -> Self
    where
        T: Into<Cow<'static, str>>,
    {
        let name = name.into();

        assert!(
            name.ends_with("_bytes"),
            "{name:?} does not end with \"_bytes\""
        );
        assert_eq!(values.len(), Self::SIZE, "{name:?} has an incorrect size");

        for value in &values {
            assert_eq!(
                value.len(),
                Self::VALUE_SIZE,
                "{name:?} has an element of incorrect size"
            );
        }

        eprintln!("initialized {name:?} bucket");

        Self { name, values }
    }

    /// Generates a bucket of bytes that are random UTF-8 characters with a
    /// width of 4 bytes encoded in CESU-8 as surrogate pairs.
    #[must_use]
    pub fn surrogate_pairs() -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(1);
        let mut gen = Utf8ClampedGen::new(&mut rng, 4);

        let passes = Self::VALUE_SIZE / 6;

        let values = (0..Self::SIZE)
            .map(|_| {
                let input = gen.take(passes).collect::<String>();
                let encoded = cesu8::to_cesu8(&input);
                encoded.into_owned()
            })
            .collect::<Vec<_>>();

        Self::new_bytes("surrogate_pairs_bytes", values)
    }

    /// A bucket of bytes that contain only the null byte encoded in MUTF-8.
    #[must_use]
    pub fn mutf8_null_bytes() -> Self {
        let value = [0xc0, 0x80].repeat(Self::VALUE_SIZE / 2);
        let values = iter::repeat_n(value, Self::SIZE).collect::<Vec<_>>();
        Self::new_bytes("mutf8_null_bytes", values)
    }

    /// Generates a bucket of CESU-8 bytes that contain a uniform spread of
    /// characters of different widths.
    ///
    /// The distribution of characters is as follows:
    ///
    /// 1. An ASCII character (1 byte)
    /// 2. A 2-byte character
    /// 3. A 3-byte character
    /// 4. A 4-byte character encoded in CESU-8 (6 bytes)
    ///
    /// This pattern repeats until the string is filled ([`Self::VALUE_SIZE`]).
    #[must_use]
    pub fn interspersed_cesu8() -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(1);
        let ascii_dist = Uniform::new_inclusive(0x01, 0x7f);

        let passes = Self::VALUE_SIZE / (6 + 3 + 2 + 1);

        let mut values = Vec::with_capacity(Self::SIZE);

        for _ in 0..Self::SIZE {
            let mut vec = Vec::with_capacity(Self::VALUE_SIZE);

            let clamped_1 = (0..passes)
                .map(|_| {
                    let n: u8 = rng.sample(ascii_dist);
                    char::from(n)
                })
                .collect::<String>();
            let clamped_2 = Utf8ClampedGen::new(&mut rng, 2)
                .take(passes)
                .collect::<String>();
            let clamped_3 = Utf8ClampedGen::new(&mut rng, 3)
                .take(passes)
                .collect::<String>();
            let clamped_4 = Utf8ClampedGen::new(&mut rng, 4)
                .take(passes)
                .collect::<String>();

            let mut clamped_1_chars = clamped_1.chars();
            let mut clamped_2_chars = clamped_2.chars();
            let mut clamped_3_chars = clamped_3.chars();
            let mut clamped_4_chars = clamped_4.chars();

            for _ in 0..passes {
                let mut input = String::with_capacity(1 + 2 + 3 + 4);

                input.push(clamped_1_chars.next().unwrap());
                input.push(clamped_2_chars.next().unwrap());
                input.push(clamped_3_chars.next().unwrap());
                input.push(clamped_4_chars.next().unwrap());

                let encoded = cesu8::to_cesu8(&input);
                vec.extend_from_slice(&encoded);
            }

            assert_eq!(vec.len(), Self::VALUE_SIZE);
            values.push(vec);
        }

        Self::new_bytes("interspersed_cesu8_bytes", values)
    }

    /// Generates a bucket of MUTF-8 bytes that contain a uniform spread of
    /// characters of different widths.
    ///
    /// The distribution of characters is as follows:
    ///
    /// 1. A non-null ASCII character (1 byte)
    /// 2. A non-null ASCII character (1 byte)
    /// 2. A 2-byte character
    /// 3. A 3-byte character
    /// 4. A 4-byte character encoded in CESU-8 (6 bytes)
    /// 5. A null byte encoded in MUTF-8 (2 bytes)
    /// 6. A 2-byte character
    /// 7. A 3-byte character
    /// 8. A 4-byte character encoded in CESU-8 (6 bytes)
    ///
    /// This pattern repeats until the string is filled ([`Self::VALUE_SIZE`]).
    #[must_use]
    pub fn interspersed_mutf8() -> Self {
        let mut rng = Xoshiro256StarStar::seed_from_u64(1);
        let ascii_dist = Uniform::new_inclusive(0x01, 0x7f);

        let passes = Self::VALUE_SIZE / (1 + 1 + 2 + 3 + 6 + 2 + 2 + 3 + 6);

        let mut values = Vec::with_capacity(Self::SIZE);

        for _ in 0..Self::SIZE {
            let mut vec = Vec::with_capacity(Self::VALUE_SIZE);

            let clamped_1 = (0..(passes * 2))
                .map(|_| {
                    let n: u8 = rng.sample(ascii_dist);
                    char::from(n)
                })
                .collect::<String>();
            let clamped_2 = Utf8ClampedGen::new(&mut rng, 2)
                .take(passes * 2)
                .collect::<String>();
            let clamped_3 = Utf8ClampedGen::new(&mut rng, 3)
                .take(passes * 2)
                .collect::<String>();
            let clamped_4 = Utf8ClampedGen::new(&mut rng, 4)
                .take(passes * 2)
                .collect::<String>();

            let mut null = iter::repeat('\0');
            let mut clamped_1_chars = clamped_1.chars();
            let mut clamped_2_chars = clamped_2.chars();
            let mut clamped_3_chars = clamped_3.chars();
            let mut clamped_4_chars = clamped_4.chars();

            for _ in 0..passes {
                let mut input = String::with_capacity(21);

                input.push(clamped_1_chars.next().unwrap());
                input.push(clamped_1_chars.next().unwrap());
                input.push(clamped_2_chars.next().unwrap());
                input.push(clamped_3_chars.next().unwrap());
                input.push(clamped_4_chars.next().unwrap());
                input.push(null.next().unwrap());
                input.push(clamped_2_chars.next().unwrap());
                input.push(clamped_3_chars.next().unwrap());
                input.push(clamped_4_chars.next().unwrap());

                let encoded = cesu8::to_java_cesu8(&input);
                vec.extend_from_slice(&encoded);
            }

            values.push(vec);
        }

        Self::new_bytes("interspersed_mutf8_bytes", values)
    }
}

impl<'a, T> IntoIterator for &'a Bucket<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator that repeatedly takes values from a slice, resetting the index
/// when it reaches the end of the slice.
#[derive(Debug, Clone)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Iter<'a, T> {
    values: &'a [T],
    index: usize,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.values.len() {
            self.index = 0;
        }

        // SAFETY: We know that `self.index` is always in bounds because we
        // reset it to 0 when it reaches the end of the slice.
        let value = unsafe { self.values.get_unchecked(self.index) };
        self.index += 1;

        Some(value)
    }
}

#[must_use = "iterators are lazy and do nothing unless consumed"]
struct Utf8ClampedGen<'a, R>
where
    R: Rng,
{
    rng: &'a mut R,
    width: usize,
}

impl<'a, R> Utf8ClampedGen<'a, R>
where
    R: Rng,
{
    fn new(rng: &'a mut R, width: usize) -> Self {
        Self { rng, width }
    }
}

impl<R> Iterator for &mut Utf8ClampedGen<'_, R>
where
    R: Rng,
{
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // NOTE: There is a world where, depending on the width, we
            // selectively call different distributions to generate the
            // characters. This would be much more efficient. However, this
            // isn't production code, so we're not going to do that.

            let random_char = self.rng.gen::<char>();
            let width = random_char.len_utf8();

            if width == self.width {
                return Some(random_char);
            }
        }
    }
}
