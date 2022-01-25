use colstodian::{Color, LinearSrgb, Scene};
use pigment_mixing::Pigment;

fn main() {
    // Define three colors as pigments
    let bright_yellow_pigment = Pigment::from_srgb_u8(252, 211, 0);
    let deep_blue_pigment = Pigment::from_srgb_u8(0, 0, 96);
    let medium_red = Pigment::from_srgb_u8(201, 37, 44);

    // Weight each one ⅓rd.
    let weight: f32 = 1.0 / 3.0;

    // Calculate the result.
    let result = bright_yellow_pigment * weight
        + weight * deep_blue_pigment
        + weight * medium_red;

    // Conbert the pigment back to an sRGB color.
    let linear_srgb_result: Color<LinearSrgb, Scene> = result.into();

    println!("{:?}", linear_srgb_result);
}
