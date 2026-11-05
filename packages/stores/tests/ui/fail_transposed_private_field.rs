//! The transposed struct must preserve private field visibility across crates.

use aether_stores::*;
use aether_stores_visibility_helper::{Item, ItemStoreExt};

fn main() {
    let store = use_store(Item::new);
    let _ = store.transpose().private_field;
}
