use owo_colors::{OwoColorize, Style};

pub fn heading() -> Style {
    Style::new().bold().underline()
}

pub fn rval() -> Style {
    Style::new().yellow().bold()
}

pub fn lval() -> Style {
    Style::new().bright_magenta()
}

pub fn val() -> Style {
    Style::new().yellow()
}

pub fn name() -> Style {
    Style::new().cyan()
}

/// For a name which doesn't refer to anything valid (incorrect variable
/// spelling, etc.)
pub fn bad_name() -> Style {
    Style::new().cyan().dimmed()
}

pub fn cell() -> Style {
    Style::new().blue()
}

pub fn valty() -> Style {
    Style::new().green()
}

pub fn instr() -> Style {
    Style::new().bold().italic()
}

pub fn bad_instr() -> Style {
    Style::new().bold().italic().dimmed()
}

pub fn note() -> Style {
    Style::new().italic().dimmed()
}

pub fn err_tok() -> owo_colors::Styled<&'static &'static str> {
    "!>".style(Style::new().bright_red())
}
