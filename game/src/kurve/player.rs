#[derive(Debug, Default)]
pub struct Player {
    pub score: u8,
    pub name: String,
}

impl Player {
    pub fn new(name: String) -> Self {
        Self { score: 0, name }
    }
}
