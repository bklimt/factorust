use std::collections::{BTreeMap, HashSet};
use std::hash::{Hash, Hasher};

use crate::recipe::Recipe;

pub struct Inventory {
    parts: BTreeMap<String, f64>,
}

impl Hash for Inventory {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (name, amount) in self.parts.iter() {
            name.hash(state);
            let rounded = (amount * 10000.0).round() as u64;
            rounded.hash(state);
        }
    }
}

impl Inventory {
    pub fn new() -> Inventory {
        Inventory {
            parts: BTreeMap::new(),
        }
    }

    pub fn parts(&self) -> &BTreeMap<String, f64> {
        &self.parts
    }

    // Applies the given recipe to see if this inventroy could have been
    // created from a previous inventory. Ignores extra outputs, so long
    // as the recipe contributed something.
    pub fn apply_backwards(&mut self, recipe: &Recipe) -> Option<Inventory> {
        // How many times do we need to apply the recipe?
        let mut times = 0.0f64;
        for (part, want) in self.parts.iter() {
            if let Some(have) = recipe.outputs.get(part) {
                times = times.max(*want / *have);
            }
        }

        if times == 0.0 {
            return None;
        }

        let mut new_inv = Inventory::new();

        // Copy over all the current things that aren't generated by the recipe.
        for (part, want) in self.parts.iter() {
            match recipe.outputs.get(part) {
                Some(_) => {}
                None => {
                    new_inv.parts.insert(part.clone(), *want);
                }
            }
        }

        // Copy over all the inputs from the recipe.
        for (part, want) in recipe.inputs.iter() {
            let amount = (times * want) + self.parts.get(part).unwrap_or(&0.0);
            new_inv.parts.insert(part.clone(), amount);
        }

        Some(new_inv)
    }
}

#[cfg(test)]
mod tests {
    use super::Inventory;
    use crate::recipe::Recipe;
    use std::collections::{BTreeMap, HashMap};

    #[test]
    pub fn test_apply_backwards_constructor() {
        let mut inventory = Inventory {
            parts: BTreeMap::from([
                (String::from("ingot"), 60.0),
                (String::from("other"), 100.0),
            ]),
        };

        let recipe = Recipe {
            name: String::from("rec"),
            building: String::from("Constructor"),
            inputs: HashMap::from([(String::from("ore"), 30.0)]),
            outputs: HashMap::from([(String::from("ingot"), 30.0)]),
        };

        let actual = inventory.apply_backwards(&recipe).expect("expected answer");
        assert_eq!(actual.parts.len(), 2);
        assert_eq!(actual.parts["ore"], 60.0);
        assert_eq!(actual.parts["other"], 100.0);
    }

    #[test]
    pub fn test_apply_backwards_refinery() {
        let mut inventory = Inventory {
            parts: BTreeMap::from([
                (String::from("solution"), 50.0),
                (String::from("scrap"), 1.0),
            ]),
        };

        let recipe = Recipe {
            name: String::from("rec"),
            building: String::from("Refinery"),
            inputs: HashMap::from([
                (String::from("bauxite"), 20.0),
                (String::from("water"), 120.0),
            ]),
            outputs: HashMap::from([
                (String::from("solution"), 10.0),
                (String::from("scrap"), 100.0),
            ]),
        };

        let actual = inventory.apply_backwards(&recipe).expect("expected answer");
        assert_eq!(actual.parts.len(), 2);
        assert_eq!(actual.parts["bauxite"], 100.0);
        assert_eq!(actual.parts["water"], 600.0);
    }
}
