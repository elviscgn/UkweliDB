use std::env;

// TODO
// init – initialise database and genesis record #10
// record append – append signed records to the ledger #11
// record verify – verify hash chain and signatures #12
// workflow load – load JSON/YAML workflow definitions #13
// workflow list – list available workflows #14
// record show – inspect records by entity or range #15
// state current – compute current entity state by replay #16
fn main() {
    let args = env::args();
    println!("{:?}", args);
}
