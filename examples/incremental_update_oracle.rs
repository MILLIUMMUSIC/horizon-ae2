use ae2_horizon::ae2_incremental_update::Updates;
use std::cmp::Ordering;
use std::io::{self, BufRead};
#[derive(Clone)]
struct Key {
    id: i32,
    object: i32,
}
impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for Key {}
impl PartialOrd for Key {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Key {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}
fn main() {
    let mut updates = Updates::default();
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let w: Vec<_> = line.split_whitespace().collect();
        let key = || Key {
            id: w[1].parse().unwrap(),
            object: w.get(2).map_or(0, |v| v.parse().unwrap()),
        };
        let value = match w[0] {
            "A" => updates
                .get_or_assign(&key())
                .map_or_else(|_| "error".into(), |n| n.to_string()),
            "S" => updates
                .serial(&key())
                .map_or_else(|| "null".into(), |n| n.to_string()),
            "R" => {
                updates.remove_serial(&key());
                "-".into()
            }
            "C" => {
                updates.clear();
                "-".into()
            }
            "F" => {
                updates.reset();
                "-".into()
            }
            "D" => {
                updates.add_change(key());
                "-".into()
            }
            "M" => {
                updates.commit();
                "-".into()
            }
            "Q" => updates
                .by_serial(w[1].parse().unwrap())
                .map_or_else(|| "null".into(), |k| format!("{}/{}", k.id, k.object)),
            _ => panic!("unknown action"),
        };
        let changes = updates
            .changes()
            .map(|k| format!("{}/{}", k.id, k.object))
            .collect::<Vec<_>>()
            .join(",");
        println!(
            "{value}|{}|{}|{changes}",
            updates.has_changes(),
            updates.is_full()
        );
    }
}
