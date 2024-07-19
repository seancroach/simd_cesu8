pub mod fallback;
pub mod simd;
pub mod word;

#[cfg(feature = "nightly")]
pub use self::simd as active;
#[cfg(not(feature = "nightly"))]
pub use self::word as active;
