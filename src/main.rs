use crate::uci::UciInterface;

mod simplified_eval;
mod uci;
mod engine;

fn main() {
    start_uci();
}

fn start_uci() {
    let mut uci_interface = UciInterface::new();
    uci_interface.run();
}