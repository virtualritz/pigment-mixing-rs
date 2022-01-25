use pigment_mixing::mix_srgb_u8;
//use mixbox-sys::

fn main() {
    let bright_yellow = [252, 211, 0];
    let deep_blue = [0, 0, 96];

    let mixing_ratio = 0.5;

    let pale_green = mix_srgb_u8(&bright_yellow, &deep_blue, mixing_ratio);

    println!(
        "Mixing {:?} with {:?} gives {:?}",
        bright_yellow, deep_blue, pale_green
    );
}
