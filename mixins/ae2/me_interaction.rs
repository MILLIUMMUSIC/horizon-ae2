// SPDX-License-Identifier: LGPL-3.0-or-later
// MEInteractionPacket and InventoryAction from pinned AE2, with the original
// Minecraft VarLong/VarInt byte semantics. Native envelope/auth lives outside.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum InventoryAction {
    PickupOrSetDown,
    SplitOrPlaceSingle,
    CreativeDuplicate,
    ShiftClick,
    CraftStack,
    CraftItem,
    CraftShift,
    CraftAll,
    FillItem,
    FillItemMoveToPlayer,
    FillEntireItem,
    FillEntireItemMoveToPlayer,
    EmptyItem,
    EmptyEntireItem,
    MoveRegion,
    PickupSingle,
    RollUp,
    RollDown,
    AutoCraft,
    PlaceSingle,
    SetFilter,
}
impl InventoryAction {
    pub const ALL: [Self; 21] = [
        Self::PickupOrSetDown,
        Self::SplitOrPlaceSingle,
        Self::CreativeDuplicate,
        Self::ShiftClick,
        Self::CraftStack,
        Self::CraftItem,
        Self::CraftShift,
        Self::CraftAll,
        Self::FillItem,
        Self::FillItemMoveToPlayer,
        Self::FillEntireItem,
        Self::FillEntireItemMoveToPlayer,
        Self::EmptyItem,
        Self::EmptyEntireItem,
        Self::MoveRegion,
        Self::PickupSingle,
        Self::RollUp,
        Self::RollDown,
        Self::AutoCraft,
        Self::PlaceSingle,
        Self::SetFilter,
    ];
    pub fn from_ordinal(value: i32) -> Option<Self> {
        usize::try_from(value)
            .ok()
            .and_then(|i| Self::ALL.get(i).copied())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Packet {
    pub container_id: i32,
    pub serial: i64,
    pub action: InventoryAction,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Truncated,
    VarLongTooBig,
    VarIntTooBig,
    InvalidAction,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodeError {
    pub kind: ErrorKind,
    pub consumed: usize,
}
fn read_var(input: &[u8], at: &mut usize, width: u32) -> Result<u64, DecodeError> {
    let max = if width == 64 { 10 } else { 5 };
    let mut value = 0u64;
    for count in 1..=max + 1 {
        let Some(&byte) = input.get(*at) else {
            return Err(DecodeError {
                kind: ErrorKind::Truncated,
                consumed: *at,
            });
        };
        *at += 1;
        // Minecraft consumes the eleventh/sixth byte before throwing, rather
        // than rejecting a continuation in the tenth/fifth byte prematurely.
        if count > max {
            return Err(DecodeError {
                kind: if width == 64 {
                    ErrorKind::VarLongTooBig
                } else {
                    ErrorKind::VarIntTooBig
                },
                consumed: *at,
            });
        }
        value |= u64::from(byte & 127) << ((count - 1) * 7);
        if byte & 128 == 0 {
            return Ok(if width == 32 {
                value & 0xffff_ffff
            } else {
                value
            });
        }
    }
    unreachable!()
}
fn write_var(mut value: u64, out: &mut Vec<u8>) {
    while value & !127 != 0 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
impl Packet {
    pub fn encode(self) -> Vec<u8> {
        let mut out = self.container_id.to_be_bytes().to_vec();
        write_var(self.serial as u64, &mut out);
        write_var(self.action as u64, &mut out);
        out
    }
    /// Return the consumed prefix. The source decoder leaves trailing bytes
    /// for its caller; framing/admission must remain an outer responsibility.
    pub fn decode(input: &[u8]) -> Result<(Self, usize), DecodeError> {
        if input.len() < 4 {
            return Err(DecodeError {
                kind: ErrorKind::Truncated,
                consumed: 0,
            });
        }
        let container_id = i32::from_be_bytes(input[..4].try_into().unwrap());
        let mut at = 4;
        let serial = read_var(input, &mut at, 64)? as i64;
        let ordinal = read_var(input, &mut at, 32)? as i32;
        let action = InventoryAction::from_ordinal(ordinal).ok_or(DecodeError {
            kind: ErrorKind::InvalidAction,
            consumed: at,
        })?;
        Ok((
            Self {
                container_id,
                serial,
                action,
            },
            at,
        ))
    }
}
pub trait Handler {
    fn container_id(&self) -> i32;
    fn handle_interaction(&mut self, serial: i64, action: InventoryAction);
}
/// The caller supplies only the current acting player's actual compatible
/// menu. Never look up a handler by the packet's container id or another player.
pub fn handle(packet: Packet, current: Option<&mut impl Handler>) {
    if let Some(handler) = current
        && handler.container_id() == packet.container_id
    {
        handler.handle_interaction(packet.serial, packet.action);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn negative_serials_use_ten_bytes_and_noncanonical_encodings_match_source() {
        let p = Packet {
            container_id: -123,
            serial: -1,
            action: InventoryAction::RollDown,
        };
        let bytes = p.encode();
        assert_eq!(bytes.len(), 15);
        assert_eq!(Packet::decode(&bytes), Ok((p, 15)));
        let mut data = vec![0, 0, 0, 1, 128, 0, 128, 0];
        data.push(77);
        let (p, used) = Packet::decode(&data).unwrap();
        assert_eq!(
            (p.serial, p.action, used),
            (0, InventoryAction::PickupOrSetDown, 8)
        );
        let mut high = vec![0, 0, 0, 1];
        high.extend([255; 9]);
        high.extend([127, 0]);
        assert_eq!(Packet::decode(&high).unwrap().0.serial, -1);
    }
    #[test]
    fn overflowing_or_truncated_varints_do_not_advance_like_a_canonical_only_parser() {
        let mut data = vec![0, 0, 0, 0];
        data.extend([128; 10]);
        assert_eq!(
            Packet::decode(&data),
            Err(DecodeError {
                kind: ErrorKind::Truncated,
                consumed: 14
            })
        );
        data.push(0);
        assert_eq!(
            Packet::decode(&data),
            Err(DecodeError {
                kind: ErrorKind::VarLongTooBig,
                consumed: 15
            })
        );
        let mut ordinal = vec![0, 0, 0, 0, 0];
        ordinal.extend([128; 5]);
        assert_eq!(
            Packet::decode(&ordinal),
            Err(DecodeError {
                kind: ErrorKind::Truncated,
                consumed: 10
            })
        );
        ordinal.push(0);
        assert_eq!(
            Packet::decode(&ordinal),
            Err(DecodeError {
                kind: ErrorKind::VarIntTooBig,
                consumed: 11
            })
        );
    }
    #[test]
    fn stale_window_and_absent_compatible_menu_do_not_dispatch() {
        struct Menu {
            id: i32,
            events: Vec<(i64, InventoryAction)>,
        }
        impl Handler for Menu {
            fn container_id(&self) -> i32 {
                self.id
            }
            fn handle_interaction(&mut self, s: i64, a: InventoryAction) {
                self.events.push((s, a));
            }
        }
        let mut menu = Menu {
            id: 8,
            events: vec![],
        };
        let p = Packet {
            container_id: 9,
            serial: -1,
            action: InventoryAction::PickupOrSetDown,
        };
        handle(p, Some(&mut menu));
        assert!(menu.events.is_empty());
        handle(p, None::<&mut Menu>);
        menu.id = 9;
        handle(p, Some(&mut menu));
        assert_eq!(menu.events, [(p.serial, p.action)]);
    }
}
