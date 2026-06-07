//! Print what `disk::probe()` sees (used by scripts/test-engine-loopback.sh).
//! Run: SDB_DEV=/dev/loopXX cargo run --example probe
fn main() {
    println!("{}", steamdualboot_lib::diagnostics_probe());
}
