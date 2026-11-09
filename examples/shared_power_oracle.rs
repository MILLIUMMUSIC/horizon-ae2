use ae2_horizon::{
    ae2_energy::PowerEvent,
    ae2_power::{EnergyNode, EnergyService, EnergySystem, Flow, SharedPower, Store},
};
use std::io::{self, Read};
fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();
    let mut tokens = input.split_whitespace();
    let mut next = || tokens.next().unwrap().parse::<f64>().unwrap();
    let cases = next() as usize;
    for _ in 0..cases {
        let amount = next();
        let maximum = next();
        let priority = next() as i32;
        let count = next() as usize;
        let power = SharedPower::new(amount, maximum);
        let mut system = EnergySystem::default();
        for id in 0..3 {
            system.add_service(id, EnergyService::default()).unwrap();
        }
        let mut attached = None;
        let mut share = 0.;
        for _ in 0..count {
            let op = next() as u8;
            let mut result = 0.;
            match op {
                0 => {
                    let id = next() as u64;
                    system
                        .service_mut(id)
                        .unwrap()
                        .add_node(
                            1,
                            EnergyNode {
                                idle: 1.,
                                store: Some(Store::shared(power.clone(), priority, Flow::Write)),
                                passive_rate: None,
                            },
                            share,
                        )
                        .unwrap();
                    share = 0.;
                    attached = Some(id);
                }
                1 => {
                    share = system
                        .remove_node(attached.take().unwrap(), 1)
                        .unwrap()
                        .saved_buffer;
                }
                2 => {
                    let n = next();
                    let commit = next() != 0.;
                    result = power.extract(n, commit);
                }
                3 => {
                    let n = next();
                    let commit = next() != 0.;
                    result = power.insert(n, commit);
                }
                4 => power.set_stored(next()),
                5 => {
                    let id = next() as u64;
                    let n = next();
                    let commit = next() != 0.;
                    result = system.inject(id, n, commit);
                }
                6 => {
                    let id = next() as u64;
                    let n = next();
                    let commit = next() != 0.;
                    result = system.extract(id, n, 1., commit);
                }
                7 => power.set_maximum(next()),
                8 => {
                    let id = next() as u64;
                    system.begin_tick();
                    system.end_tick(id, 0., 1.);
                }
                _ => unreachable!(),
            }
            let events = power.take_events();
            for &e in &events {
                if e == PowerEvent::Receive
                    && let Some(id) = attached
                {
                    system
                        .service_mut(id)
                        .unwrap()
                        .owner_storage_event(1, e)
                        .unwrap();
                }
            }
            print!(
                "{result:.17e} {:.17e} {:.17e} {share:.17e} {} {}",
                power.amount(),
                power.maximum(),
                attached.map_or(-1, |n| n as i32),
                events.len()
            );
            for e in events {
                print!(" {}", u8::from(e == PowerEvent::Receive));
            }
            for id in 0..3 {
                let s = system.service_mut(id).unwrap();
                s.refresh_power();
                print!(
                    " {:.17e} {:.17e} {:.17e} {}",
                    s.buffer_amount(),
                    s.maximum(),
                    s.stored_power(),
                    u8::from(s.is_powered())
                );
            }
            println!();
        }
    }
}
