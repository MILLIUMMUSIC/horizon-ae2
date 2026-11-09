use ae2_horizon::ae2_inventory_access::{self as access, Context, Fault};
use std::fmt::Write;

#[derive(Clone)]
struct Stack {
    key: i32,
    count: i32,
    maximum: i32,
    component: i32,
}
pub struct Fixture {
    heap: Vec<Stack>,
    slot: usize,
    original: usize,
    argument: usize,
    limit: i32,
    valid: bool,
    extract: bool,
    client: i32,
    enabled: bool,
    notifying: bool,
    race: i32,
    callbacks: i32,
    trace: String,
}
impl Fixture {
    fn add(&mut self, key: i32, count: i32, maximum: i32, component: i32) -> usize {
        self.heap.push(Stack {
            key,
            count,
            maximum,
            component,
        });
        self.heap.len() - 1
    }
    fn show(&self, stack: usize) -> String {
        let s = &self.heap[stack];
        format!(
            "{stack}:{}/{}/{}/{}",
            s.key, s.count, s.maximum, s.component
        )
    }
    pub fn run(line: &str) -> String {
        let w: Vec<_> = line.split_whitespace().collect();
        let n = |i: usize| w[i].parse::<i32>().unwrap();
        let mut f = Self {
            heap: vec![Stack {
                key: -1,
                count: 0,
                maximum: 64,
                component: 0,
            }],
            slot: 0,
            original: 0,
            argument: 0,
            limit: n(6),
            valid: n(4) != 0,
            extract: n(5) != 0,
            client: n(7),
            enabled: n(8) != 0,
            notifying: false,
            race: n(9),
            callbacks: 0,
            trace: String::new(),
        };
        f.slot = f.add(n(10), n(11), n(12), n(13));
        f.original = f.slot;
        f.argument = if n(18) != 0 {
            f.slot
        } else {
            f.add(n(14), n(15), n(16), n(17))
        };
        let result = match w[0] {
            "I" => {
                let arg = f.argument;
                access::insert(&mut f, n(1), arg, n(3) != 0)
            }
            "E" => access::extract(&mut f, n(1), n(2), n(3) != 0),
            "W" => access::wrapped_extract(&mut f, n(1), n(2), n(3) != 0),
            _ => panic!("operation"),
        };
        let result = match result {
            Ok(stack) => f.show(stack),
            Err(Fault::SlotCheck) => "SlotCheck".into(),
            Err(Fault::SlotRead) => "SlotRead".into(),
        };
        format!(
            "{result}|{}|{}|{}|{}",
            f.show(f.slot),
            f.show(f.argument),
            f.show(f.original),
            f.trace
        )
    }
}
impl Context for Fixture {
    type Stack = usize;
    fn size(&self) -> i32 {
        1
    }
    fn stack(&mut self, slot: i32) -> Result<usize, Fault> {
        write!(self.trace, "GET:{slot};").unwrap();
        if slot == 0 {
            Ok(self.slot)
        } else {
            Err(Fault::SlotRead)
        }
    }
    fn empty_stack(&self) -> usize {
        0
    }
    fn stored_stack(&mut self, slot: i32) -> Result<usize, Fault> {
        if slot == 0 {
            Ok(self.slot)
        } else {
            Err(Fault::SlotRead)
        }
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
    fn maximum(&self, s: &usize) -> i32 {
        self.heap[*s].maximum
    }
    fn copy_stack(&mut self, s: &usize) -> usize {
        write!(self.trace, "COPY:{s};").unwrap();
        if self.empty(s) {
            0
        } else {
            let item = self.heap[*s].clone();
            self.add(item.key, item.count, item.maximum, item.component)
        }
    }
    fn set_count(&mut self, s: &usize, count: i32) {
        write!(self.trace, "COUNT:{s}:{count};").unwrap();
        self.heap[*s].count = count;
    }
    fn slot_limit(&mut self, slot: i32) -> i32 {
        write!(self.trace, "LIMIT:{slot};").unwrap();
        self.limit
    }
    fn valid(&mut self, slot: i32, stack: &usize) -> bool {
        write!(self.trace, "VALID:{slot}:{stack};").unwrap();
        self.valid
    }
    fn same(&mut self, a: &usize, b: &usize) -> bool {
        write!(self.trace, "SAME:{a}:{b};").unwrap();
        self.heap[*a].key == self.heap[*b].key && self.heap[*a].component == self.heap[*b].component
    }
    fn allow_extract(&mut self, slot: i32, amount: i32) -> bool {
        write!(self.trace, "EXTRACT:{slot}:{amount};").unwrap();
        self.extract
    }
    fn set_direct(&mut self, slot: i32, stack: usize) {
        assert_eq!(slot, 0);
        self.slot = stack;
        self.notify(slot);
    }
    fn notify(&mut self, slot: i32) {
        if self.client < 0 || (self.client != 0 && !self.enabled) || self.notifying {
            return;
        }
        self.notifying = true;
        self.callbacks += 1;
        write!(self.trace, "ON:{slot}:{};", self.show(self.slot)).unwrap();
        if self.race & 1 != 0 {
            self.heap[self.argument].count = self.heap[self.argument].count.wrapping_add(3);
        }
        if self.race & 2 != 0 {
            self.slot = self.add(2, 7, 64, 2);
        }
        if self.race & 4 != 0 {
            self.heap[self.original].count = 1;
        }
        if self.race & 8 != 0 {
            self.notify(slot);
        }
        write!(self.trace, "SAVE:{slot};").unwrap();
        self.notifying = false;
    }
    fn wrapped(&self, stack: &usize) -> bool {
        !self.empty(stack) && self.heap[*stack].key == 3
    }
}
