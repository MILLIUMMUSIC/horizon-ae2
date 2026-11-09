// SPDX-License-Identifier: LGPL-3.0-or-later AND Apache-2.0 AND MIT
// QuadRotator, Fabric packed normals/colors and the pinned JOML scalar path.
// FabricMC copyright 2016-2019; JOML copyright 2015-2022. See LICENSES.
use super::ae2_grid::Direction;

// BEGIN SOURCE ROTATION TABLES
const QUATERNIONS: [[u32; 4]; 24] = [
    [0xbf3504f3, 0x00000000, 0x00000000, 0x3f3504f3],
    [0x3effffff, 0xbeffffff, 0xbeffffff, 0xbeffffff],
    [0x00000000, 0xbf3504f3, 0xbf3504f3, 0x00000000],
    [0xbeffffff, 0xbeffffff, 0xbeffffff, 0x3effffff],
    [0x00000000, 0xbf3504f3, 0x3f3504f3, 0x00000000],
    [0xbeffffff, 0xbeffffff, 0x3effffff, 0xbeffffff],
    [0xbf3504f3, 0x00000000, 0x00000000, 0xbf3504f3],
    [0x3effffff, 0xbeffffff, 0x3effffff, 0x3effffff],
    [0x00000000, 0x00000000, 0x00000000, 0x3f800000],
    [0x80000000, 0x00000000, 0xbf3504f3, 0xbf3504f3],
    [0x00000000, 0x00000000, 0xbf800000, 0x00000000],
    [0x00000000, 0x00000000, 0xbf3504f3, 0x3f3504f3],
    [0x00000000, 0xbf800000, 0x00000000, 0x00000000],
    [0x3f3504f3, 0xbf3504f3, 0x00000000, 0x00000000],
    [0x3f800000, 0x00000000, 0x00000000, 0x00000000],
    [0x3f3504f3, 0x3f3504f3, 0x00000000, 0x00000000],
    [0x00000000, 0xbf3504f3, 0x00000000, 0xbf3504f3],
    [0x3effffff, 0x3effffff, 0x3effffff, 0x3effffff],
    [0x3f3504f3, 0x00000000, 0x3f3504f3, 0x00000000],
    [0x3effffff, 0xbeffffff, 0x3effffff, 0xbeffffff],
    [0x00000000, 0xbf3504f3, 0x00000000, 0x3f3504f3],
    [0x3effffff, 0x3effffff, 0xbeffffff, 0xbeffffff],
    [0x3f3504f3, 0x00000000, 0xbf3504f3, 0x00000000],
    [0x3effffff, 0xbeffffff, 0xbeffffff, 0x3effffff],
];
const ROTATED_FACES: [[u8; 6]; 24] = [
    [3, 2, 0, 1, 4, 5],
    [5, 4, 0, 1, 3, 2],
    [2, 3, 0, 1, 5, 4],
    [4, 5, 0, 1, 2, 3],
    [3, 2, 1, 0, 5, 4],
    [4, 5, 1, 0, 3, 2],
    [2, 3, 1, 0, 4, 5],
    [5, 4, 1, 0, 2, 3],
    [0, 1, 2, 3, 4, 5],
    [5, 4, 2, 3, 0, 1],
    [1, 0, 2, 3, 5, 4],
    [4, 5, 2, 3, 1, 0],
    [0, 1, 3, 2, 5, 4],
    [5, 4, 3, 2, 1, 0],
    [1, 0, 3, 2, 4, 5],
    [4, 5, 3, 2, 0, 1],
    [0, 1, 4, 5, 3, 2],
    [2, 3, 4, 5, 0, 1],
    [1, 0, 4, 5, 2, 3],
    [3, 2, 4, 5, 1, 0],
    [0, 1, 5, 4, 2, 3],
    [3, 2, 5, 4, 0, 1],
    [1, 0, 5, 4, 3, 2],
    [2, 3, 5, 4, 1, 0],
];
const FACE_VERTICES: [[[usize; 3]; 4]; 6] = [
    [[4, 0, 3], [4, 0, 2], [5, 0, 2], [5, 0, 3]],
    [[4, 1, 2], [4, 1, 3], [5, 1, 3], [5, 1, 2]],
    [[5, 1, 2], [5, 0, 2], [4, 0, 2], [4, 1, 2]],
    [[4, 1, 3], [4, 0, 3], [5, 0, 3], [5, 1, 3]],
    [[4, 1, 2], [4, 0, 2], [4, 0, 3], [4, 1, 3]],
    [[5, 1, 3], [5, 0, 3], [5, 0, 2], [5, 1, 2]],
];
// END SOURCE ROTATION TABLES

