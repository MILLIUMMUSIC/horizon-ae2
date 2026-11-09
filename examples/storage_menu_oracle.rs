#[path = "support/storage_menu_fixture.rs"]
mod fixture;
use ae2_horizon::{
    ae2_incremental_update::Updates, ae2_me_interaction::InventoryAction, ae2_storage_menu as menu,
};
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let value: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let mut fixture = fixture::Fixture::new(&value);
        let mut updates = Updates::default();
        for key in 0..5 {
            updates.get_or_assign(&key).unwrap();
        }
        let action =
            InventoryAction::from_ordinal(value["action"].as_i64().unwrap() as i32).unwrap();
        let result = menu::interact(
            &mut fixture,
            &updates,
            value["serial"].as_i64().unwrap(),
            action,
        );
        println!("{}", fixture.finish(result.is_err()));
    }
}
