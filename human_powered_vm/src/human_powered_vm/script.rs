use std::{fmt, ops::ControlFlow};

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::human_powered_vm::styles::err_tok;

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
                    println!(
                        "=> {}{:>40}",
                        cmd.bold().italic(),
                        "(Auto-running command...)"
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
                            println!(
                                "{} Error while running script command `{}` at line {}:",
                                err_tok(),
                                cmd.bold().italic(),
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

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub static EDITORS_AVAILABLE: &[(&str, &[&str])] = &[
    (
        "CLI editors",
        &[
            "sensible-editor",
            "nano",
            "pico",
            "vim",
            "nvim",
            "vi",
            "emacs",
        ],
    ),
    ("GUI editors", &["code", "atom", "subl", "gedit", "gvim"]),
    (
        "Generic \"file openers\"",
        &["xdg-open", "gnome-open", "kde-open"],
    ),
];

#[cfg(target_os = "macos")]
pub static EDITORS_AVAILABLE: &[(&str, &[&str])] = &[
    (
        "CLI editors",
        &["nano", "pico", "vim", "nvim", "vi", "emacs", "open -Wt"],
    ),
    (
        "GUI editors",
        &["code -w", "atom -w", "subl -w", "gvim", "mate"],
    ),
    (
        "Generic \"file openers\"",
        &["open -a TextEdit", "open -a TextMate", "open"],
    ),
];

#[cfg(target_os = "windows")]
pub static EDITORS_AVAILABLE: &[(&str, &[&str])] = &[
    (
        "GUI editors",
        &[
            "code.cmd -n -w",
            "atom.exe -w",
            "subl.exe -w",
            "notepad.exe",
        ],
    ),
    ("Generic \"file openers\"", &["cmd.exe /C start"]),
];