/// Fabric's three header words plus four eight-word vertices. Colors in this
/// representation are ARGB. Header/lighting/shader words survive the transform.
/// The face-normal/geometry cache outside the packed words is not modeled here.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PackedQuad {
    pub header: [u32; 3],
    pub vertices: [[u32; 8]; 4],
    pub nominal: Direction,
    pub geometry_invalid: bool,
}
impl PackedQuad {
    pub fn cull_face(&self) -> Option<Direction> {
        Direction::ALL.get((self.header[0] & 7) as usize).copied()
    }
    pub fn has_normal(&self, vertex: usize) -> bool {
        self.header[0] & (1 << (6 + vertex)) != 0
    }
    pub fn to_vanilla(&self) -> [[u32; 8]; 4] {
        self.vertices.map(|mut vertex| {
            vertex[3] = to_vanilla_color(vertex[3]);
            vertex
        })
    }
    pub fn from_vanilla(&mut self, vertices: [[u32; 8]; 4]) {
        self.vertices = vertices.map(|mut vertex| {
            vertex[3] = from_vanilla_color(vertex[3]);
            vertex
        });
        self.geometry_invalid = true;
    }
}

fn to_vanilla_color(color: u32) -> u32 {
    if cfg!(target_endian = "big") {
        color.rotate_left(8)
    } else {
        (color & 0xff00_ff00) | ((color >> 16) & 255) | ((color & 255) << 16)
    }
}
fn from_vanilla_color(color: u32) -> u32 {
    if cfg!(target_endian = "big") {
        color.rotate_right(8)
    } else {
        to_vanilla_color(color)
    }
}
fn clamp(value: f32) -> f32 {
    // Mth.clamp uses comparisons; NaN survives until Java's int conversion.
    value.clamp(-1.0, 1.0)
}
pub fn pack_normal(value: [f32; 4]) -> u32 {
    value.into_iter().enumerate().fold(0, |packed, (i, value)| {
        packed | ((((clamp(value) * 127.0) as i32 as u32) & 255) << (i * 8))
    })
}
pub fn unpack_normal(packed: u32) -> [f32; 4] {
    std::array::from_fn(|i| f32::from((packed >> (i * 8)) as i8) / 127.0)
}

fn rotate(q: [f32; 4], [x, y, z]: [f32; 3]) -> [f32; 3] {
    let [qx, qy, qz, qw] = q;
    let (xx, yy, zz, ww) = (qx * qx, qy * qy, qz * qz, qw * qw);
    let (xy, xz, yz, xw, zw, yw) = (qx * qy, qx * qz, qy * qz, qx * qw, qz * qw, qy * qw);
    let k = 1.0 / (xx + yy + zz + ww);
    // JOML 1.10.5 Math.fma defaults to a*b+c, nested in this order.
    [
        ((xx - yy - zz + ww) * k) * x + ((2.0 * (xy - zw) * k) * y + (2.0 * (xz + yw) * k) * z),
        (2.0 * (xy + zw) * k) * x + (((yy - xx - zz + ww) * k) * y + (2.0 * (yz - xw) * k) * z),
        (2.0 * (xz - yw) * k) * x + ((2.0 * (yz + xw) * k) * y + ((zz - xx - yy + ww) * k) * z),
    ]
}

/// Original FaceBakery winding changes positions and UV only. It keeps scanning
/// after a match: the last coincident vertex supplies UVs for a degenerate face.
fn recalculate_winding(vertices: &mut [[u32; 8]; 4], face: Direction) {
    let original = *vertices;
    let mut bounds = [999.0f32, -999.0, 999.0, -999.0, 999.0, -999.0];
    let axes = [(4, 5), (0, 1), (2, 3)];
    for vertex in original {
        for (i, (min, max)) in axes.into_iter().enumerate() {
            let value = f32::from_bits(vertex[i]);
            if value < bounds[min] {
                bounds[min] = value;
            }
            if value > bounds[max] {
                bounds[max] = value;
            }
        }
    }
    for (i, indices) in FACE_VERTICES[face as usize].into_iter().enumerate() {
        let point = indices.map(|index| bounds[index]);
        for j in 0..3 {
            vertices[i][j] = point[j].to_bits();
        }
        for old in original {
            if (0..3).all(|axis| (point[axis] - f32::from_bits(old[axis])).abs() < 0.00001) {
                vertices[i][4] = old[4];
                vertices[i][5] = old[5];
            }
        }
    }
}

