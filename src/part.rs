#[derive(Copy, Clone)]
pub enum State {
    Solid,
    Liquid,
    Gas,
}

pub struct Part {
    pub name: String,
    pub state: State,
    pub atomic: bool,
    pub sinkable: bool,
    pub score: i32,
}

impl Part {
    pub fn new() -> Self {
        Part {
            name: String::new(),
            state: State::Solid,
            atomic: true,
            sinkable: true,
            score: 1000,
        }
    }

    pub fn print(&self) {
        let s = match self.state {
            State::Solid => 'S',
            State::Liquid => 'L',
            State::Gas => 'G',
        };
        let atom = if self.atomic { '@' } else { ' ' };
        let sink = if self.sinkable { ' ' } else { '&' };

        println!("{:4} {}{}{} {}", self.score, s, atom, sink, self.name);
    }
}
