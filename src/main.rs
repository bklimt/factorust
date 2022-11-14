
use crate::recipe::RecipeManager;

use std::env::args;

mod error;
mod ingredient;
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
}
