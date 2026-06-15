use crate::uci::UciInterface;

mod engine;
mod move_list;
mod simplified_eval;
mod transposition_table;
mod uci;

fn main() {
    start_uci();
}

fn start_uci() {
    let mut uci_interface = UciInterface::new();
    uci_interface.run();
}
