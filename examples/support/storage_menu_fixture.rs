use ae2_horizon::{
    ae2_me_interaction::InventoryAction, ae2_storage_helper as power, ae2_storage_menu as menu,
};
use serde_json::Value;
#[derive(Clone)]
struct Stack {
    key: i32,
    count: i32,
}
pub struct Fixture {
    heap: Vec<Stack>,
    carried: usize,
    slots: [usize; 2],
    counts: [i64; 5],
    energy: f64,
    connected: bool,
    creative: bool,
    locator: bool,
    player: bool,
    kind: i32,
    tank: i64,
    cap: i64,
    race: i32,
    pub trace: String,
}
fn max_count(key: i32) -> i32 {
    match key {
        1 | 3 => 16,
        4 => 1,
        _ => 64,
    }
}
fn registry_item(key: i32) -> i32 {
    match key {
        0 | 1 => 0,
        3 => 1,
        4 => 2,
        _ => -1,
    }
}
fn op(mode: power::Action) -> u8 {
    u8::from(mode == power::Action::Modulate)
}
impl Fixture {
    pub fn new(v: &Value) -> Self {
        let mut f = Self {
            heap: vec![Stack { key: -1, count: 0 }],
            carried: 0,
            slots: [0; 2],
            counts: std::array::from_fn(|i| v["counts"][i].as_i64().unwrap()),
            energy: f64::from_bits(u64::from_str_radix(v["energy"].as_str().unwrap(), 16).unwrap()),
            connected: v["connected"].as_bool().unwrap(),
            creative: v["creative"].as_bool().unwrap(),
            locator: v["locator"].as_bool().unwrap(),
            player: v["player"].as_bool().unwrap(),
            kind: v["kind"].as_i64().unwrap() as i32,
            tank: v["tank"].as_i64().unwrap(),
            cap: v["cap"].as_i64().unwrap(),
            race: v["race"].as_i64().unwrap() as i32,
            trace: String::new(),
        };
        f.carried = f.stack(
            v["cursor"][0].as_i64().unwrap() as i32,
            v["cursor"][1].as_i64().unwrap() as i32,
        );
        for i in 0..2 {
            f.slots[i] = f.stack(
                v["slots"][i][0].as_i64().unwrap() as i32,
                v["slots"][i][1].as_i64().unwrap() as i32,
            );
        }
        f
    }
    fn stack(&mut self, key: i32, count: i32) -> usize {
        self.heap.push(Stack { key, count });
        self.heap.len() - 1
    }
    fn empty(&self, s: usize) -> bool {
        self.heap[s].key < 0 || self.heap[s].count <= 0
    }
    fn count(&self, s: usize) -> i32 {
        if self.empty(s) { 0 } else { self.heap[s].count }
    }
    fn describe(&self, s: usize) -> String {
        if self.empty(s) {
            "-1/0".into()
        } else {
            format!("{}/{}", self.heap[s].key, self.heap[s].count)
        }
    }
    fn append(&mut self, text: String) {
        self.trace.push_str(&text);
        self.trace.push(';');
    }
    pub fn finish(&self, fault: bool) -> String {
        format!(
            "{}|{}|{}|{},{}|{},{},{},{},{}|{:x}|{}",
            if fault { "NullFluidKey" } else { "OK" },
            self.trace,
            self.describe(self.carried),
            self.describe(self.slots[0]),
            self.describe(self.slots[1]),
            self.counts[0],
            self.counts[1],
            self.counts[2],
            self.counts[3],
            self.counts[4],
            self.energy.to_bits(),
            self.tank
        )
    }
}
impl power::Context for Fixture {
    type Key = i32;
    fn extract(&mut self, key: &i32, amount: i64, mode: power::Action) -> i64 {
        let n = amount.max(0).min(self.counts[*key as usize].max(0));
        self.append(format!("E:{key}:{amount}:{}:{n}", op(mode)));
        if mode == power::Action::Modulate {
            self.counts[*key as usize] -= n;
        }
        n
    }
    fn insert(&mut self, key: &i32, amount: i64, mode: power::Action) -> i64 {
        let n = amount
            .max(0)
            .min((1_000_000 - self.counts[*key as usize]).max(0));
        self.append(format!("I:{key}:{amount}:{}:{n}", op(mode)));
        if mode == power::Action::Modulate {
            self.counts[*key as usize] += n;
        }
        n
    }
    fn power(&mut self, amount: f64, mode: power::Action) -> f64 {
        self.append(format!("P:{:x}:{}", amount.to_bits(), op(mode)));
        let result = if amount <= 0. {
            0.
        } else if self.energy.is_nan() {
            f64::NAN
        } else {
            amount.min(self.energy)
        };
        if mode == power::Action::Modulate {
            self.energy -= result;
            if self.race & 1 != 0 {
                let stack = self.stack(1, 1);
                self.carried = stack;
            }
            if self.race & 2 != 0 {
                for n in &mut self.counts {
                    *n /= 2;
                }
            }
            if self.race & 4 != 0 {
                let stack = self.stack(1, 1);
                self.slots[0] = stack;
            }
        }
        result
    }
    fn amount_per_operation(&self, key: &i32) -> i32 {
        if *key == 2 { 125 } else { 1 }
    }
    fn is_item(&self, key: &i32) -> bool {
        *key != 2
    }
    fn statistic(&mut self, kind: power::Statistic, amount: i32) {
        if self.player {
            self.append(format!("T:{kind:?}:{amount}"));
        }
    }
}
impl menu::Context for Fixture {
    type Stack = usize;
    type Item = i32;
    type Slot = usize;
    type Container = ();
    fn connected(&mut self) -> bool {
        let result = self.connected;
        self.append(format!("L:{}", u8::from(result)));
        result
    }
    fn has_locator(&self) -> bool {
        self.locator
    }
    fn open_craft_amount(&mut self, key: &i32) {
        self.append(format!("O:{key}:{}", if *key == 2 { 1000 } else { 1 }));
    }
    fn carried(&self) -> usize {
        self.carried
    }
    fn set_carried(&mut self, s: usize) {
        self.carried = s;
        self.append(format!("C:{}", self.describe(s)));
    }
    fn empty_stack(&self) -> usize {
        0
    }
    fn is_empty(&self, s: &usize) -> bool {
        self.empty(*s)
    }
    fn count(&self, s: &usize) -> i32 {
        self.count(*s)
    }
    fn max_stack(&self, s: &usize) -> i32 {
        max_count(self.heap[*s].key)
    }
    fn item(&self, s: &usize) -> i32 {
        if self.empty(*s) {
            -1
        } else {
            registry_item(self.heap[*s].key)
        }
    }
    fn key_of_stack(&self, s: &usize) -> Option<i32> {
        (!self.empty(*s)).then_some(self.heap[*s].key)
    }
    fn key_matches(&self, key: &i32, s: &usize) -> bool {
        !self.empty(*s) && self.heap[*s].key == *key
    }
    fn key_max_stack(&self, key: &i32) -> i32 {
        max_count(*key)
    }
    fn to_stack(&mut self, key: &i32, count: i32) -> usize {
        self.stack(*key, count)
    }
    fn copy_stack(&mut self, s: &usize) -> usize {
        let old = self.heap[*s].clone();
        self.stack(old.key, old.count)
    }
    fn set_count(&mut self, s: &usize, n: i32) {
        self.heap[*s].count = n;
    }
    fn grow(&mut self, s: &usize, n: i32) {
        self.heap[*s].count = self.heap[*s].count.wrapping_add(n);
    }
    fn shrink(&mut self, s: &usize, n: i32) {
        self.heap[*s].count = self.heap[*s].count.wrapping_sub(n);
    }
    fn creative(&self) -> bool {
        self.creative
    }
    fn player_items_len(&self) -> usize {
        2
    }
    fn add_to_player(&mut self, s: &usize) -> bool {
        self.append(format!("ADD:{}", self.describe(*s)));
        for empty in [false, true] {
            for i in 0..2 {
                let slot = self.slots[i];
                if self.empty(slot) != empty {
                    continue;
                }
                if self.empty(slot) || self.heap[slot].key == self.heap[*s].key {
                    let amount = self
                        .count(*s)
                        .min(max_count(self.heap[*s].key) - self.count(slot))
                        .max(0);
                    if amount > 0 {
                        if self.empty(slot) {
                            self.slots[i] = self.stack(self.heap[*s].key, amount);
                        } else {
                            self.heap[slot].count += amount;
                        }
                        self.heap[*s].count -= amount;
                    }
                }
            }
        }
        self.empty(*s)
    }
    fn quick_move_destinations(&mut self, s: &usize) -> Vec<usize> {
        self.append(format!("DEST:{}", self.describe(*s)));
        vec![0, 1]
    }
    fn placeable_amount(&mut self, slot: &usize, key: &i32) -> i32 {
        let s = self.slots[*slot];
        let result = if self.empty(s) {
            max_count(*key)
        } else if self.heap[s].key == *key {
            (max_count(*key) - self.count(s)).max(0)
        } else {
            0
        };
        self.append(format!("PLACE:{slot}:{key}:{result}"));
        result
    }
    fn slot_item(&self, slot: &usize) -> usize {
        self.slots[*slot]
    }
    fn slot_set_by_player(&mut self, slot: &usize, s: usize) {
        self.slots[*slot] = s;
        self.append(format!("SI:{slot}:{}", self.describe(s)));
    }
    fn has_fluid_bucket(&self, key: &i32) -> bool {
        *key == 2
    }
    fn is_bucket(&self, s: &usize) -> bool {
        self.item(s) == 1
    }
    fn bucket_key(&self) -> i32 {
        3
    }
    fn default_bucket(&mut self) -> usize {
        self.stack(3, 1)
    }
    fn amount_per_unit(&self, key: &i32) -> i32 {
        if *key == 2 { 1000 } else { 1 }
    }
    fn filling_context(&mut self, key: Option<&i32>) -> Option<()> {
        self.append(format!("F:{}", key.copied().unwrap_or(-1)));
        (self.kind != 0 && !self.empty(self.carried) && key.is_none_or(|k| *k == 2)).then_some(())
    }
    fn emptying_context(&mut self) -> Option<()> {
        self.append("N".into());
        (self.kind != 0 && !self.empty(self.carried)).then_some(())
    }
    fn container_content(&mut self, _: &()) -> Option<(i32, i64)> {
        let content = if self.kind == 1 {
            if self.heap[self.carried].key == 4 {
                1000
            } else {
                0
            }
        } else {
            self.tank
        };
        self.append(format!("GET:{content}"));
        Some((2, content))
    }
    fn container_insert(&mut self, _: &(), key: &i32, n: i64, mode: power::Action) -> i64 {
        let result = if *key != 2 {
            0
        } else if self.kind == 1 {
            if self.is_bucket(&self.carried) && n >= 1000 {
                1000
            } else {
                0
            }
        } else {
            n.max(0).min((self.cap - self.tank).max(0))
        };
        self.append(format!("CI:{n}:{}:{result}", op(mode)));
        if mode == power::Action::Modulate && result != 0 {
            if self.kind == 1 {
                let stack = self.stack(4, 1);
                self.set_carried(stack);
            } else {
                self.tank += result;
            }
        }
        result
    }
    fn container_extract(&mut self, _: &(), _: &i32, n: i64, mode: power::Action) -> i64 {
        let available = if self.kind == 1 {
            if self.heap[self.carried].key == 4 {
                1000
            } else {
                0
            }
        } else {
            self.tank
        };
        let mut result = if self.kind == 1 {
            if n >= 1000 { available } else { 0 }
        } else {
            n.max(0).min(available.max(0))
        };
        if mode == power::Action::Modulate && self.kind == 3 && result > 0 {
            result -= 1;
        }
        self.append(format!("CE:{n}:{}:{result}", op(mode)));
        if mode == power::Action::Modulate && result != 0 {
            if self.kind == 1 {
                let stack = self.stack(3, 1);
                self.set_carried(stack);
            } else {
                self.tank -= result;
            }
        }
        result
    }
    fn play_fill_sound(&mut self, _: &(), _: &i32) {
        self.append("SF".into());
    }
    fn play_empty_sound(&mut self, _: &(), _: &i32) {
        self.append("SE".into());
    }
    fn error(&mut self, error: menu::Error) {
        self.append(format!("ERR:{error:?}"));
    }
    fn unhandled(&mut self, action: InventoryAction) {
        self.append(format!("U:{}", action as i32));
    }
}
