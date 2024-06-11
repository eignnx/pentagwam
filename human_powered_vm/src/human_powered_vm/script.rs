use std::{fmt, ops::ControlFlow};

use serde::{Deserialize, Serialize};

use super::{error::Result, HumanPoweredVm};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Script {
    pub lines: Vec<ScriptLine>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptLine {
    Doc(String),
    Cmd(String),
}

impl Script {
    pub fn parse(script_text: &str) -> Result<Self> {
        let mut lines = vec![];

        for line in script_text.lines() {
            if let Some(cmd) = line.strip_prefix(':') {
                lines.push(ScriptLine::Cmd(cmd.trim().to_string()));
            } else {
                lines.push(ScriptLine::Doc(line.to_string()));
            }
        }

        Ok(Self { lines })
    }

    pub fn exec(&self, hpvm: &mut HumanPoweredVm) -> Result<()> {
        for (i, line) in self.lines.iter().enumerate() {
            match line {
                ScriptLine::Doc(_) => {}
                ScriptLine::Cmd(cmd) => {
                    bunt::println!(
                        "=> {[bold+intense+italic]}\t\t(Auto-running command...)",
                        cmd
                    );
                    match hpvm.handle_cmd(cmd) {
                        Ok(ControlFlow::Continue(())) => {}
                        Ok(ControlFlow::Break(())) => {
                            println!(
                                "=> Breaking out of script command auto-run at line {}: `{cmd}`",
                                i + 1
                            );
                            return Ok(());
                        }
                        Err(e) => {
                            bunt::println!(
                                "{$red}!>{/$} Error while running script command `{[bold+intense+italic]}` at line {}:",
                                cmd,
                                i + 1
                            );
                            return Err(e);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for line in &self.lines {
            match line {
                ScriptLine::Doc(doc) => writeln!(f, "{doc}")?,
                ScriptLine::Cmd(cmd) => writeln!(f, ": {cmd}")?,
            }
        }
        Ok(())
    }
}
