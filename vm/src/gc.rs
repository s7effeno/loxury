#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Gc(usize);

pub struct Manager {
    strings: Vec<String>,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
        }
    }
}

impl Manager {
    pub fn new_string(&mut self, s: String) -> Gc {
        self.strings.push(s);
        Gc(self.strings.len() - 1)
    }

    pub fn get_string(&mut self, s: Gc) -> &str {
        &self.strings[s.0]
    }
}
