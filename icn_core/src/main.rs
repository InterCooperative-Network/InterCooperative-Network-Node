use icn_core::node::NodeManager;

fn main() {
    println!("Starting ICN Node...");
    let mut node = NodeManager::new();
    node.start();
}
