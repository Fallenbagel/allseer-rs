use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// #[derive(Debug)]
// pub struct HashData {
//     context_map: Arc<Mutex<HashMap<u64, HashContext>>>,
// }

#[derive(Debug)]
pub struct HashContext {
    pub is_issue: bool,
}

pub struct Handler {
    pub context_map: Arc<Mutex<HashMap<u64, HashContext>>>,
}

// impl HashData {
//     pub fn new() -> Self {
//         Self {
//             context_map: Arc::new(Mutex::new(HashMap::new())),
//         }
//     }
// }

// impl Handler {
//     pub fn new() -> Self {
//         Self {
//             context_map: Arc::new(Mutex::new(HashMap::new())),
//         }
//     }
// }
