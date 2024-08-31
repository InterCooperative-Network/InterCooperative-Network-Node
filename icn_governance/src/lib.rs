pub struct Proposal {
    pub id: u32,
    pub description: String,
    pub votes_for: u32,
    pub votes_against: u32,
}

impl Proposal {
    pub fn new(id: u32, description: &str) -> Self {
        Proposal {
            id,
            description: description.to_string(),
            votes_for: 0,
            votes_against: 0,
        }
    }

    pub fn vote_for(&mut self) {
        self.votes_for += 1;
    }

    pub fn vote_against(&mut self) {
        self.votes_against += 1;
    }
}
