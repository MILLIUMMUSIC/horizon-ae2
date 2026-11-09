use ae2_horizon::ae2_storage_helper::{self as helper, Action, Context, Statistic};
use std::io::{self, BufRead};
struct Fixture {
    factor: i32,
    item: bool,
    player: bool,
    simulated: i64,
    committed: i64,
    available: f64,
    trace: String,
}
impl Context for Fixture {
    type Key = ();
    fn extract(&mut self, _: &(), amount: i64, action: Action) -> i64 {
        self.trace
            .push_str(&format!("E:{amount}:{};", action == Action::Modulate));
        if action == Action::Simulate {
            self.simulated
        } else {
            self.committed
        }
    }
    fn insert(&mut self, _: &(), amount: i64, action: Action) -> i64 {
        self.trace
            .push_str(&format!("I:{amount}:{};", action == Action::Modulate));
        if action == Action::Simulate {
            self.simulated
        } else {
            self.committed
        }
    }
    fn power(&mut self, amount: f64, action: Action) -> f64 {
        self.trace.push_str(&format!(
            "P:{:x}:{}:CONFIG;",
            amount.to_bits(),
            action == Action::Modulate
        ));
        self.available
    }
    fn amount_per_operation(&self, _: &()) -> i32 {
        self.factor
    }
    fn is_item(&self, _: &()) -> bool {
        self.item
    }
    fn statistic(&mut self, statistic: Statistic, amount: i32) {
        if self.player {
            self.trace.push_str(&format!(
                "T:{}:{amount};",
                if statistic == Statistic::Extracted {
                    "ItemsExtracted"
                } else {
                    "ItemsInserted"
                }
            ));
        }
    }
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let words: Vec<_> = line.split_whitespace().collect();
        let mut fixture = Fixture {
            factor: words[2].parse().unwrap(),
            item: words[3] == "1",
            player: words[4] == "1",
            simulated: words[6].parse().unwrap(),
            committed: words[7].parse().unwrap(),
            available: f64::from_bits(u64::from_str_radix(words[8], 16).unwrap()),
            trace: String::new(),
        };
        let mode = if words[1] == "1" {
            Action::Modulate
        } else {
            Action::Simulate
        };
        let amount = words[5].parse().unwrap();
        let result = if words[0] == "E" {
            helper::powered_extraction(&mut fixture, &(), amount, mode)
        } else {
            helper::powered_insert(&mut fixture, &(), amount, mode)
        };
        println!("{result}|{}", fixture.trace);
    }
}
