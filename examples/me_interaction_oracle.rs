use ae2_horizon::ae2_me_interaction::{self as wire, Handler, InventoryAction, Packet};
use std::io::{self, BufRead};
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let w: Vec<_> = line.split_whitespace().collect();
        match w[0] {
            "P" => {
                let packet = Packet {
                    container_id: w[1].parse().unwrap(),
                    serial: w[2].parse().unwrap(),
                    action: InventoryAction::from_ordinal(w[3].parse().unwrap()).unwrap(),
                };
                println!("{}", hex(&packet.encode()));
            }
            "D" => {
                let text = w.get(1).copied().unwrap_or("");
                let bytes: Vec<_> = (0..text.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
                    .collect();
                match Packet::decode(&bytes) {
                    Ok((p, n)) => {
                        println!("OK:{}:{}:{}:{n}", p.container_id, p.serial, p.action as i32)
                    }
                    Err(e) => println!("ERR:{:?}:{}", e.kind, e.consumed),
                }
            }
            "H" => {
                struct Menu {
                    id: i32,
                    event: Option<(i64, InventoryAction)>,
                }
                impl Handler for Menu {
                    fn container_id(&self) -> i32 {
                        self.id
                    }
                    fn handle_interaction(&mut self, s: i64, a: InventoryAction) {
                        self.event = Some((s, a));
                    }
                }
                let p = Packet {
                    container_id: w[1].parse().unwrap(),
                    serial: w[2].parse().unwrap(),
                    action: InventoryAction::from_ordinal(w[3].parse().unwrap()).unwrap(),
                };
                let mut menu = Menu {
                    id: w[4].parse().unwrap(),
                    event: None,
                };
                wire::handle(p, if w[5] == "1" { Some(&mut menu) } else { None });
                println!(
                    "{}",
                    menu.event
                        .map_or_else(|| "ignored".into(), |(s, a)| format!("{s}:{}", a as i32))
                );
            }
            _ => panic!("unknown operation"),
        }
    }
}
