use crate::error::Error;
use crate::inventory::Inventory;
use crate::part::{Part, State};

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Recipe {
    pub name: String,
    pub building: String,
    pub outputs: HashMap<String, f64>,
    pub inputs: HashMap<String, f64>,
    pub score: i32,
}

impl Recipe {
    pub fn new() -> Recipe {
        Recipe {
            name: String::new(),
            building: String::new(),
            outputs: HashMap::new(),
            inputs: HashMap::new(),
            score: 1000,
        }
    }

    pub fn print(&self) {
        println!("# {} ({})", self.name, self.score);
        println!("@ {}", self.building);
        for (part, amount) in self.outputs.iter() {
            println!("< {:>8.4} {}", amount, part);
        }
        for (part, amount) in self.inputs.iter() {
            println!("> {:>8.4} {}", amount, part);
        }
        println!("");
    }

    fn ensure_named(&mut self) -> Result<(), Error> {
        if self.name != "" {
            return Ok(());
        }
        if self.outputs.len() != 1 {
            return Err(Error::InvalidArgument(String::from("no name for recipe")));
        }
        for (part, _) in self.outputs.iter() {
            self.name = part.clone();
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct Plan {
    steps: Vec<(f64, String)>,
}

impl Plan {
    pub fn steps(&self) -> &Vec<(f64, String)> {
        &self.steps
    }
}

pub struct RecipeManager {
    pub recipes: HashMap<String, Recipe>,
    pub parts: BTreeMap<String, Part>,
    pub sorted_recipes: BTreeMap<(i32, String), String>,
}

impl RecipeManager {
    pub fn new() -> Self {
        RecipeManager {
            recipes: HashMap::new(),
            parts: BTreeMap::new(),
            sorted_recipes: BTreeMap::new(),
        }
    }

    fn parse_ingredient(s: &str) -> Result<(String, f64), Error> {
        let s = s.trim();
        let i = match s.find(' ') {
            Some(i) => i,
            None => return Err(Error::InvalidArgument(String::from("malformed ingredient"))),
        };
        let (amount_part, part) = s.split_at(i);
        let amount = match amount_part.parse::<f64>() {
            Ok(f) => f,
            Err(error) => {
                return Err(Error::InvalidArgument(format!(
                    "invalid number {:?}: {:?}",
                    amount_part, error
                )))
            }
        };
        Ok((String::from(part.trim()), amount))
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
                    recipe.ensure_named()?;
                }
                if recipe.building == "" {
                    recipe.building = building.clone();
                }

                if recipe.name != "" {
                    // TODO(klimt): Verify it doesn't exist already.
                    if self.recipes.contains_key(&recipe.name) {
                        return Err(Error::InvalidArgument(format!(
                            "duplicate recipes named {:?}",
                            &recipe.name
                        )));
                    }
                    self.recipes.insert(recipe.name.clone(), recipe);
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
                    let (part, amount) = match Self::parse_ingredient(text) {
                        Ok((part, amount)) => (part, amount),
                        Err(error) => {
                            return Err(Error::InvalidArgument(format!(
                                "unable to parse ingredient {:?}: {:?}",
                                text, error
                            )))
                        }
                    };
                    if kind == '<' || (kind != '>' && recipe.outputs.len() == 0) {
                        recipe.outputs.insert(part, amount);
                    } else {
                        recipe.inputs.insert(part, amount);
                    }
                }
            };
        }
        self.derive_parts();
        Ok(())
    }

