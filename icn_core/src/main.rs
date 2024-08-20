use icn_core::node::NodeManager;

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut node = NodeManager::new();
    node.start().await;
}
