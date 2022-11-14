use crate::error::Error;

#[derive(Debug)]
pub struct Ingredient {
    pub amount: f32,
    pub part: String,
}

impl Ingredient {
    pub fn parse(s: &str) -> Result<Ingredient, Error> {
        let s = s.trim();
        let i = match s.find(' ') {
            Some(i) => i,
            None => return Err(Error::InvalidArgument(String::from("malformed ingredient"))),
        };
        let (amount_part, part) = s.split_at(i);
        let amount = match amount_part.parse::<f32>() {
            Ok(f) => f,
            Err(error) => {
                return Err(Error::InvalidArgument(format!(
                    "invalid number {:?}: {:?}",
                    amount_part, error
                )))
            }
        };
        Ok(Ingredient {
            amount,
            part: String::from(part.trim()),
        })
    }
}
