// src/federation.rs

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Federation {
    pub name: String,
    pub members: Vec<String>,
}

impl Federation {
    pub fn new(name: &str) -> Self {
        Federation {
            name: name.to_string(),
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, member: String) {
        self.members.push(member);
    }

    pub fn remove_member(&mut self, member: &str) {
        self.members.retain(|m| m != member);
    }

    pub fn list_members(&self) -> Vec<String> {
        self.members.clone()
    }
}
