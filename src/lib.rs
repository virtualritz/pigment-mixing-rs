//!
//! ```rust
//! # use pigment_mixing::mix_srgb_u8;
//! let bright_yellow = [252, 211, 0];
//! let deep_blue = [0, 0, 96];
//!
//! let mixing_ratio = 0.5;
//!
//! let result = mix_srgb_u8(
//!     &bright_yellow,
//!     &deep_blue,
//!     mixing_ratio
//! );
//!
//! println!("Green: {:?}", result);
//! println!("Foobar");
//! ```
//!
use colstodian::{kolor::Vec3, Color, Display, EncodedSrgb, LinearSrgb, Scene};
use core::{u16, u8};
use mixbox_sys::mixbox_lerp_srgb32f;
use num_traits::cast::AsPrimitive;

#[cfg(feature = "pigment")]
mod pigment;
#[cfg(feature = "pigment")]
pub use pigment::*;

mod quantize;
pub use quantize::*;

/// Mixes two sRGB colors.
pub fn mix_linear_srgb<T, St>(
    srgb_a: &Color<LinearSrgb, St>,
    srgb_b: &Color<LinearSrgb, St>,
    ratio: T,
) -> Color<LinearSrgb, Display>
where
    T: AsPrimitive<f32>,
{
    let mut color = std::mem::MaybeUninit::<Vec3>::uninit();
    let color_ptr = color.as_mut_ptr().cast::<f32>();

    Color::from_raw(unsafe {
        mixbox_lerp_srgb32f(
            srgb_a.raw[0],
            srgb_a.raw[1],
            srgb_a.raw[2],
            srgb_b.raw[0],
            srgb_b.raw[1],
            srgb_b.raw[2],
            ratio.as_(),
            color_ptr as _,
            color_ptr.offset(1) as _,
            color_ptr.offset(2) as _,
        );
        color.assume_init()
    })
}

/// Mixes two `u8` component sRGB colors.
///
/// The colors are assumed to be in sRGB with an encoded gamma of 2.2.
///
/// Colors will be linearized internally before mixing.
///
/// The output is in sRGB with an encoded gamma of 2.2.
#[inline]
pub fn mix_srgb_u8<T>(srgb_a: &[u8; 3], srgb_b: &[u8; 3], ratio: T) -> [u8; 3]
where
    T: AsPrimitive<f32>,
    f32: From<T>,
{
    let a_linear = Color::<EncodedSrgb, Scene>::new(
        srgb_a[0] as f32 / u8::MAX as f32,
        srgb_a[1] as f32 / u8::MAX as f32,
        srgb_a[2] as f32 / u8::MAX as f32,
    )
    .linearize();
    let b_linear = Color::<EncodedSrgb, Scene>::new(
        srgb_b[0] as f32 / u8::MAX as f32,
        srgb_b[1] as f32 / u8::MAX as f32,
        srgb_b[2] as f32 / u8::MAX as f32,
    )
    .linearize();

    let result = mix_linear_srgb(&a_linear, &b_linear, ratio).convert_to::<EncodedSrgb>();

    [
        (result.raw[0] * u8::MAX as f32 + 0.5) as _,
        (result.raw[1] * u8::MAX as f32 + 0.5) as _,
        (result.raw[2] * u8::MAX as f32 + 0.5) as _,
    ]
}

