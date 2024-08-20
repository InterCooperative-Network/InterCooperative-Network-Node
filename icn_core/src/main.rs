use icn_core::node::NodeManager;

#[tokio::main]
async fn main() {
    static INIT: std::sync::Once = std::sync::Once::new(); INIT.call_once(|| { env_logger::init(); });
    let mut node = NodeManager::new();
    node.start().await;
}
