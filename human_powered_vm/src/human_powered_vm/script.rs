use std::{fmt, fs, io, ops::ControlFlow, path::PathBuf};

use pentagwam::bc::instr::InstrName;
use serde::{Deserialize, Serialize};

use super::{error::Result, HumanPoweredVm, SCRIPTS_DIR};
use crate::human_powered_vm::styles::{err_tok, note};

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
            if line.starts_with("```") {
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
        use owo_colors::OwoColorize;

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
                ScriptSection::Cmd(lines) => write!(f, "```r\n{lines}```")?,
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

impl HumanPoweredVm {
    pub fn script_file(instr_name: InstrName) -> PathBuf {
        Self::save_dir_location()
            .join(SCRIPTS_DIR)
            .join(instr_name.to_string() + ".md")
    }

    pub fn script_file_exists(instr_name: InstrName) -> bool {
        !fs::File::open(Self::script_file(instr_name))
            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
    }

    pub fn read_script_file(&self, instr_name: InstrName) -> io::Result<Option<String>> {
        match fs::read_to_string(Self::script_file(instr_name)) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e),
            Ok(content) => Ok(Some(content)),
        }
    }

    pub fn write_script_file(&self, instr_name: InstrName, content: &str) -> io::Result<()> {
        use io::Write;
        let mut f = fs::File::create(Self::script_file(instr_name))?;
        write!(f, "{content}")?;
        f.sync_data()?;
        Ok(())
    }

    pub fn delete_script_file(&self, instr_name: InstrName) -> io::Result<String> {
        let script_file = Self::script_file(instr_name);
        let content = std::fs::read_to_string(&script_file)?;
        fs::remove_file(&script_file)?;
        Ok(content)
    }
}
