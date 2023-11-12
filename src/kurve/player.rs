use super::curve::MoveKeys;

#[derive(Debug, Default)]
pub struct Player {
    pub score: u8,
    pub name: String,
    pub move_keys: MoveKeys,
}

impl Player {
    pub fn new(name: String, move_keys: MoveKeys) -> Self {
        Self {
            score: 0,
            name,
            move_keys,
        }
    }
}
