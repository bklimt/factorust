use crate::inventory::Inventory;
use crate::recipe::RecipeManager;

use clap::Parser;

mod error;
mod inventory;
mod part;
mod recipe;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    recipe_path: String,

    #[arg(long)]
    default_building: Option<String>,

    #[arg(long)]
    print_parts: bool,

    #[arg(long)]
    print_recipes: bool,

    #[arg(long)]
    design: Option<String>,

    #[arg(long)]
    amount: Option<f64>,
}

fn main() {
    let args = Args::parse();

    let mut recipes = RecipeManager::new();
    match recipes.load(&args.recipe_path, args.default_building) {
        Ok(_) => (),
        Err(error) => panic!("{:?}", error),
    };

    if args.print_parts {
        recipes.print_parts();
    }

    if args.print_recipes {
        recipes.print_recipes();
    }

    if let Some(output) = args.design {
        let mut inventory = Inventory::new();
        inventory
            .parts_mut()
            .insert(output.clone(), args.amount.unwrap_or(1.0));

        let results = recipes.search(&inventory);

        for plan in results.iter() {
            println!("Plan: ");
            for (amount, recipe) in plan.steps().iter() {
                println!("  {} {}", amount, recipe);
            }
            println!("");
        }
    }
}
