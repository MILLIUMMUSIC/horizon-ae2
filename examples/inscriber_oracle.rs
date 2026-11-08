// SPDX-License-Identifier: LGPL-3.0-or-later
use ae2_horizon::ae2_inscriber::{Host, Machine, Modulation};
use std::fmt::Write;

struct Binding {
    task: bool,
    counts: [u32; 4],
    same: bool,
    result: u32,
    press: bool,
    cards: u32,
    local: f64,
    grid: f64,
    ready: bool,
    export: bool,
    pushed: bool,
    trace: String,
}
impl Binding {
    fn free(&self) -> u32 {
        if self.counts[3] == 0 || self.same {
            64u32.saturating_sub(self.counts[3])
        } else {
            0
        }
    }
}
impl Host for Binding {
    fn has_task(&mut self) -> bool {
        self.trace.push('T');
        self.task && self.counts[2] > 0
    }
    fn output_fits(&mut self) -> bool {
        self.trace.push('I');
        self.free() >= self.result
    }
    fn insert_result(&mut self) -> bool {
        self.trace.push('C');
        let accepted = self.result.min(self.free());
        self.counts[3] += accepted;
        accepted == self.result
    }
    fn consume_inputs(&mut self, _machine: &Machine) {
        if self.press {
            self.trace.push_str("AB");
            self.counts[0] = self.counts[0].saturating_sub(1);
            self.counts[1] = self.counts[1].saturating_sub(1);
        }
        self.trace.push('M');
        self.counts[2] = self.counts[2].saturating_sub(1);
    }
    fn grid_ready(&self) -> bool {
        self.ready
    }
    fn speed_cards(&self) -> u32 {
        self.cards
    }
    fn local_power(&mut self, amount: f64, commit: bool) -> f64 {
        self.trace.push(if commit { 'L' } else { 'l' });
        let extracted = if self.local.is_nan() {
            self.local
        } else {
            self.local.min(amount)
        };
        if commit {
            self.local -= extracted;
        }
        extracted
    }
    fn grid_power(&mut self, amount: f64, commit: bool) -> f64 {
        self.trace.push(if commit { 'G' } else { 'g' });
        let extracted = if self.grid.is_nan() {
            self.grid
        } else {
            self.grid.min(amount)
        };
        if commit {
            self.grid -= extracted;
        }
        extracted
    }
    fn push_result(&mut self) -> bool {
        self.trace.push('P');
        if self.pushed && self.export && self.counts[3] > 0 {
            self.counts[3] = 0;
            true
        } else {
            false
        }
    }
    fn auto_export_work(&self) -> bool {
        self.export && self.counts[3] > 0
    }
    fn mark_for_update(&mut self, _machine: &Machine) {
        self.trace.push('U');
    }
    fn save_changes(&mut self, _machine: &Machine) {
        self.trace.push('S');
    }
}

fn main() {
    use std::io::Read;
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let mut words = input.split_whitespace();
    let mut next = || words.next().unwrap();
    let cases: usize = next().parse().unwrap();
    let mut output = String::new();
    let initial = std::env::args().any(|a| a == "--initial");
    for _ in 0..cases {
        let mut machine = Machine {
            processing: next().parse().unwrap(),
            smash: next() == "1",
            final_step: next().parse().unwrap(),
        };
        let mut host = Binding {
            task: next() == "1",
            counts: std::array::from_fn(|_| next().parse().unwrap()),
            same: next() == "1",
            result: next().parse().unwrap(),
            press: next() == "1",
            cards: next().parse().unwrap(),
            local: f64::from_bits(next().parse().unwrap()),
            grid: f64::from_bits(next().parse().unwrap()),
            ready: next() == "1",
            export: next() == "1",
            pushed: next() == "1",
            trace: String::new(),
        };
        let _elapsed: i32 = next().parse().unwrap();
        let callbacks: usize = next().parse().unwrap();
        if initial {
            let sleeping = machine.sleeping_request(&mut host);
            writeln!(output, "{} {}", u8::from(sleeping), machine.processing).unwrap();
        }
        for _ in 0..callbacks {
            host.trace.clear();
            let modulation = machine.tick(&mut host);
            writeln!(
                output,
                "{} {} {} {} {} {} {} {} {} {} {}",
                machine.processing,
                u8::from(machine.smash),
                machine.final_step,
                host.counts[0],
                host.counts[1],
                host.counts[2],
                host.counts[3],
                host.local.to_bits(),
                host.grid.to_bits(),
                match modulation {
                    Modulation::Urgent => 0,
                    Modulation::Slower => 1,
                    Modulation::Sleep => 2,
                },
                host.trace
            )
            .unwrap();
        }
    }
    print!("{output}");
}
