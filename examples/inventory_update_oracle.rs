use ae2_horizon::{
    ae2_incremental_update::Updates,
    ae2_inventory_update::{Builder, DecodedPacket, Entry, KeyCodec, Repo},
};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write,
    io::{self, BufRead},
};
struct Codec {
    padding: usize,
    fail: i32,
}
fn write_var(mut n: u64, out: &mut Vec<u8>) {
    while n > 127 {
        out.push((n as u8 & 127) | 128);
        n >>= 7;
    }
    out.push(n as u8);
}
fn read_var(input: &[u8], at: &mut usize) -> u32 {
    let mut n = 0;
    let mut shift = 0;
    loop {
        let b = input[*at];
        *at += 1;
        n |= u32::from(b & 127) << shift;
        if b < 128 {
            return n;
        }
        shift += 7;
    }
}
impl KeyCodec<i32> for Codec {
    type Error = ();
    fn write(&mut self, key: Option<&i32>, out: &mut Vec<u8>) -> Result<(), ()> {
        out.push(u8::from(key.is_some()));
        if let Some(&key) = key {
            write_var(key as u64, out);
            if key == self.fail {
                return Err(());
            }
            write_var(self.padding as u64, out);
            out.extend((0..self.padding).map(|i| (key as usize * 31 + i) as u8));
        }
        Ok(())
    }
    fn read(&mut self, input: &[u8], at: &mut usize) -> Result<Option<i32>, ()> {
        let present = input[*at] != 0;
        *at += 1;
        if !present {
            return Ok(None);
        }
        let key = read_var(input, at) as i32;
        let length = read_var(input, at) as usize;
        *at += length;
        Ok(Some(key))
    }
}
fn counts(v: &Value) -> BTreeMap<i32, i64> {
    v.as_array()
        .unwrap()
        .iter()
        .enumerate()
        .filter_map(|(i, n)| {
            let n = n.as_i64().unwrap();
            (n != 0).then_some((i as i32, n))
        })
        .collect()
}
fn hex(bytes: &[u8], out: &mut String) {
    const DIGITS: &[u8] = b"0123456789abcdef";
    for &b in bytes {
        out.push(DIGITS[(b >> 4) as usize] as char);
        out.push(DIGITS[(b & 15) as usize] as char);
    }
}
fn main() {
    for line in io::stdin().lock().lines() {
        let row: Value = serde_json::from_str(&line.unwrap()).unwrap();
        let mut updates = Updates::default();
        let mut repo = Repo::default();
        let mut warnings = Vec::new();
        let mut out = String::new();
        for op in row["ops"].as_array().unwrap() {
            let mode = op["mode"].as_str().unwrap();
            match mode {
                "clear" => {
                    updates.clear();
                    continue;
                }
                "reset" => {
                    updates.reset();
                    continue;
                }
                "remove" => {
                    updates.remove_serial(&(op["key"].as_i64().unwrap() as i32));
                    continue;
                }
                _ => {}
            }
            let mut builder = Builder::new(op["id"].as_i64().unwrap() as i32, updates.is_full());
            let mut codec = Codec {
                padding: op["padding"].as_u64().unwrap() as usize,
                fail: op["fail"].as_i64().unwrap() as i32,
            };
            let result = if mode == "raw" {
                (0..op["n"].as_i64().unwrap()).try_for_each(|i| {
                    builder.add(
                        &Entry {
                            serial: i,
                            key: Some((i % 8) as i32),
                            stored: i,
                            requestable: -i,
                            craftable: i & 1 == 0,
                        },
                        &mut codec,
                    )
                })
            } else {
                for key in op["changes"].as_array().unwrap() {
                    updates.add_change(key.as_i64().unwrap() as i32);
                }
                let storage = counts(&op["stored"]);
                let req = counts(&op["req"]);
                let craft: BTreeSet<_> = (0..8)
                    .filter(|i| op["craft"].as_i64().unwrap() & (1 << i) != 0)
                    .collect();
                let visible = |k: &i32| op["filter"].as_i64().unwrap() & (1 << k) != 0;
                if mode == "full" {
                    builder.add_full(&mut updates, &storage, &craft, &req, visible, &mut codec)
                } else {
                    builder.add_changes(&mut updates, &storage, &craft, &req, visible, &mut codec)
                }
            };
            let fault = result.is_err();
            out.push_str(if fault { "FAULT|" } else { "OK|" });
            let len = builder.build().len();
            assert_eq!(builder.build().len(), len);
            for packet in builder.build() {
                let bytes = packet.encode();
                write!(out, "{}:{}:", packet.count, i32::from(packet.full)).unwrap();
                hex(&bytes, &mut out);
                out.push(';');
                if !fault {
                    let decoded = DecodedPacket::decode(&bytes, &mut 0, &mut codec).unwrap();
                    repo.update(decoded.full, &decoded.entries, |s| warnings.push(s));
                }
            }
            out.push('|');
            for k in 0..8 {
                if let Some(n) = updates.serial(&k) {
                    write!(out, "{n},").unwrap();
                } else {
                    out.push_str("null,");
                }
            }
            write!(out, "{}:", i32::from(updates.is_full())).unwrap();
            for key in updates.changes() {
                write!(out, "{key},").unwrap();
            }
            out.push('|');
            for e in repo.entries().values() {
                write!(
                    out,
                    "{}:{}:{}:{}:{},",
                    e.serial,
                    e.key.unwrap_or(-1),
                    e.stored,
                    e.requestable,
                    i32::from(e.craftable)
                )
                .unwrap();
            }
            out.push('|');
            for s in &warnings {
                write!(out, "{s},").unwrap();
            }
            out.push('#');
        }
        println!("{out}");
    }
}
