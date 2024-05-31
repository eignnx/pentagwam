use super::{error::Result, HumanPoweredVm};
use pentagwam::{bc::instr::Instr, cell::Functor};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Scenario<L> {
    pub description: String,
    pub setup: Vec<String>,
    pub symbols: Vec<String>,
    pub program: Vec<Instr<L>>,
}

impl HumanPoweredVm {
    // pub fn run_scenario<'a, L>(&mut self, scenario: Scenario<L>) -> Result<()>
    // where
    //     L: Deserialize<'a>,
    pub fn run_scenario(&mut self, scenario: Scenario<Functor>) -> Result<()> {
        for sym in &scenario.symbols {
            let _ = self.intern_sym(sym);
        }

        println!("SETUP:");

        for cmd in scenario.setup {
            println!();
            println!("=> {cmd}");
            match self.handle_cmd(&cmd, &scenario.program[..]) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error while running scenario setup command `{cmd}`:");
                    println!("{e}");
                }
            }
        }

        for (idx, sym) in scenario.symbols.into_iter().enumerate() {
            let expected = idx;
            let actual = self.intern_sym(&sym).usize();
            if actual != expected {
                println!(
                    "WARNING: Expected symbol `{sym}` to have been interned at \
                    `{expected}`, but found it interned at `{actual}`. WAM code \
                    program probably won't work as intented."
                );
            }
        }

        println!();
        println!("BEGIN SESSION:");

        self.run(&scenario.program[..])
    }
}
