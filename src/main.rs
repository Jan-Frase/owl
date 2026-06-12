use crate::uci::UciInterface;

mod engine;
mod simplified_eval;
mod uci;

fn main() {
    start_uci();
}

fn start_uci() {
    let mut uci_interface = UciInterface::new();
    uci_interface.run();
}
