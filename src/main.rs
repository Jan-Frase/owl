use crate::uci::UciInterface;

mod simplified_eval;
mod uci;

fn main() {
    start_uci();
}

fn start_uci() {
    let uci_interface = UciInterface::new();
    uci_interface.run();
}