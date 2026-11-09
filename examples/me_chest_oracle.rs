#[path = "support/me_chest_fixture.rs"]
mod fixture;
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let row = serde_json::from_str(&line.unwrap()).unwrap();
        println!("{}", fixture::run(&row));
    }
}
