// /// Raw transmutation to `u32`.
// ///
// /// Transmutes the given `f32` into it's raw memory representation.
// /// Similar to `f32::to_bits` but even more raw.
// #[inline]
// fn to_bits(x: f32) -> u32 {
//     unsafe { ::std::mem::transmute::<f32, u32>(x) }
// }

// /// Raw transmutation from `u32`.
// ///
// /// Converts the given `u32` containing the float's raw memory representation into the `f32` type.
// /// Similar to `f32::from_bits` but even more raw.
// #[inline]
// fn from_bits(x: u32) -> f32 {
//     unsafe { ::std::mem::transmute::<u32, f32>(x) }
// }

// /// Raises 2 to a floating point power.
// #[inline]
// pub fn pow2(p: f32) -> f32 {
//     let clipp = if p < -126.0 { -126.0_f32 } else { p };
//     let v = ((1 << 23) as f32 * (clipp + 126.94269504_f32)) as u32;
//     from_bits(v)
// }

#[inline]
pub fn pitch_to_f(p: f32, sample_rate: f32) -> f32 {
    // pow2((p-49.0)/12.0) * 440.0 / sample_rate
    (2.0_f32).powf((p-49.0)/12.0) * 440.0 / sample_rate
}