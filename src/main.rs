use std::{
    env::args,
    fs,
    io::{stdin, stdout},
};

use sustlang::{RunningScript, Script};

fn main() {
    let script = Script::parse(fs::read_to_string("test.sus").unwrap());
    let mut running_script = RunningScript::new(script);
    running_script.set_standard_vars(args().collect(), Box::new(stdout()), Box::new(stdin()));
    running_script.run()
}
