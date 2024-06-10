use std::path::PathBuf;

use human_powered_vm::{error::Result, scenario::Scenario, HumanPoweredVm};
use pentagwam::cell::Functor;

pub mod human_powered_vm;
pub mod vals;

fn main() -> Result<()> {
    let mut vm = HumanPoweredVm::new()?;

    let args = std::env::args().collect::<Vec<_>>();
    let scenario: Scenario<Functor<String>> = match &args[..] {
        [_, scenario_path] => {
            let full_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(scenario_path);
            let mut file = std::fs::File::open(full_path)?;
            ron::de::from_reader(&mut file)?
        }
        _ => {
            eprintln!();
            eprintln!("Usage: human_powered_vm <scenario-file>");
            eprintln!();
            eprintln!("\tPlease provide a scenario file.");
            eprintln!();
            std::process::exit(1);
        }
    };

    vm.run_scenario(scenario)
}
