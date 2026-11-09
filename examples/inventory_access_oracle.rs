#[path = "support/inventory_access_fixture.rs"]
mod fixture;
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        println!("{}", fixture::Fixture::run(&line.unwrap()));
    }
}