/// BlockOrientation.get indexes facing*4+spin without normalizing spin. Refuse
/// the same out-of-range indices before touching the quad. Valid reporting
/// parts normally supply spin 0..3; saved signed bytes may cross enum rows.
pub fn transform(quad: &mut PackedQuad, facing: Direction, spin: i32) -> Result<(), &'static str> {
    let index = (facing as i32).wrapping_mul(4).wrapping_add(spin);
    let index = usize::try_from(index)
        .ok()
        .filter(|index| *index < 24)
        .ok_or("orientation index out of range")?;
    if index == 8 {
        return Ok(());
    } // NORTH_UP's NULL_TRANSFORM is a true no-op.
    let q = QUATERNIONS[index].map(f32::from_bits);
    for i in 0..4 {
        let p = std::array::from_fn(|axis| f32::from_bits(quad.vertices[i][axis]) - 0.5);
        let p = rotate(q, p).map(|n| n + 0.5);
        for (axis, value) in p.into_iter().enumerate() {
            quad.vertices[i][axis] = value.to_bits();
        }
        quad.geometry_invalid = true;
        if quad.has_normal(i) {
            let [x, y, z, _] = unpack_normal(quad.vertices[i][7]);
            let [x, y, z] = rotate(q, [x, y, z]);
            quad.vertices[i][7] = pack_normal([x, y, z, 0.0]);
        }
    }
    if let Some(cull) = quad.cull_face() {
        quad.header[0] = (quad.header[0] & !7) | u32::from(ROTATED_FACES[index][cull as usize]);
    }
    quad.nominal = Direction::ALL[ROTATED_FACES[index][quad.nominal as usize] as usize];
    let mut vanilla = quad.to_vanilla();
    recalculate_winding(&mut vanilla, quad.nominal);
    quad.from_vanilla(vanilla);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> PackedQuad {
        PackedQuad {
            header: [0xa1b2_c006, 0x00ab_cdef, 0xdead_beef],
            vertices: std::array::from_fn(|i| {
                let [x, y, z] = FACE_VERTICES[Direction::North as usize][i]
                    .map(|n| if n % 2 == 0 { 0.0f32 } else { 1.0 });
                [
                    x.to_bits(),
                    y.to_bits(),
                    z.to_bits(),
                    0xff00_0000 + i as u32,
                    (i as f32).to_bits(),
                    (-(i as f32)).to_bits(),
                    0x1234_0000 + i as u32,
                    0x8080_8080 + i as u32,
                ]
            }),
            nominal: Direction::North,
            geometry_invalid: false,
        }
    }
    #[test]
    fn identity_preserves_all_raw_words_and_cache_state() {
        let mut quad = fixture();
        quad.vertices[0][0] = 0x7fc0_4321;
        quad.vertices[0][1] = (-0.0f32).to_bits();
        let original = quad.clone();
        transform(&mut quad, Direction::North, 0).unwrap();
        assert_eq!(quad, original);
    }
    #[test]
    fn raw_spin_can_cross_enum_rows_but_bad_indices_leave_the_quad_untouched() {
        let mut first = fixture();
        let mut second = first.clone();
        transform(&mut first, Direction::North, -8).unwrap();
        transform(&mut second, Direction::Down, 0).unwrap();
        assert_eq!(first, second);
        let original = first.clone();
        assert!(transform(&mut first, Direction::East, i32::MAX).is_err());
        assert_eq!(first, original);
    }
    #[test]
    fn degenerate_winding_uses_last_uv_without_reordering_other_attributes() {
        let mut quad = fixture();
        for vertex in &mut quad.vertices {
            vertex[..3].fill(0.5f32.to_bits());
        }
        let original = quad.vertices;
        recalculate_winding(&mut quad.vertices, Direction::North);
        for (i, vertex) in quad.vertices.into_iter().enumerate() {
            assert_eq!(&vertex[..4], &original[i][..4]);
            assert_eq!(&vertex[4..6], &original[3][4..6]);
            assert_eq!(&vertex[6..], &original[i][6..]);
        }
    }
    #[test]
    fn packed_normals_keep_source_truncation_clamp_and_signed_byte_decode() {
        assert_eq!(
            pack_normal([f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0]),
            0x0081_7f00
        );
        assert_eq!(unpack_normal(0x0000_0080)[0], -128.0 / 127.0);
        assert_eq!(pack_normal([0.5, -0.5, 1.0, -1.0]), 0x817f_c13f);
    }
    #[test]
    fn every_orientation_preserves_opaque_words_and_rotates_face_metadata() {
        for side in Direction::ALL {
            for spin in 0..4 {
                let mut quad = fixture();
                let original = quad.clone();
                transform(&mut quad, side, spin).unwrap();
                assert_eq!(quad.header[1..], original.header[1..]);
                assert_eq!(quad.header[0], original.header[0]); // null cull, no normals
                assert_eq!(quad.nominal, side);
                for i in 0..4 {
                    for field in [3, 6, 7] {
                        assert_eq!(quad.vertices[i][field], original.vertices[i][field]);
                    }
                }
            }
        }
    }
}
