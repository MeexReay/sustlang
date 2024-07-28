use std::{
    env::args,
    fs,
    io::{stdin, stdout},
};

use sustlang::{RunningScript, Script};

fn main() {
    let args: Vec<String> = args().collect();

    let filename = args[1].clone();
    let args = args[1..].to_vec();

    let script = match Script::parse(fs::read_to_string(filename).unwrap()) {
        Ok(i) => i,
        Err((e, c)) => {
            println!("error ({:?}) line: {}", e, c);
            return;
        }
    };

    let mut running_script = RunningScript::new(script);
    running_script
        .set_standard_vars(args, Box::new(stdout()), Box::new(stdin()))
        .unwrap();
    match running_script.run() {
        Ok(_) => {}
        Err((e, c)) => {
            println!("error ({:?}) command: {:?}", e, c);
        }
    };
}
