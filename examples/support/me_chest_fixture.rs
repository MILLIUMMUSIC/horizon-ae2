use ae2_horizon::{
    ae2_energy::{PowerEvent, StoredEnergy},
    ae2_me_chest::{self as chest, Context, Multiplier, State},
    ae2_storage_helper::Action,
};
use serde_json::Value;
use std::fmt::Write;
#[derive(Clone)]
struct Stack {
    key: i32,
    count: i32,
}
pub struct Fixture {
    state: State<i32>,
    heap: Vec<Stack>,
    cell: usize,
    input: usize,
    counts: [[i64; 3]; 2],
    cap: i64,
    local: StoredEnergy,
    grid_energy: f64,
    factor: f64,
    grid: bool,
    grid_powered: bool,
    node_powered: bool,
    pub online: bool,
    client: bool,
    level: bool,
    race: i32,
    notifying: [bool; 2],
    trace: String,
}
pub fn dbl(v: &Value) -> f64 {
    f64::from_bits(u64::from_str_radix(v.as_str().unwrap(), 16).unwrap())
}
pub fn bits(v: f64) -> String {
    format!(
        "{:x}",
        if v.is_nan() {
            f64::NAN.to_bits()
        } else {
            v.to_bits()
        }
    )
}
fn mode(a: Action) -> u8 {
    u8::from(a == Action::Modulate)
}
impl Fixture {
    pub fn new(v: &Value) -> Self {
        let factor = dbl(&v["factor"]);
        let mut f = Self {
            state: State::default(),
            heap: vec![Stack { key: -1, count: 0 }],
            cell: 0,
            input: 0,
            counts: std::array::from_fn(|i| {
                std::array::from_fn(|k| v["counts"][i][k].as_i64().unwrap())
            }),
            cap: v["cap"].as_i64().unwrap(),
            local: StoredEnergy::new(dbl(&v["local"]), 500. * factor),
            grid_energy: dbl(&v["grid_energy"]),
            factor,
            grid: v["grid"].as_bool().unwrap(),
            grid_powered: v["grid_powered"].as_bool().unwrap(),
            node_powered: v["node_powered"].as_bool().unwrap(),
            online: v["online"].as_bool().unwrap(),
            client: v["client"].as_bool().unwrap(),
            level: v["level"].as_bool().unwrap(),
            race: v["race"].as_i64().unwrap() as i32,
            notifying: [false; 2],
            trace: String::new(),
        };
        f.cell = f.stack(
            v["cell"][0].as_i64().unwrap() as i32,
            v["cell"][1].as_i64().unwrap() as i32,
        );
        f.input = f.stack(
            v["input"][0].as_i64().unwrap() as i32,
            v["input"][1].as_i64().unwrap() as i32,
        );
        f
    }
    pub fn stack(&mut self, key: i32, count: i32) -> usize {
        self.heap.push(Stack { key, count });
        self.heap.len() - 1
    }
    fn shown(&self, s: usize) -> String {
        if self.empty(&s) {
            "-1/0".to_owned()
        } else {
            format!("{}/{}", self.heap[s].key, self.heap[s].count)
        }
    }
    fn event(&mut self, s: impl AsRef<str>) {
        self.trace.push_str(s.as_ref());
        self.trace.push(';');
    }
    pub fn set_cell(&mut self, key: i32, count: i32) {
        self.cell = self.stack(key, count);
        self.notify(1);
    }
    fn notify(&mut self, id: usize) {
        if !self.client && !self.notifying[id] {
            self.notifying[id] = true;
            if id == 0 {
                chest::on_input_slot_changed(self);
            } else {
                chest::on_cell_slot_changed(self);
            }
            self.event(format!("SAVE:{id}"));
            self.notifying[id] = false;
        }
    }
    fn events(&mut self, events: Vec<PowerEvent>) {
        for event in events {
            self.event(format!("EVENT:{}", u8::from(event == PowerEvent::Receive)));
            chest::emit_power_event(self, event);
        }
    }
    pub fn set_power(&mut self, value: f64) {
        let events = self.local.set_stored(value);
        self.events(events);
    }
    pub fn finish(self, results: String) -> String {
        format!(
            "{}|{}|{}:{}:{}:{}:{}:{}:{}:{}|{}|{}|{:?}|{}:{}:{}",
            results,
            self.trace,
            self.state.handler.unwrap_or(-1),
            u8::from(self.state.cached),
            u8::from(self.state.fluid_handler),
            bits(self.state.idle),
            self.state.priority,
            self.state.client_cell_state,
            u8::from(self.state.client_powered),
            u8::from(self.state.was_online),
            self.shown(self.cell),
            self.shown(self.input),
            self.counts,
            bits(self.local.amount()),
            bits(self.grid_energy),
            bits(self.factor)
        )
    }
}
impl Context for Fixture {
    type Cell = i32;
    type Stack = usize;
    type Key = i32;
    type Source = ();
    type Fluid = (i32, i32);
    fn state(&self) -> &State<i32> {
        &self.state
    }
    fn state_mut(&mut self) -> &mut State<i32> {
        &mut self.state
    }
    fn client_side(&self) -> bool {
        self.client
    }
    fn level_present(&self) -> bool {
        self.level
    }
    fn cell_stack(&self) -> usize {
        self.cell
    }
    fn input_stack(&self) -> usize {
        self.input
    }
    fn input_empty(&self) -> bool {
        self.empty(&self.input)
    }
    fn empty_stack(&self) -> usize {
        0
    }
    fn empty(&self, s: &usize) -> bool {
        self.heap[*s].key < 0 || self.heap[*s].count <= 0
    }
    fn count(&self, s: &usize) -> i32 {
        if self.empty(s) {
            0
        } else {
            self.heap[*s].count
        }
    }
    fn shrink(&mut self, s: &usize, n: i32) {
        self.heap[*s].count = self.heap[*s].count.wrapping_sub(n);
    }
    fn set_input_direct(&mut self, s: usize) {
        self.input = s;
        self.notify(0);
    }
    fn item_key(&self, s: &usize) -> Option<i32> {
        (!self.empty(s)).then_some(self.heap[*s].key)
    }
    fn key_is_item(&self, k: &i32) -> bool {
        *k != 2
    }
    fn amount_per_operation(&self, k: &i32) -> i32 {
        if *k == 2 { 125 } else { 1 }
    }
    fn fluid_key(&self, fluid: &(i32, i32)) -> Option<i32> {
        (fluid.0 == 2 && fluid.1 > 0).then_some(2)
    }
    fn fluid_amount(&self, fluid: &(i32, i32)) -> i32 {
        fluid.1
    }
    fn machine_source(&self) {}
    fn load_cell(&mut self, s: &usize) -> Option<i32> {
        self.event(format!("LOAD:{}", self.shown(*s)));
        (!self.empty(s) && matches!(self.heap[*s].key, 11 | 13)).then_some(self.heap[*s].key)
    }
    fn has_cell_handler(&mut self, s: &usize) -> bool {
        self.event(format!("HANDLER:{}", self.shown(*s)));
        !self.empty(s) && (11..=13).contains(&self.heap[*s].key)
    }
    fn cell_idle(&self, c: &i32) -> f64 {
        if *c == 11 { 0.5 } else { 2. }
    }
    fn cell_status(&self, c: &i32) -> u8 {
        let n: i64 = self.counts[usize::from(*c != 11)].iter().sum();
        if n == 0 {
            1
        } else if n >= self.cap {
            4
        } else {
            2
        }
    }
    fn cell_insert(&mut self, c: &i32, k: &i32, n: i64, a: Action, _: &()) -> i64 {
        let i = usize::from(*c != 11);
        let k = *k as usize;
        let amount = n.max(0).min((self.cap - self.counts[i][k]).max(0));
        self.event(format!("I:{c}:{k}:{n}:{}:{amount}", mode(a)));
        if a == Action::Modulate && amount > 0 {
            self.counts[i][k] += amount;
            if self.race & 1 != 0 {
                self.input = self.stack(1, 1);
            }
            if self.race & 2 != 0 {
                self.set_cell(
                    if self.heap[self.cell].key == 11 {
                        13
                    } else {
                        11
                    },
                    1,
                );
            }
            chest::on_cell_content_changed(self);
        }
        amount
    }
    fn cell_extract(&mut self, c: &i32, k: &i32, n: i64, a: Action, _: &()) -> i64 {
        let i = usize::from(*c != 11);
        let k = *k as usize;
        let amount = n.max(0).min(self.counts[i][k].max(0));
        self.event(format!("X:{c}:{k}:{n}:{}:{amount}", mode(a)));
        if a == Action::Modulate && amount > 0 {
            self.counts[i][k] -= amount;
            chest::on_cell_content_changed(self);
        }
        amount
    }
    fn persist_cell(&mut self, c: &i32) {
        self.event(format!("PERSIST:{c}"));
    }
    fn set_node_idle(&mut self, idle: f64) {
        self.event(format!("IDLE:{}", bits(idle)));
        if self.race & 8 != 0 {
            self.set_cell(-1, 0);
        }
    }
    fn create_fluid_handler(&mut self) {
        self.event("FLUID");
    }
    fn node_powered(&self) -> bool {
        self.node_powered
    }
    fn node_online(&self) -> bool {
        self.online
    }
    fn grid_present(&self) -> bool {
        self.grid
    }
    fn grid_powered(&self) -> bool {
        self.grid_powered
    }
    fn local_power(&self) -> f64 {
        self.local.amount()
    }
    fn factor(&self, m: Multiplier) -> f64 {
        if m == Multiplier::One {
            1.
        } else {
            self.factor
        }
    }
    fn grid_extract(&mut self, n: f64, a: Action, m: Multiplier) -> f64 {
        self.event(format!(
            "G:{}:{}:{}",
            bits(n),
            mode(a),
            u8::from(m == Multiplier::Config)
        ));
        let result = n.max(0.).min(self.grid_energy);
        if a == Action::Modulate {
            self.grid_energy -= result;
            if self.race & 4 != 0 {
                self.factor = 2.;
            }
        }
        result
    }
    fn local_extract(&mut self, n: f64, a: Action) -> f64 {
        self.event(format!("L:{}:{}", bits(n), mode(a)));
        let (result, events) = self.local.extract(n, a == Action::Modulate);
        self.events(events);
        if a == Action::Modulate && self.race & 4 != 0 {
            self.factor = 2.;
        }
        result
    }
    fn request_storage_update(&mut self) {
        self.event("STORAGE");
    }
    fn receive_power_event(&mut self) {
        if self.grid {
            self.event("RECEIVE");
        }
    }
    fn invalidate_capabilities(&mut self) {
        self.event("CAP");
    }
    fn mark_for_update(&mut self) {
        self.event("MARK");
    }
    fn block_entity_changed(&mut self) {
        self.event("BLOCK");
    }
    fn mount_cell(&mut self, c: &i32, p: i32) {
        self.event(format!("MOUNT:{c}:{p}"));
    }
    fn open_storage_menu(&mut self) {
        self.event("OPEN:0");
    }
    fn open_cell_menu(&mut self) {
        self.event("OPEN:1");
    }
}
pub fn run(row: &Value) -> String {
    let mut f = Fixture::new(row);
    let mut results = String::new();
    for a in row["ops"].as_array().unwrap() {
        let action = |n: usize| {
            if a[n].as_bool().unwrap() {
                Action::Modulate
            } else {
                Action::Simulate
            }
        };
        let key = |n: usize| a[n].as_i64().unwrap() as i32;
        let mut result = "-".to_owned();
        match a[0].as_str().unwrap() {
            "U" => chest::open_cell_inventory(&mut f),
            "V" => {
                if f.state.fluid_handler {
                    result = format!(
                        "{}:{}:{}:{}",
                        chest::fluid_capacity(&f),
                        u8::from(chest::fluid_valid(&f)),
                        chest::tank_capacity(key(1)),
                        u8::from(chest::tank_valid(key(1)))
                    );
                }
            }
            "Z" => {
                result = if f.state.fluid_handler {
                    chest::fill_fluid(&mut f, &(key(1), key(2)), action(3))
                } else {
                    0
                }
                .to_string();
            }
            "H" => chest::update_handler(&mut f),
            "T" => chest::server_tick(&mut f),
            "D" => chest::recalculate_display(&mut f),
            "C" => f.set_cell(key(1), key(2)),
            "I" => {
                let s = f.stack(key(1), key(2));
                f.set_input_direct(s);
            }
            "F" => {
                let s = f.stack(key(1), key(2));
                result = u8::from(chest::input_allowed(&mut f, &s)).to_string();
            }
            "L" => {
                result = match chest::link(&mut f) {
                    chest::Link::UnreadableCell => 0,
                    chest::Link::OutOfPower => 1,
                    chest::Link::Connected => 2,
                }
                .to_string()
            }
            "Q" => chest::set_priority(&mut f, key(1)),
            "N" => {
                f.online = a[1].as_bool().unwrap();
                chest::on_node_state_changed(&mut f);
            }
            "M" => chest::mount(&mut f),
            "O" => result = u8::from(chest::open_gui(&mut f)).to_string(),
            "P" => f.set_power(dbl(&a[1])),
            "E" => {
                result = bits(chest::extract_power(
                    &mut f,
                    dbl(&a[1]),
                    action(2),
                    if key(3) == 0 {
                        Multiplier::One
                    } else {
                        Multiplier::Config
                    },
                ))
            }
            "X" | "Y" => {
                chest::update_handler(&mut f);
                let mut n = 0;
                if let Some(c) = f.state.handler {
                    let k = key(1);
                    let amount = a[2].as_i64().unwrap();
                    n = if a[0] == "X" {
                        chest::monitor_extract(&mut f, &c, &k, amount, action(3), &())
                    } else {
                        chest::monitor_insert(&mut f, &c, &k, amount, action(3), &())
                    };
                }
                result = n.to_string();
            }
            _ => unreachable!(),
        }
        write!(results, "{result},").unwrap();
    }
    f.finish(results)
}
