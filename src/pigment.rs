use crate::{clamp, AsPrimitive};
use arrayvec::ArrayVec;
use colstodian::{Color, EncodedSrgb, Scene};
use core::ops::{Add, Mul};
use mixbox_sys::{mixbox_latent_to_srgb32f, mixbox_srgb32f_to_latent, MIXBOX_NUMLATENTS};
use num_traits::{
    float::Float,
    identities::{one, zero},
};

const PIGMENT_LEN: usize = MIXBOX_NUMLATENTS as _;

pub struct Pigment([f32; PIGMENT_LEN]);

impl Pigment {
    pub fn from_mix<T>(a: Pigment, b: Pigment, ratio: T) -> Self
    where
        T: Float,
        f32: Mul<T, Output = f32>,
    {
        let ratio = clamp(ratio, zero(), one());
        let result: ArrayVec<_, PIGMENT_LEN> =
            a.0.iter()
                .zip(b.0.iter())
                .map(|(&a, &b)| a * (one::<T>() - ratio) + b * ratio)
                .collect();

        unsafe { Self(result.into_inner_unchecked()) }
    }

    pub fn mix<T>(&mut self, b: Pigment, ratio: T)
    where
        T: Float,
        f32: Mul<T, Output = f32>,
    {
        let ratio = clamp(ratio, zero(), one());
        self.0
            .iter_mut()
            .zip(b.0.into_iter())
            .for_each(|(a, b)| *a = *a * (one::<T>() - ratio) + b * ratio);
    }

    pub fn from_srgb_u8(r: u8, g: u8, b: u8) -> Self {
        let srgb_linear = Color::<EncodedSrgb, Scene>::new(
            r as f32 / u8::MAX as f32,
            g as f32 / u8::MAX as f32,
            b as f32 / u8::MAX as f32,
        )
        .linearize();

        let mut pigment = std::mem::MaybeUninit::<[f32; PIGMENT_LEN]>::uninit();

        unsafe {
            mixbox_srgb32f_to_latent(
                srgb_linear.raw[0],
                srgb_linear.raw[1],
                srgb_linear.raw[2],
                pigment.as_mut_ptr() as _,
            );

            Self(pigment.assume_init())
        }
    }

    pub fn from_linear_srgb_u16(r: u16, g: u16, b: u16) -> Self {
        let mut pigment = std::mem::MaybeUninit::<[f32; PIGMENT_LEN]>::uninit();

        unsafe {
            mixbox_srgb32f_to_latent(
                r as f32 / u8::MAX as f32,
                g as f32 / u8::MAX as f32,
                b as f32 / u8::MAX as f32,
                pigment.as_mut_ptr() as _,
            );

            Self(pigment.assume_init())
        }
    }
}

impl Mul<f32> for Pigment {
    type Output = Pigment;

    fn mul(self, rhs: f32) -> Self {
        let result: ArrayVec<_, PIGMENT_LEN> = self.0.iter().map(|a| a * rhs).collect();
        unsafe { Self(result.into_inner_unchecked()) }
    }
}

impl Add for Pigment {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let result: ArrayVec<_, PIGMENT_LEN> = self
            .0
            .iter()
            .zip(rhs.0.iter())
            .map(|(a, b)| a + b)
            .collect();
        unsafe { Self(result.into_inner_unchecked()) }
    }
}

impl<T> FromIterator<T> for Pigment
where
    T: AsPrimitive<f32>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        let result: ArrayVec<_, PIGMENT_LEN> = iter
            .into_iter()
            .take(PIGMENT_LEN)
            .map(|i| i.as_())
            .collect();

        Pigment(result.into_inner().unwrap())
    }
}

impl From<[f32; PIGMENT_LEN]> for Pigment {
    #[inline]
    fn from(pigment: [f32; PIGMENT_LEN]) -> Self {
        Self(pigment)
    }
}

impl From<Pigment> for [f32; PIGMENT_LEN] {
    #[inline]
    fn from(pigment: Pigment) -> Self {
        pigment.0
    }
}

/// Convert a linear sRGB slice to a `Pigment`.
impl From<[f32; 3]> for Pigment {
    #[inline]
    fn from(srgb: [f32; 3]) -> Self {
        Pigment::from((srgb[0], srgb[1], srgb[2]))
    }
}

/// Convert a linear sRGB tuple to a `Pigment`.
impl From<(f32, f32, f32)> for Pigment {
    #[inline]
    fn from(srgb: (f32, f32, f32)) -> Self {
        let mut pigment = std::mem::MaybeUninit::<[f32; PIGMENT_LEN]>::uninit();

        unsafe {
            mixbox_srgb32f_to_latent(srgb.0, srgb.1, srgb.2, pigment.as_mut_ptr() as _);

            Self(pigment.assume_init())
        }
    }
}

/// Convert a `Pigment` to a linear sRGB slice.
impl From<Pigment> for [f32; 3] {
    #[inline]
    fn from(pigment: Pigment) -> Self {
        let mut srgb = std::mem::MaybeUninit::<[f32; 3]>::uninit();

        unsafe {
            mixbox_latent_to_srgb32f(
                &pigment.0 as *const _ as _,
                srgb.as_mut_ptr() as _,
                srgb.as_mut_ptr().offset(1) as _,
                srgb.as_mut_ptr().offset(2) as _,
            );

            srgb.assume_init()
        }
    }
}

/// Convert a `Pigment` to a linear sRGB tuple.
impl From<Pigment> for (f32, f32, f32) {
    #[inline]
    fn from(pigment: Pigment) -> Self {
        let srgb = std::mem::MaybeUninit::<(f32, f32, f32)>::uninit();

        unsafe {
            let mut srgb = srgb.assume_init();

            mixbox_latent_to_srgb32f(
                &pigment.0 as *const _ as _,
                &mut srgb.0 as _,
                &mut srgb.1 as _,
                &mut srgb.2 as _,
            );

            srgb
        }
    }
}
