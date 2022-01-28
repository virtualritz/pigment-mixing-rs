#![no_std]
//! High level Rust wrapper around [Mixbox](https://scrtwpns.com/mixbox).
//!
//! <p align="center">
//!   <img src="https://scrtwpns.com/mixbox/teaser.jpg"/>
//! </p>
//!
//! This crate uses the `mixbox-sys` wrapper around the [Mixbox C++ reference
//! implementation](https://github.com/scrtwpns/pigment-mixing).
//!
//! Mixbox treats colors as if they were made of actual real-world pigments.
//! It uses the It uses the
//! [Kubelka-Munk theory](https://en.wikipedia.org/wiki/Kubelka-Munk_theory)
//! to predict the color of the resulting mixture.
//!
//! See [the website of SeCReT WeaPoNS](https://scrtwpns.com/mixbox) for more
//! information on the idead behind this.
//! ## Usage
//!
//! The simplest way to use this crate is through one of the `mix_` prefix
//! functions:
//!
//! ```
//! # use pigment_mixing::mix_srgb_u8;
//! // Define some colors in encoded sRGB (gamma 2.2).
//! let bright_yellow = [252, 211, 0];
//! let deep_blue = [0, 0, 96];
//!
//! // 50/50 mix.
//! let mixing_ratio = 0.5;
//!
//! // The colors are linearized internally but the returned result is
//! // converted back to encoded sRGB.
//! let pale_green = mix_srgb_u8(&bright_yellow, &deep_blue, mixing_ratio);
//! ```
//!
//! Alternatively, you can use the [`Pigment`] type. This allows mixing multiple
//! colors at once using arbitrary weights:
//!
//! ```
//! # use pigment_mixing::Pigment;
//! use colstodian::{Color, LinearSrgb, Scene};
//!
//! // Define three colors as pigments
//! let bright_yellow_pigment = Pigment::from_srgb_u8(252, 211, 0);
//! let medium_red_pigment = Pigment::from_srgb_u8(201, 37, 44);
//! let deep_blue_pigment = Pigment::from_srgb_u8(0, 0, 96);
//!
//! // Weight each one ⅓rd.
//! let weight: f32 = 1.0 / 3.0;
//!
//! // Calculate the result.
//! let result = weight * bright_yellow_pigment
//!     + weight * medium_red_pigment
//!     + weight * deep_blue_pigment;
//!
//! // Convert the pigment back to an sRGB color.
//! let linear_srgb_result: Color<LinearSrgb, Scene> = result.into();
//! ```
//!
//! ## Notes on Color
//!
//! The original paper mentions only `sRGB` as the working space. This makes
//! sense if we assume a *linear* `sRGB` working space.
//!
//! The reference implementation in C++ does not seem to treat `f32` component
//! RGB tuples any different from `u8` component RGB tuples though.
//!
//! From looking at the C++ code my current conclusion is that this is an error
//! as `u8` component `sRGB` will in almost all cases be *display-referred* with
//! an encoded *gamma of 2.2*.
//!
//! Using such values in any color math without linearizing them first (removing
//! the gamma) leads to wrong results. Which is very obvious when doing e.g.
//! mixing in RGB. I.e. fringes when mixing semi-opaque pixels – more visible
//! when their color is close to primaries like red and green).
//!
//! I have a
//! [ticket open with the authors](https://github.com/scrtwpns/pigment-mixing/issues/1)
//! to clarify this.
//!
//! The code in this crate only uses the `f32` component functions from the C++
//! code internally.
//!
//! The `u8` component convenience functions on the Rust side assume *encoded*
//! `sRGB` and do decoding (linearization) befor mixing and encoding before
//! returning.
//!
//! All color conversion is done via the excellent
//! [`colstodian`](https://github.com/termhn/colstodian) crate.
//!
//! ## License
//!
//! The underlying implementation is:
//!
//! > Copyright © 2021, Secret Weapons. All rights reserved.
//! > This code is for non-commercial use only. It is provided for research and
//! > evaluation purposes.
//! > If you wish to obtain commercial license, please contact:
//! > mixbox@scrtwpns.com
//!
//! The Rust wrappers, both the low level one, `mixbox-sys`, and this one are
//! licensed under either of
//!
//! * [Apache, version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
//! * [BSD 3-Clause](https://opensource.org/licenses/BSD-3-Clause)
//! * [MIT](http://opensource.org/licenses/MIT)
//! * [Zlib](https://opensource.org/licenses/Zlib)
//!
//! at your option.
//!
//! ## Contribution
//!
//! Unless you explicitly state otherwise, any contribution intentionally
//! submitted for inclusion in the work by you, as defined in the `Apache-2.0`
//! license, shall be licensed as above, without any additional terms or
//! conditions.
use colstodian::{kolor::Vec3, Color, Display, EncodedSrgb, LinearSrgb, Scene};
use core::mem::MaybeUninit;
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
    let mut color = MaybeUninit::<Vec3>::uninit();
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

    let result = mix_linear_srgb(&a_linear, &b_linear, ratio)
        .convert_to::<EncodedSrgb>();

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
/// The output is in sRGB with an encoded gamma of 2.2.
/// It is quantized from [`f32`] to [`u8`] using  an error diffusion dither with
/// an amplitude of 0.5.
#[inline]
pub fn mix_srgb_u8_dither<T>(
    srgb_a: &[u8; 3],
    srgb_b: &[u8; 3],
    ratio: T,
    rng: &mut Rng,
) -> [u8; 3]
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

    let result = mix_linear_srgb(&a_linear, &b_linear, ratio)
        .convert_to::<EncodedSrgb>();

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
pub fn mix_linear_srgb_u16<T>(
    srgb_a: &[u16; 3],
    srgb_b: &[u16; 3],
    ratio: T,
) -> [u16; 3]
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
/// The output is quantizes the with an error diffusion dither with an amplitude
/// of 0.5 is in sRGB with an encoded gamma of 2.2.
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