    fn derive_parts(&mut self) {
        // Find the atomic items and the liquids/gasses.
        for (_, recipe) in self.recipes.iter() {
            for (name, _amount) in recipe.inputs.iter() {
                if !self.parts.contains_key(name) {
                    let mut part = Part::new();
                    part.name = name.clone();
                    self.parts.insert(name.clone(), part);
                }
            }
            for (name, _amount) in recipe.outputs.iter() {
                if !self.parts.contains_key(name) {
                    let mut part = Part::new();
                    part.name = name.clone();
                    self.parts.insert(name.clone(), part);
                }
                self.parts.get_mut(name).unwrap().atomic = false;
            }
            if recipe.building == "Packager" && recipe.inputs.len() == 2 {
                if recipe.inputs.contains_key("Empty Canister") {
                    for (name, _) in recipe.inputs.iter() {
                        if name != "Empty Canister" {
                            self.parts.get_mut(name).unwrap().state = State::Liquid;
                        }
                    }
                }
                if recipe.inputs.contains_key("Empty Fluid Tank") {
                    for (name, _) in recipe.inputs.iter() {
                        if name != "Empty Fluid Tank" {
                            self.parts.get_mut(name).unwrap().state = State::Gas;
                        }
                    }
                }
            }
        }

        // Compute scores.

        // Mark the atomics.
        for (_, part) in self.parts.iter_mut() {
            if part.atomic {
                part.score = 0;
            }
        }

        // Derive scores for parts.
        loop {
            let mut change = false;
            for (_, recipe) in self.recipes.iter() {
                let mut max_input = 0;
                for (name, _) in recipe.inputs.iter() {
                    max_input = max_input.max(self.parts[name].score);
                }
                max_input += 1;
                for (name, _) in recipe.outputs.iter() {
                    let part = self.parts.get_mut(name).unwrap();
                    if max_input < part.score {
                        part.score = max_input;
                        change = true;
                    }
                }
            }
            if !change {
                break;
            }
        }

        // Derive scores for recipes.
        for (_, recipe) in self.recipes.iter_mut() {
            let mut max_input = 0;
            for (name, _) in recipe.inputs.iter() {
                max_input = max_input.max(self.parts[name].score + 1);
            }
            recipe.score = max_input;
            self.sorted_recipes
                .insert((recipe.score, recipe.name.clone()), recipe.name.clone());
        }
    }

    pub fn is_atomic(&self, inventory: &Inventory) -> bool {
        for (name, _) in inventory.parts() {
            if let Some(part) = self.parts.get(name) {
                if !part.atomic {
                    return false;
                }
            }
        }
        true
    }

    pub fn print_parts(&self) {
        for (_, part) in self.parts.iter() {
            part.print();
        }
    }

    pub fn print_recipes(&self) {
        for (_, name) in self.sorted_recipes.iter() {
            let recipe = self.recipes.get(name).expect("missing recipe");
            recipe.print();
        }
    }

    pub fn search(&self, inventory: &Inventory) -> Vec<Plan> {
        let plan = Plan { steps: Vec::new() };
        let seen: Vec<&Inventory> = Vec::new();
        let mut results: Vec<Plan> = Vec::new();
        self.search_internal(&inventory, &plan, &seen, &mut results, 0);
        results
    }

    pub fn search_internal(
        &self,
        inventory: &Inventory,
        plan: &Plan,
        seen: &Vec<&Inventory>,
        results: &mut Vec<Plan>,
        depth: i32,
    ) {
        // Check if we've seen this inventory before.
        for prev_inv in seen.iter() {
            if prev_inv.is_subset(inventory) {
                return;
            }
        }
        let mut new_seen = seen.clone();
        new_seen.push(inventory);

        for _ in 0..depth {
            print!("  ");
        }
        println!("{:?}", inventory);
        if self.is_atomic(inventory) {
            results.push(plan.clone());
            return;
        }
        for (_, recipe_name) in self.sorted_recipes.iter() {
            let recipe = self.recipes.get(recipe_name).expect("missing recipe");
            if let Some((times, new_inv)) = inventory.apply_backwards(recipe) {
                for _ in 0..depth {
                    print!("  ");
                }
                println!("{}: Applying {} {} times.", depth, recipe.name, times);
                let mut new_plan = plan.clone();
                new_plan.steps.push((times, recipe.name.clone()));
                self.search_internal(&new_inv, &new_plan, &new_seen, results, depth + 1);
            }
        }
    }
}
