use crate::error::Error;
use crate::ingredient::Ingredient;

use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Recipe {
    pub name: String,
    pub building: String,
    pub outputs: Vec<Ingredient>,
    pub inputs: Vec<Ingredient>,
}

impl Recipe {
    pub fn new() -> Recipe {
        Recipe {
            name: String::new(),
            building: String::new(),
            outputs: Vec::new(),
            inputs: Vec::new(),
        }
    }

    pub fn print(&self) {
        println!("# {}", self.name);
        println!("@ {}", self.building);
        for ingredient in self.outputs.iter() {
            println!("< {:>8.4} {}", ingredient.amount, ingredient.part);
        }
        for ingredient in self.inputs.iter() {
            println!("> {:>8.4} {}", ingredient.amount, ingredient.part);
        }
        println!("");
    }

    fn ensure_named(&mut self) {
        if self.name != "" {
            return;
        }
        if self.outputs.len() > 0 {
            self.name = self.outputs[0].part.clone();
        }
    }
}

pub struct RecipeManager {
    pub recipes: Vec<Recipe>,
}

impl RecipeManager {
    pub fn new() -> RecipeManager {
        RecipeManager {
            recipes: Vec::new(),
        }
    }

    pub fn load(&mut self, path: &str, default_building: Option<String>) -> Result<(), Error> {
        let building = default_building.unwrap_or(String::new());

        let file = match File::open(path) {
            Ok(file) => file,
            Err(error) => {
                return Err(Error::InvalidArgument(format!(
                    "unable to open file {:?}: {:?}",
                    path, error
                )))
            }
        };

        let mut r = BufReader::new(file);
        let mut recipe = Recipe::new();
        loop {
            let mut line = String::new();
            let n = r.read_line(&mut line).unwrap();
            let trimmed = line.trim();
            // println!("line: {:?}", trimmed);

            if trimmed == "" {
                if recipe.inputs.len() > 0 || recipe.outputs.len() > 0 {
                    recipe.ensure_named();
                }
                if recipe.building == "" {
                    recipe.building = building.clone();
                }

                if recipe.name != "" {
                    // println!("recipe: {:?}", recipe);
                    // recipe.print();
                    self.recipes.push(recipe);
                }
                recipe = Recipe::new();

                if n == 0 {
                    break;
                }
                continue;
            }

            let kind = trimmed.chars().nth(0).unwrap();
            match kind {
                '#' => recipe.name = String::from(trimmed[1..].trim()),
                '@' => recipe.building = String::from(trimmed[1..].trim()),
                _ => {
                    let text = match kind {
                        '<' | '>' => trimmed[1..].trim(),
                        _ => trimmed,
                    };
                    let ingredient = match Ingredient::parse(text) {
                        Ok(ingredient) => ingredient,
                        Err(error) => {
                            return Err(Error::InvalidArgument(format!(
                                "unable to parse ingredient {:?}: {:?}",
                                text, error
                            )))
                        }
                    };
                    if kind == '<' || (kind != '>' && recipe.outputs.len() == 0) {
                        recipe.outputs.push(ingredient);
                    } else {
                        recipe.inputs.push(ingredient);
                    }
                }
            };
        }
        Ok(())
    }

    pub fn print(&self) {
        for recipe in self.recipes.iter() {
            recipe.print();
        }
    }
}
