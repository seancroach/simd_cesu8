# `simd_cesu8`

An extremely fast encoding and decoding library
for [CESU-8](https://en.wikipedia.org/wiki/UTF-8#CESU-8) and [Modified
UTF-8](https://en.wikipedia.org/wiki/UTF-8#Modified_UTF-8).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies.simd_cesu8]
version = "1"
features = ["nightly"]
```

## Features

- `std` enables `std::error::Error` support for `DecodeError`. It also enables
  the `std` feature for `simdutf8`, which
  enables runtime architecture detection.
- `nightly` enables nightly features, such as `portable_simd`, which is crucial
  for some of the optimizations in this
  library. However, without this feature, a best-attempt is made by using
  word-at-a-time operations over byte-at-a-time.

## Performance

It is highly encouraged to use the `nightly` feature. With it enabled, on the
7950X3D I have access to, this library can test the validity of 

The performance of this library is significantly faster when it's possible to
borrow from the input. This is because, when possible, this library will use
[`Cow::Borrowed`](https://doc.rust-lang.org/nightly/alloc/borrow/enum.Cow.html#variant.Borrowed).

This library speculatively allocates memory when required, to avoid
reallocations or undoubtedly over-allocations. If the amount of memory is a
concern, consider
calling [`Vec::shrink_to_fit`](https://doc.rust-lang.org/nightly/alloc/vec/struct.Vec.html#method.shrink_to_fit)
or [`String::shrink_to_fit`](https://doc.rust-lang.org/nightly/alloc/string/struct.String.html#method.shrink_to_fit)
on any owned data returned by functions in this library.