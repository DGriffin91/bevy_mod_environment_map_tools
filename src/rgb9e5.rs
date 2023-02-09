#[allow(dead_code)]
const RGB9E5_EXPONENT_BITS: i32 = 5;
const RGB9E5_MANTISSA_BITS: i32 = 9;
const RGB9E5_EXP_BIAS: i32 = 15;
const RGB9E5_MAX_VALID_BIASED_EXP: i32 = 31;

const MAX_RGB9E5_EXP: i32 = RGB9E5_MAX_VALID_BIASED_EXP - RGB9E5_EXP_BIAS;
const RGB9E5_MANTISSA_VALUES: i32 = 1 << RGB9E5_MANTISSA_BITS;
const MAX_RGB9E5_MANTISSA: i32 = RGB9E5_MANTISSA_VALUES - 1;
#[allow(dead_code)]
const MAX_RGB9E5: f32 =
    (MAX_RGB9E5_MANTISSA as f32) / RGB9E5_MANTISSA_VALUES as f32 * (1 << MAX_RGB9E5_EXP) as f32;
#[allow(dead_code)]
const EPSILON_RGB9E5: f32 = (1.0 / RGB9E5_MANTISSA_VALUES as f32) / (1 << RGB9E5_EXP_BIAS) as f32;

// https://www.khronos.org/registry/OpenGL/extensions/EXT/EXT_texture_shared_exponent.txt
#[inline]
pub fn float3_to_rgb9e5(rgb: &[f32]) -> u32 {
    let rc = rgb[0].clamp(0.0, MAX_RGB9E5);
    let gc = rgb[1].clamp(0.0, MAX_RGB9E5);
    let bc = rgb[2].clamp(0.0, MAX_RGB9E5);

    let maxrgb = rc.max(gc).max(bc);
    let mut exp_shared =
        (-RGB9E5_EXP_BIAS - 1).max(maxrgb.log2().floor() as i32) + 1 + RGB9E5_EXP_BIAS;

    // assert!(exp_shared <= RGB9E5_MAX_VALID_BIASED_EXP);
    // assert!(exp_shared >= 0);

    let mut denom = ((exp_shared - RGB9E5_EXP_BIAS - RGB9E5_MANTISSA_BITS) as f32).exp2();

    let maxm = (maxrgb / denom + 0.5).floor() as i32;
    if maxm == MAX_RGB9E5_MANTISSA + 1 {
        denom *= 2.0;
        exp_shared += 1;
        // assert!(exp_shared <= RGB9E5_MAX_VALID_BIASED_EXP);
    } else {
        // assert!(maxm <= MAX_RGB9E5_MANTISSA);
    }

    let rm = (rc / denom + 0.5).floor() as i32;
    let gm = (gc / denom + 0.5).floor() as i32;
    let bm = (bc / denom + 0.5).floor() as i32;

    // assert!(rm <= MAX_RGB9E5_MANTISSA);
    // assert!(gm <= MAX_RGB9E5_MANTISSA);
    // assert!(bm <= MAX_RGB9E5_MANTISSA);
    // assert!(rm >= 0);
    // assert!(gm >= 0);
    // assert!(bm >= 0);

    let rm = rm as u32;
    let gm = gm as u32;
    let bm = bm as u32;
    let exp_shared = exp_shared as u32;

    #[allow(clippy::identity_op)]
    let ret = (exp_shared << 27) | (bm << 18) | (gm << 9) | (rm << 0);

    ret
}

#[inline]
fn bitfield_extract(value: u32, offset: u32, bits: u32) -> u32 {
    let mask = (1u32 << bits) - 1u32;
    (value >> offset) & mask
}

#[inline]
pub fn rgb9e5_to_float3(v: u32) -> [f32; 3] {
    let exponent = bitfield_extract(v, 27, RGB9E5_EXPONENT_BITS as u32) as i32
        - RGB9E5_EXP_BIAS
        - RGB9E5_MANTISSA_BITS;
    let scale = (exponent as f32).exp2();

    [
        bitfield_extract(v, 0, RGB9E5_MANTISSA_BITS as u32) as f32 * scale,
        bitfield_extract(v, 9, RGB9E5_MANTISSA_BITS as u32) as f32 * scale,
        bitfield_extract(v, 18, RGB9E5_MANTISSA_BITS as u32) as f32 * scale,
    ]
}
