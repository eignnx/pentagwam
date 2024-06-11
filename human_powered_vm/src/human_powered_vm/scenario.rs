use super::{error::Result, HumanPoweredVm};
use pentagwam::{bc::instr::Instr, cell::Functor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Scenario<L> {
    pub description: String,
    pub setup: Vec<String>,
    pub program: Vec<Instr<L, String>>,
}

impl HumanPoweredVm {
    // pub fn run_scenario<'a, L>(&mut self, scenario: Scenario<L>) -> Result<()>
    // where
    //     L: Deserialize<'a>,
    pub fn run_scenario(&mut self, scenario: Scenario<Functor<String>>) -> Result<()> {
        bunt::println!("{$bold+underline}SETUP:{/$}");

        for cmd in scenario.setup {
            println!();
            bunt::println!(": {[italic]}", cmd);
            match self.handle_cmd(&cmd) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error while running scenario setup command `{cmd}`:");
                    println!("{e}");
                }
            }
        }

        println!();
        bunt::println!("{$bold+underline}BEGIN SESSION:{/$}");

        self.load_program(scenario.program)
            .run::<Functor<String>, String>()
    }
}
