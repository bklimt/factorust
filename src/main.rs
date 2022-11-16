use crate::inventory::Inventory;
use crate::recipe::RecipeManager;

use std::env::args;

mod error;
mod inventory;
mod part;
mod recipe;

fn main() {
    let path = args().nth(1).expect("usage: factorust <path>");
    let default_building = args().nth(2);

    let mut recipes = RecipeManager::new();
    match recipes.load(&path, default_building) {
        Ok(_) => (),
        Err(error) => panic!("{:?}", error),
    };

    recipes.print();
    println!("");

    let mut inventory = Inventory::new();
    inventory
        .parts_mut()
        .insert(String::from("Heavy Modular Frame"), 1.0);

    let results = recipes.search(&inventory);

    for plan in results.iter() {
        println!("Plan: ");
        for (amount, recipe) in plan.steps().iter() {
            println!("  {} {}", amount, recipe);
        }
        println!("");
    }
}
