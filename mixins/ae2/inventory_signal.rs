// SPDX-License-Identifier: LGPL-3.0-or-later
// InternalInventory -> ContainerAdapter -> AbstractContainerMenu. Both native
// hosts have slot limit one; min(container limit, valid item limit) is thus one.
pub fn signal(counts: impl IntoIterator<Item = u32>, slots: usize) -> i32 {
    if slots == 0 {
        return 0;
    }
    let mut fullness = 0.0_f32;
    for count in counts.into_iter().take(slots) {
        fullness += count as f32;
    }
    fullness /= slots as f32;
    // Mth.lerpDiscrete(fullness, 0, 15), without an invented fullness clamp.
    ((fullness * 14.0).floor() as i32).wrapping_add(i32::from(fullness > 0.0))
}

/// ContainerAdapter takes the minimum of the host's slot limits, then the
/// vanilla comparator takes min(container limit, each item's max stack size).
pub fn signal_with_limits(stacks: impl IntoIterator<Item = (u32, u32)>, slots: usize) -> i32 {
    if slots == 0 {
        return 0;
    }
    let mut fullness = 0.0_f32;
    for (count, limit) in stacks.into_iter().take(slots) {
        if count > 0 {
            fullness += count as f32 / limit as f32;
        }
    }
    fullness /= slots as f32;
    ((fullness * 14.0).floor() as i32).wrapping_add(i32::from(fullness > 0.0))
}

#[cfg(test)]
mod tests {
    use super::signal;

    #[test]
    fn inscriber_uses_minimum_container_limit_even_for_its_larger_output_slot() {
        use super::signal_with_limits as level;
        assert_eq!(level([(1, 64), (0, 64), (0, 64), (0, 64)], 4), 1);
        assert_eq!(level([(1, 4), (0, 4), (0, 4), (0, 4)], 4), 1);
        assert_eq!(level([(1, 1), (0, 1), (0, 1), (0, 1)], 4), 4);
        assert_eq!(level([(1, 1); 4], 4), 15);
        assert_eq!(level([(4, 4); 4], 4), 15);
        assert_eq!(level([(64, 64); 4], 4), 15);
        // A normal output stack still divides by the configured input limit;
        // preserve the source result, rather than silently clamp fullness.
        assert_eq!(level([(0, 1), (0, 1), (0, 1), (64, 1)], 4), 225);
    }

    #[test]
    fn all_drive_occupancies_use_ten_real_slots_and_ignore_transport_padding() {
        let expected = [0, 2, 3, 5, 6, 8, 9, 10, 12, 13, 15];
        for mask in 0_u32..1024 {
            let counts = (0..18).map(|slot| if slot < 10 { (mask >> slot) & 1 } else { 99 });
            assert_eq!(signal(counts, 10), expected[mask.count_ones() as usize]);
        }
    }

    #[test]
    fn direct_source_overstacks_are_not_clamped_to_the_normal_insertion_limit() {
        for count in 0..=99 {
            assert_eq!(
                signal([count], 1),
                (count * 14 + u32::from(count > 0)) as i32
            );
        }
        for total in 0..=990 {
            assert_eq!(
                signal([total], 10),
                (total * 14 / 10 + u32::from(total > 0)) as i32
            );
        }
    }

    #[test]
    fn missing_or_zero_count_slots_are_empty() {
        assert_eq!(signal([], 1), 0);
        assert_eq!(signal([0; 18], 10), 0);
        assert_eq!(signal([99], 0), 0);
    }
}
