use std::{fmt, ops::ControlFlow};

use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

use crate::human_powered_vm::styles::{err_tok, note};

use super::{error::Result, HumanPoweredVm};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Script {
    pub sections: Vec<ScriptSection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptSection {
    Doc(String),
    Cmd(String),
}

impl Script {
    pub fn parse(script_text: &str) -> Result<Self> {
        let mut sections = vec![];

        for line in script_text.lines() {
            if line.trim() == "```" {
                match sections.last_mut() {
                    None => sections.push(ScriptSection::Cmd(String::new())),
                    Some(ScriptSection::Doc(_)) => sections.push(ScriptSection::Cmd(String::new())),
                    Some(ScriptSection::Cmd(_)) => sections.push(ScriptSection::Doc(String::new())),
                }
            } else {
                match sections.last_mut() {
                    None => sections.push(ScriptSection::Doc(line.to_owned() + "\n")),
                    Some(ScriptSection::Doc(s)) | Some(ScriptSection::Cmd(s)) => {
                        *s += line;
                        *s += "\n";
                    }
                }
            }
        }

        Ok(Self { sections })
    }

    pub fn exec(&self, hpvm: &mut HumanPoweredVm) -> Result<()> {
        for (i, section) in self.sections.iter().enumerate() {
            match section {
                ScriptSection::Doc(_) => {}
                ScriptSection::Cmd(cmds) => {
                    for cmd in cmds.lines().filter(|line| !line.trim().is_empty()) {
                        println!(
                            "=> {:<40}{:>40}",
                            cmd.bold().italic(),
                            "(Auto-running command...)".style(note()),
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
        }
        Ok(())
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for section in &self.sections {
            match section {
                ScriptSection::Doc(lines) => write!(f, "{lines}")?,
                ScriptSection::Cmd(lines) => write!(f, "```\n{lines}```")?,
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
