pub struct Identity {
    pub id: String,
    pub name: String,
}

impl Identity {
    pub fn new(id: &str, name: &str) -> Self {
        Identity {
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}
