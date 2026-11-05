//! A downstream crate MUST NOT be able to call the `pub(crate)` field accessor.

use aether_stores::*;
use aether_stores_visibility_helper::{Item, ItemStoreExt};

fn main() {
    let store = use_store(Item::new);
    let _ = store.crate_field();
}
