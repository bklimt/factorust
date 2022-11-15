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
}

impl Part {
    pub fn new() -> Part {
        Part {
            name: String::new(),
            state: State::Solid,
            atomic: true,
            sinkable: true,
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

        println!("{}{}{} {}", s, atom, sink, self.name);
    }
}
