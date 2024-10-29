extern crate serde_json as json;

#[macro_use]
pub mod log;

pub mod page_graph;
pub mod page_storage;

pub mod explorer;

fn main() {
    explorer::explore("Pure_C");
}