/// Mixes two `u8` component sRGB and dithers the result.
///
/// The colors are assumed to be in sRGB with an encoded gamma of 2.2.
///
/// Colors will be linearized internally before mixing.
///
/// The output is quantizes the with an error diffusion dither with an amplitude of 0.5
/// is in sRGB with an encoded gamma of 2.2.
#[inline]
pub fn mix_srgb_u8_dither<T>(srgb_a: &[u8; 3], srgb_b: &[u8; 3], ratio: T, rng: &mut Rng) -> [u8; 3]
where
    T: AsPrimitive<f32>,
    f32: From<T>,
{
    let a_linear = Color::<EncodedSrgb, Scene>::new(
        srgb_a[0] as f32 / u8::MAX as f32,
        srgb_a[1] as f32 / u8::MAX as f32,
        srgb_a[2] as f32 / u8::MAX as f32,
    )
    .linearize();
    let b_linear = Color::<EncodedSrgb, Scene>::new(
        srgb_b[0] as f32 / u8::MAX as f32,
        srgb_b[1] as f32 / u8::MAX as f32,
        srgb_b[2] as f32 / u8::MAX as f32,
    )
    .linearize();

    let result = mix_linear_srgb(&a_linear, &b_linear, ratio).convert_to::<EncodedSrgb>();

    // QUantize the result to u8 using an error diffusion dither
    // with an amplitude of 0.5 to avoid artifacts.
    let (r, g, b) = quantize_triplet(
        (result.raw[0], result.raw[1], result.raw[2]),
        u8::MAX as _, // one
        0.0,          // min
        u8::MAX as _, // max
        rng,
    );

    [r as _, g as _, b as _]
}

/// Mixes two `u16` component sRGB colors.
///
/// The colors are assumed to be in linear sRGB (gamma 1.0).
///
/// The output is in sRGB with an encoded gamma of 2.2.
#[inline]
pub fn mix_linear_srgb_u16<T>(srgb_a: &[u16; 3], srgb_b: &[u16; 3], ratio: T) -> [u16; 3]
where
    T: AsPrimitive<f32>,
    f32: From<T>,
{
    let a_linear = Color::<LinearSrgb, Scene>::new(
        srgb_a[0] as f32 / u16::MAX as f32,
        srgb_a[1] as f32 / u16::MAX as f32,
        srgb_a[2] as f32 / u16::MAX as f32,
    );
    let b_linear = Color::<LinearSrgb, Scene>::new(
        srgb_b[0] as f32 / u16::MAX as f32,
        srgb_b[1] as f32 / u16::MAX as f32,
        srgb_b[2] as f32 / u16::MAX as f32,
    );

    let result = mix_linear_srgb(&a_linear, &b_linear, ratio);

    [
        (result.raw[0] * u16::MAX as f32 + 0.5) as _,
        (result.raw[1] * u16::MAX as f32 + 0.5) as _,
        (result.raw[2] * u16::MAX as f32 + 0.5) as _,
    ]
}

/// Mixes two `u8` component sRGB and dithers the result.
///
/// The colors are assumed to be in linear sRGB (gamma 1.0).
///
/// Colors will be linearized internally before mixing.
///
/// The output is quantizes the with an error diffusion dither with an amplitude of 0.5
/// is in sRGB with an encoded gamma of 2.2.
#[inline]
pub fn mix_srgb_u16_dither<T>(
    srgb_a: &[u8; 3],
    srgb_b: &[u8; 3],
    ratio: T,
    rng: &mut Rng,
) -> [u8; 3]
where
    T: AsPrimitive<f32>,
    f32: From<T>,
{
    let a_linear = Color::<LinearSrgb, Scene>::new(
        srgb_a[0] as f32 / u16::MAX as f32,
        srgb_a[1] as f32 / u16::MAX as f32,
        srgb_a[2] as f32 / u16::MAX as f32,
    );
    let b_linear = Color::<LinearSrgb, Scene>::new(
        srgb_b[0] as f32 / u16::MAX as f32,
        srgb_b[1] as f32 / u16::MAX as f32,
        srgb_b[2] as f32 / u16::MAX as f32,
    );

    let result = mix_linear_srgb(&a_linear, &b_linear, ratio);

    // Quantize the result to u16 using an error diffusion dither
    // with an amplitude of 0.5 to avoid artifacts.
    let (r, g, b) = quantize_triplet(
        (result.raw[0], result.raw[1], result.raw[2]),
        u16::MAX as _, // one
        0.0,           // min
        u16::MAX as _, // max
        rng,
    );

    [r as _, g as _, b as _]
}

#[inline]
fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}
