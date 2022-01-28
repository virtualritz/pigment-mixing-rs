use crate::clamp;
use core::ops::Add;
use nanorand::Rng as WyRandRng;
pub use nanorand::WyRand;
use num_traits::Float;

pub type Rng = WyRand;

/// Generates a random number in the range -0.5 .. 0.5.
#[inline]
fn generate_random_number(rng: &mut Rng) -> f32 {
    rng.generate_range(u32::MIN / 2..=u32::MAX / 2) as f32 / u32::MAX as f32
}

#[inline]
pub fn quantize_triplet<T>(
    value: (T, T, T),
    one: T,
    min: T,
    max: T,
    rng: &mut Rng,
) -> (T, T, T)
where
    T: Float + Add<f32, Output = T>,
{
    let random = generate_random_number(rng);
    (
        clamp((one * value.0 + random).round(), min, max),
        clamp((one * value.1 + random).round(), min, max),
        clamp((one * value.2 + random).round(), min, max),
    )
}
