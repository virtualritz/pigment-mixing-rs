# `pigment-mixing` – High level Rust wrapper around [Mixbox](https://scrtwpns.com/mixbox)

<p align="center">
  <img src="https://scrtwpns.com/mixbox/teaser.jpg"/>
</p>

This crate uses the `mixbox-sys` wrapper around the [Mixbox C++ reference
implementation](https://github.com/scrtwpns/pigment-mixing).
`Mixbox` treats colors as if they were made of actual real-world pigments.
It uses the Kubelka & Munk theory to predict the color of the resulting mixture.

See [the website of SeCReT WeaPoNS](https://scrtwpns.com/mixbox) for more
information on the ideas behind this.
## Usage

The simplest way to use this crate is through one of the `mix_` prefix
functions:

```rs
use pigment_mixing::mix_srgb_u8;

// Define some colors in encoded sRGB (gamma 2.2).
let bright_yellow = [252, 211, 0];
let deep_blue = [0, 0, 96];

// 50/50 mix.
let mixing_ratio = 0.5;

// The colors are linearized internally but the
// returned result is converted back to encoded
// sRGB.
let pale_green =
    mix_srgb_u8(
        &bright_yellow,
        &deep_blue,
        mixing_ratio
    );
```

Alternatively, you can use the `Pigment` type. This allows mixing multiple colors at once using arbitrary weights:

```rs
use colstodian::{Color, Scene, LinearSrgb};
use pigment_mixing::Pigment;

// Define three colors as pigments
let bright_yellow_pigment = Pigment::from_srgb_u8(252, 211, 0);
let deep_blue_pigment = Pigment::from_srgb_u8(0, 0, 96);
let medium_red = Pigment::from_srgb_u8(201, 37, 44);

// Weight each one ⅓rd.
let weight: f32 = 1.0 / 3.0;

// Calculate the result.
let result = bright_yellow_pigment * weight + weight * deep_blue_pigment + weight * medium_red;

// Convert the pigment back to an sRGB color.
let linear_srgb_result: Color<LinearSrgb, Scene> = result.into();
```

## Notes on Color

The original paper mentions only `sRGB` as the working space. This makes sense
if we assume a *linear* `sRGB` working space.

The reference implementation in C++ does not seem to treat `f32` component
RGB tuples any different from `u8` component RGB tuples though.

From looking at the C++ code my current conclusion is that this is an error as
`u8` component `sRGB` will in almost all cases be *display-referred* with an
encoded *gamma of 2.2*.

Using such values in any color math without linearizing them first (removing
the gamma) leads to wrong results. Which is very obvious when doing e.g. mixing
in RGB. I.e. fringes when mixing semi-opaque pixels – more visible when their
color is close to primaries like red and green).

I have a
[ticket open with the authors](https://github.com/scrtwpns/pigment-mixing/issues/1)
to clarify this.

The code in this crate only uses the `f32` component functions from the C++ code
internally.
The `u8` component convenience functions assume encoded sRGB and do decoding
(linearization) befor mixing and encoding before returning through the excellent
[`colstodian`](https://github.com/termhn/colstodian) crate.

The code in this crate only uses the `f32` component functions from the C++ code
internally.

The `u8` component convenience functions assume *encoded* `sRGB` and do decoding
(linearization) befor mixing and encoding before returning.

All color conversion is done via the excellent
[`colstodian`](https://github.com/termhn/colstodian) crate.

## License

The underlying implementation is:

> Copyright © 2021, Secret Weapons. All rights reserved.
> This code is for non-commercial use only. It is provided for research and
> evaluation purposes.
> If you wish to obtain commercial license, please contact:
> mixbox@scrtwpns.com

The Rust wrappers, both the low level one, `mixbox-sys`, and this one are
licensed under either of

* [Apache, version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
* [BSD 3-Clause](https://opensource.org/licenses/BSD-3-Clause)
* [MIT](http://opensource.org/licenses/MIT)
* [Zlib](https://opensource.org/licenses/Zlib)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the `Apache-2.0` license, shall
be licensed as above, without any additional terms or conditions.
