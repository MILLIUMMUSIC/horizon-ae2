#!/usr/bin/env python3
"""AE2 full-orientation blockstate rotations in Horizon's variant form.

The source jar writes `x`, `y` and the AE2-only `ae2:z` variant keys. AE2's
`VariantDeserializerMixin` turns those into a JOML `rotateYXZ(-y, -x, -z)`
transform. Horizon's blockstate parser reads plain `x`, `y`, `z` quarter turns
and composes them in its own order, so this module finds the equivalent
Horizon triple for each source orientation. The mapping is derived from the
original `BlockOrientation` angle table; it is not a hand-written approximation.
"""
import math

# `Direction.values()` order, which indexes `BlockOrientation` and `SpinMapping`.
FACING_ORDER = ["down", "up", "north", "south", "west", "east"]

# `BlockOrientation` angles in enum order: facing ordinal * 4 + spin.
ANGLES = [
    (90, 0, 0), (90, 0, 270), (90, 0, 180), (90, 0, 90),
    (270, 0, 180), (270, 0, 90), (270, 0, 0), (270, 0, 270),
    (0, 0, 0), (0, 0, 270), (0, 0, 180), (0, 0, 90),
    (0, 180, 0), (0, 180, 90), (0, 180, 180), (0, 180, 270),
    (0, 270, 0), (0, 270, 270), (0, 270, 180), (0, 270, 90),
    (0, 90, 0), (0, 90, 270), (0, 90, 180), (0, 90, 90),
]


def _q(axis, degrees):
    half = math.radians(degrees) / 2.0
    s, c = math.sin(half), math.cos(half)
    if axis == "x":
        return (s, 0.0, 0.0, c)
    if axis == "y":
        return (0.0, s, 0.0, c)
    return (0.0, 0.0, s, c)


def _mul(a, b):
    ax, ay, az, aw = a
    bx, by, bz, bw = b
    return (aw * bx + ax * bw + ay * bz - az * by,
            aw * by - ax * bz + ay * bw + az * bx,
            aw * bz + ax * by - ay * bx + az * bw,
            aw * bw - ax * bx - ay * by - az * bz)


def _matrix(x, y, z, w):
    return ((1 - 2 * (y * y + z * z), 2 * (x * y - z * w), 2 * (x * z + y * w)),
            (2 * (x * y + z * w), 1 - 2 * (x * x + z * z), 2 * (y * z - x * w)),
            (2 * (x * z - y * w), 2 * (y * z + x * w), 1 - 2 * (x * x + y * y)))


def _round(matrix):
    return tuple(tuple(int(round(value)) for value in row) for row in matrix)


def source_matrix(angle_x, angle_y, angle_z):
    """JOML `rotateYXZ(-y, -x, -z)` as an integer rotation matrix."""
    q = _mul(_mul(_q("y", -angle_y), _q("x", -angle_x)), _q("z", -angle_z))
    return _round(_matrix(*q))


def horizon_matrix(turns):
    """Horizon's `BlockModelTransform` rows construction for x, y, z turns."""
    rows = [[1, 0, 0], [0, 1, 0], [0, 0, 1]]
    for axis, count in enumerate(turns):
        for _ in range(count % 4):
            a, b = (axis + 1) % 3, (axis + 2) % 3
            old = rows[a]
            rows[a] = rows[b][:]
            rows[b] = [-value for value in old]
    return tuple(tuple(row) for row in rows)


_TRIPLES = {}
for _x in range(4):
    for _y in range(4):
        for _z in range(4):
            _TRIPLES.setdefault(horizon_matrix((_x, _y, _z)), (_x, _y, _z))
assert len(_TRIPLES) == 24


def horizon_turns(facing, spin):
    """The Horizon x, y, z quarter turns for a source facing and spin."""
    index = FACING_ORDER.index(facing) * 4 + spin
    target = source_matrix(*ANGLES[index])
    try:
        return _TRIPLES[target]
    except KeyError:
        raise ValueError(f"no Horizon rotation for {facing} spin {spin}") from None


def oriented_variants(original, model=None):
    """Rewrite an original 24-variant blockstate for Horizon's rotation keys.

    Every source variant must be present, and its own `x`, `y`, `ae2:z` keys
    must agree with the pinned `BlockOrientation` table. The caller may
    substitute the model (Drive composes its cells/LEDs at runtime).
    """
    variants = {}
    for key, value in original.get("variants", {}).items():
        parts = dict(clause.split("=") for clause in key.split(","))
        if set(parts) != {"facing", "spin"}:
            raise ValueError(f"unexpected orientation variant {key}")
        facing, spin = parts["facing"], int(parts["spin"])
        if facing not in FACING_ORDER or not 0 <= spin <= 3:
            raise ValueError(f"unexpected orientation variant {key}")
        expected = ANGLES[FACING_ORDER.index(facing) * 4 + spin]
        found = (int(value.get("x", 0)), int(value.get("y", 0)),
                 int(value.get("ae2:z", 0)))
        if found != expected:
            raise ValueError(f"variant {key} angles {found} differ from {expected}")
        variant = {"model": model or value["model"]}
        for axis, turns in zip(("x", "y", "z"), horizon_turns(facing, spin)):
            if turns:
                variant[axis] = turns * 90
        if key in variants:
            raise ValueError(f"duplicate orientation variant {key}")
        variants[key] = variant
    if len(variants) != 24:
        raise ValueError(f"expected 24 orientation variants, found {len(variants)}")
    return variants
