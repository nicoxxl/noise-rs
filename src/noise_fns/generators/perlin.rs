use crate::{
    math::{self, interpolate},
    noise_fns::{NoiseFn, Seedable},
    permutationtable::PermutationTable,
};

/// Noise function that outputs 2/3/4-dimensional Perlin noise.
#[derive(Clone, Copy, Debug)]
pub struct Perlin {
    seed: u32,
    perm_table: PermutationTable,
}

impl Perlin {
    pub const DEFAULT_SEED: u32 = 0;

    pub fn new() -> Self {
        Self {
            seed: Self::DEFAULT_SEED,
            perm_table: PermutationTable::new(Self::DEFAULT_SEED),
        }
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}

impl Seedable for Perlin {
    /// Sets the seed value for Perlin noise
    fn set_seed(self, seed: u32) -> Self {
        // If the new seed is the same as the current seed, just return self.
        if self.seed == seed {
            return self;
        }

        // Otherwise, regenerate the permutation table based on the new seed.
        Self {
            seed,
            perm_table: PermutationTable::new(seed),
        }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}

/// 2-dimensional perlin noise
impl NoiseFn<[f64; 2]> for Perlin {
    fn get(&self, point: [f64; 2]) -> f64 {
        perlin_2d(self.perm_table, point[0], point[1])
    }
}

#[inline(always)]
pub(crate) fn perlin_2d(perm_table: PermutationTable, x: f64, y: f64) -> f64 {
    #[inline(always)]
    #[rustfmt::skip]
    fn gradient_dot_v(perm: usize, x: f64, y: f64) -> f64 {
        match perm & 0b11 {
            0 =>  x + y,  // ( 1,  1)
            1 => -x + y, // (-1,  1)
            2 =>  x - y,  // ( 1, -1)
            3 => -x - y, // (-1, -1)
            _ => unreachable!(),
        }
    }

    let point = [x, y];

    // Unscaled range of linearly interpolated perlin noise should be (-sqrt(N/4), sqrt(N/4)).
    // Need to invert this value and multiply the unscaled result by the value to get a scaled
    // range of (-1, 1).
    let scale_factor = (2.0_f64).sqrt(); // 1/sqrt(N/4), N=2 -> 1/sqrt(1/2) -> sqrt(2)

    let floored = math::map2(point, f64::floor);
    let near_corner = math::to_isize2(floored);
    let far_corner = math::add2(near_corner, [1; 2]);
    let near_distance = math::sub2(point, floored);
    let far_distance = math::sub2(near_distance, [1.0; 2]);

    let g00 = gradient_dot_v(
        perm_table.get2(near_corner),
        near_distance[0],
        near_distance[1],
    );
    let g10 = gradient_dot_v(
        perm_table.get2([far_corner[0], near_corner[1]]),
        far_distance[0],
        near_distance[1],
    );
    let g01 = gradient_dot_v(
        perm_table.get2([near_corner[0], far_corner[1]]),
        near_distance[0],
        far_distance[1],
    );
    let g11 = gradient_dot_v(
        perm_table.get2(far_corner),
        far_distance[0],
        far_distance[1],
    );

    let u = interpolate::s_curve5(near_distance[0]);
    let v = interpolate::s_curve5(near_distance[1]);

    let unscaled_result = bilinear_interpolation(u, v, g00, g01, g10, g11);

    let scaled_result = unscaled_result * scale_factor;

    // At this point, we should be really damn close to the (-1, 1) range, but some float errors
    // could have accumulated, so let's just clamp the results to (-1, 1) to cut off any
    // outliers and return it.

    math::clamp(scaled_result, -1.0, 1.0)
}

#[inline(always)]
fn bilinear_interpolation(u: f64, v: f64, g00: f64, g01: f64, g10: f64, g11: f64) -> f64 {
    let k0 = g00;
    let k1 = g10 - g00;
    let k2 = g01 - g00;
    let k3 = g00 + g11 - g10 - g01;

    k0 + k1 * u + k2 * v + k3 * u * v
}

/// 3-dimensional perlin noise
impl NoiseFn<[f64; 3]> for Perlin {
    fn get(&self, point: [f64; 3]) -> f64 {
        perlin_3d(self.perm_table, point[0], point[1], point[2])
    }
}

#[inline(always)]
#[allow(clippy::many_single_char_names)]
pub(crate) fn perlin_3d(perm_table: PermutationTable, x: f64, y: f64, z: f64) -> f64 {
    #[inline(always)]
    #[rustfmt::skip]
    fn gradient_dot_v(perm: usize, point: [f64; 3]) -> f64 {
        let x = point[0];
        let y = point[1];
        let z = point[2];

        match perm & 0b1111 {
            0 =>  x + y, // ( 1,  1,  0)
            1 => -x + y, // (-1,  1,  0)
            2 =>  x - y, // ( 1, -1,  0)
            3 => -x - y, // (-1, -1,  0)
            4 =>  x + z, // ( 1,  0,  1)
            5 => -x + z, // (-1,  0,  1)
            6 =>  x - z, // ( 1,  0, -1)
            7 => -x - z, // (-1,  0, -1)
            8 =>  y + z, // ( 0,  1,  1)
            9 => -y + z, // ( 0, -1,  1)
            10 =>  y - z, // ( 0,  1, -1)
            11 => -y - z, // ( 0, -1, -1)
            12 =>  x + y, // ( 1,  1,  0)
            13 => -x + y, // (-1,  1,  0)
            14 => -y + z, // ( 0, -1,  1)
            15 => -y - z, // ( 0, -1, -1)
            _ => unreachable!(),
        }
    }

    let point = [x, y, z];

    // Unscaled range of linearly interpolated perlin noise should be (-sqrt(N/4), sqrt(N/4)).
    // Need to invert this value and multiply the unscaled result by the value to get a scaled
    // range of (-1, 1).
    let scale_factor = 2.0_f64 / ((3.0_f64).sqrt()); // 1/sqrt(N/4), N=3 -> 1/sqrt(3/4) -> 2/sqrt(3)

    let floored = math::map3(point, f64::floor);
    let near_corner = math::to_isize3(floored);
    let far_corner = math::add3(near_corner, [1; 3]);
    let near_distance = math::sub3(point, floored);
    let far_distance = math::sub3(near_distance, [1.0; 3]);

    let g000 = gradient_dot_v(perm_table.get3(near_corner), near_distance);
    let g100 = gradient_dot_v(
        perm_table.get3([far_corner[0], near_corner[1], near_corner[2]]),
        [far_distance[0], near_distance[1], near_distance[2]],
    );
    let g010 = gradient_dot_v(
        perm_table.get3([near_corner[0], far_corner[1], near_corner[2]]),
        [near_distance[0], far_distance[1], near_distance[2]],
    );
    let g110 = gradient_dot_v(
        perm_table.get3([far_corner[0], far_corner[1], near_corner[2]]),
        [far_distance[0], far_distance[1], near_distance[2]],
    );
    let g001 = gradient_dot_v(
        perm_table.get3([near_corner[0], near_corner[1], far_corner[2]]),
        [near_distance[0], near_distance[1], far_distance[2]],
    );
    let g101 = gradient_dot_v(
        perm_table.get3([far_corner[0], near_corner[1], far_corner[2]]),
        [far_distance[0], near_distance[1], far_distance[2]],
    );
    let g011 = gradient_dot_v(
        perm_table.get3([near_corner[0], far_corner[1], far_corner[2]]),
        [near_distance[0], far_distance[1], far_distance[2]],
    );
    let g111 = gradient_dot_v(perm_table.get3(far_corner), far_distance);

    let a = interpolate::s_curve5(near_distance[0]);
    let b = interpolate::s_curve5(near_distance[1]);
    let c = interpolate::s_curve5(near_distance[2]);

    let k0 = g000;
    let k1 = g100 - g000;
    let k2 = g010 - g000;
    let k3 = g001 - g000;
    let k4 = g000 + g110 - g100 - g010;
    let k5 = g000 + g101 - g100 - g001;
    let k6 = g000 + g011 - g010 - g001;
    let k7 = g100 + g010 + g001 + g111 - g000 - g110 - g101 - g011;

    let unscaled_result =
        k0 + k1 * a + k2 * b + k3 * c + k4 * a * b + k5 * a * c + k6 * b * c + k7 * a * b * c;

    let scaled_result = unscaled_result * scale_factor;

    // At this point, we should be really damn close to the (-1, 1) range, but some float errors
    // could have accumulated, so let's just clamp the results to (-1, 1) to cut off any
    // outliers and return it.

    math::clamp(scaled_result, -1.0, 1.0)
}

/// 4-dimensional perlin noise
impl NoiseFn<[f64; 4]> for Perlin {
    fn get(&self, point: [f64; 4]) -> f64 {
        perlin_4d(self.perm_table, point[0], point[1], point[2], point[3])
    }
}

#[inline(always)]
#[rustfmt::skip]
#[allow(clippy::many_single_char_names)]
pub(crate) fn perlin_4d(perm_table: PermutationTable, x: f64, y: f64, z: f64, w: f64) -> f64 {
    #[inline(always)]
    fn gradient_dot_v(perm: usize, x: f64, y: f64, z: f64, w: f64) -> f64 {
        match perm & 0b11111 {
            0 =>   x + y + z    , // ( 1,  1,  1,  0)
            1 =>  -x + y + z    , // (-1,  1,  1,  0)
            2 =>   x - y + z    , // ( 1, -1,  1,  0)
            3 =>   x + y - z    , // ( 1,  1, -1,  0)
            4 =>  -x + y - z    , // (-1,  1, -1,  0)
            5 =>   x - y - z    , // ( 1, -1, -1,  0)
            6 =>   x - y - z    , // (-1, -1, -1,  0)
            7 =>   x + y     + w, // ( 1,  1,  0,  1)
            8 =>  -x + y     + w, // (-1,  1,  0,  1)
            9 =>   x - y     + w, // ( 1, -1,  0,  1)
            10 =>  x + y     - w, // ( 1,  1,  0, -1)
            11 =>  x + y     - w, // (-1,  1,  0, -1)
            12 =>  x + y     - w, // ( 1, -1,  0, -1)
            13 => -x - y     - w, // (-1, -1,  0, -1)
            14 =>  x     + z + w, // ( 1,  0,  1,  1)
            15 => -x     + z + w, // (-1,  0,  1,  1)
            16 =>  x     - z + w, // ( 1,  0, -1,  1)
            17 =>  x     + z - w, // ( 1,  0,  1, -1)
            18 =>  x     + z - w, // (-1,  0,  1, -1)
            19 =>  x     + z - w, // ( 1,  0, -1, -1)
            20 => -x     - z - w, // (-1,  0, -1, -1)
            21 =>      y + z + w, // ( 0,  1,  1,  1)
            22 =>     -y + z + w, // ( 0, -1,  1,  1)
            23 =>      y - z + w, // ( 0,  1, -1,  1)
            24 =>      y - z - w, // ( 0,  1,  1, -1)
            25 =>     -y - z - w, // ( 0, -1,  1, -1)
            26 =>  x + y + z - w, // ( 0,  1, -1, -1)
            27 => -x + y + z - w, // ( 0, -1, -1, -1)
            28 =>  x + y + z    , // ( 1,  1,  1,  0)
            29 =>  x + y     + w, // ( 1,  1,  0,  1)
            30 =>  x     + z + w, // ( 1,  0,  1,  1)
            31 =>      y + z + w, // ( 0,  1,  1,  1)
            _ => unreachable!(),
        }
    }

    let point = [x, y, z, w];

    // Unscaled range of linearly interpolated perlin noise should be (-sqrt(N/4), sqrt(N/4)).
    // Need to invert this value and multiply the unscaled result by the value to get a scaled
    // range of (-1, 1).
    let scale_factor = 2.0; // 1/sqrt(N/4), N=4 -> 1/sqrt(4/4) -> 2/sqrt(1)

    let floored = math::map4(point, f64::floor);
    let near_corner = math::to_isize4(floored);
    let far_corner = math::add4(near_corner, [1; 4]);
    let near_distance = math::sub4(point, floored);
    let far_distance = math::sub4(near_distance, [1.0; 4]);

    let g0000 = gradient_dot_v(
        perm_table.get4(near_corner),
        near_distance[0],
        near_distance[1],
        near_distance[2],
        near_distance[3],
    );
    let g1000 = gradient_dot_v(
        perm_table.get4([
            far_corner[0],
            near_corner[1],
            near_corner[2],
            near_corner[3],
        ]),
        far_distance[0],
        near_distance[1],
        near_distance[2],
        near_distance[3],
    );
    let g0100 = gradient_dot_v(
        perm_table.get4([
            near_corner[0],
            far_corner[1],
            near_corner[2],
            near_corner[3],
        ]),
        near_distance[0],
        far_distance[1],
        near_distance[2],
        near_distance[3],
    );
    let g1100 = gradient_dot_v(
        perm_table.get4([
            far_corner[0],
            far_corner[1],
            near_corner[2],
            near_corner[3]
        ]),
        far_distance[0],
        far_distance[1],
        near_distance[2],
        near_distance[3],
    );
    let g0010 = gradient_dot_v(
        perm_table.get4([
            near_corner[0],
            near_corner[1],
            far_corner[2],
            near_corner[3],
        ]),
        near_distance[0],
        near_distance[1],
        far_distance[2],
        near_distance[3],
    );
    let g1010 = gradient_dot_v(
        perm_table.get4([
            far_corner[0],
            near_corner[1],
            far_corner[2],
            near_corner[3]
        ]),
        far_distance[0],
        near_distance[1],
        far_distance[2],
        near_distance[3],
    );
    let g0110 = gradient_dot_v(
        perm_table.get4([
            near_corner[0],
            far_corner[1],
            far_corner[2],
            near_corner[3]
        ]),
        near_distance[0],
        far_distance[1],
        far_distance[2],
        near_distance[3],
    );
    let g1110 = gradient_dot_v(
        perm_table.get4([
            far_corner[0],
            far_corner[1],
            far_corner[2],
            near_corner[3]
        ]),
        far_distance[0],
        far_distance[1],
        far_distance[2],
        near_distance[3],
    );
    let g0001 = gradient_dot_v(
        perm_table.get4([
            near_corner[0],
            near_corner[1],
            near_corner[2],
            far_corner[3],
        ]),
        near_distance[0],
        near_distance[1],
        near_distance[2],
        far_distance[3],
    );
    let g1001 = gradient_dot_v(
        perm_table.get4([
            far_corner[0],
            near_corner[1],
            near_corner[2],
            far_corner[3]
        ]),
        far_distance[0],
        near_distance[1],
        near_distance[2],
        far_distance[3],
    );
    let g0101 = gradient_dot_v(
        perm_table.get4([
            near_corner[0],
            far_corner[1],
            near_corner[2],
            far_corner[3]
        ]),
        near_distance[0],
        far_distance[1],
        near_distance[2],
        far_distance[3],
    );
    let g1101 = gradient_dot_v(
        perm_table.get4([
            far_corner[0],
            far_corner[1],
            near_corner[2],
            far_corner[3]
        ]),
        far_distance[0],
        far_distance[1],
        near_distance[2],
        far_distance[3],
    );
    let g0011 = gradient_dot_v(
        perm_table.get4([
            near_corner[0],
            near_corner[1],
            far_corner[2],
            far_corner[3]
        ]),
        near_distance[0],
        near_distance[1],
        far_distance[2],
        far_distance[3],
    );
    let g1011 = gradient_dot_v(
        perm_table.get4([
            far_corner[0],
            near_corner[1],
            far_corner[2],
            far_corner[3]
        ]),
        far_distance[0],
        near_distance[1],
        far_distance[2],
        far_distance[3],
    );
    let g0111 = gradient_dot_v(
        perm_table.get4([
            near_corner[0],
            far_corner[1],
            far_corner[2],
            far_corner[3]
        ]),
        near_distance[0],
        far_distance[1],
        far_distance[2],
        far_distance[3],
    );
    let g1111 = gradient_dot_v(
        perm_table.get4(far_corner),
        far_distance[0],
        far_distance[1],
        far_distance[2],
        far_distance[3],
    );

    let a = interpolate::s_curve5(near_distance[0]);
    let b = interpolate::s_curve5(near_distance[1]);
    let c = interpolate::s_curve5(near_distance[2]);
    let d = interpolate::s_curve5(near_distance[3]);

    let k0 = g0000;
    let k1 = g1000 - g0000;
    let k2 = g0100 - g0000;
    let k3 = g0010 - g0000;
    let k4 = g0001 - g0000;
    let k5 = g0000 + g1100 - g1000 - g0100;
    let k6 = g0000 + g1010 - g1000 - g0010;
    let k7 = g0000 + g1001 - g1000 - g0001;
    let k8 = g0000 + g0110 - g0100 - g0010;
    let k9 = g0000 + g0101 - g0100 - g0001;
    let k10 = g0000 + g0011 - g0010 - g0001;
    let k11 = g1110 + g1000 + g0100 + g0010 - g0000 - g0111 - g1011 - g1101;
    let k12 = g1101 + g1000 + g0100 + g0001 - g0000 - g0111 - g1011 - g1110;
    let k13 = g1011 + g1000 + g0010 + g0001 - g0000 - g0111 - g1101 - g1110;
    let k14 = g0111 + g0100 + g0010 + g0001 - g0000 - g1011 - g1101 - g1110;
    let k15 = g1111 + g1000 + g0100 + g0010 + g0001 - g0000 - g0111 - g1011 - g1101 - g1110;

    let unscaled_result = k0
        + k1 * a
        + k2 * b
        + k3 * c
        + k4 * d
        + k5 * a * b
        + k6 * a * c
        + k7 * a * d
        + k8 * b * c
        + k9 * b * d
        + k10 * c * d
        + k11 * a * b * c
        + k12 * a * b * d
        + k13 * a * c * d
        + k14 * b * c * d
        + k15 * a * b * c * d;


    let scaled_result = unscaled_result * scale_factor;

    // At this point, we should be really damn close to the (-1, 1) range, but some float errors
    // could have accumulated, so let's just clamp the results to (-1, 1) to cut off any
    // outliers and return it.

    math::clamp(scaled_result, -1.0, 1.0)
}
