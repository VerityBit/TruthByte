use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let exit_code = truthbyte::run_cli(&args);
    std::process::exit(exit_code);
}
