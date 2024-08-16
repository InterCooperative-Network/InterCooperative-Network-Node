pub mod federation;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Federation {
    pub name: String,
    pub members: Vec<String>,  // List of DAOs in this federation
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_federation_creation() {
        let federation = Federation::new("Example Federation");
        assert_eq!(federation.name, "Example Federation");
        assert!(federation.members.is_empty());
    }

    #[test]
    fn test_add_and_remove_member() {
        let mut federation = Federation::new("Test Federation");
        federation.add_member("DAO1".to_string());
        federation.add_member("DAO2".to_string());

        assert_eq!(federation.members.len(), 2);
        assert_eq!(federation.members, vec!["DAO1", "DAO2"]);

        federation.remove_member("DAO1");
        assert_eq!(federation.members.len(), 1);
        assert_eq!(federation.members, vec!["DAO2"]);
    }
}
