use owo_colors::OwoColorize;

use super::HumanPoweredVm;
use crate::human_powered_vm::styles::{bad_name, cell, lval, name, rval, val};

impl HumanPoweredVm {
    pub(super) fn print_help(&self) {
        println!("{:-^80}", "COMMAND DOCUMENTATION");
        println!(
            "\
Commands:
  {rval} <- {lval} - Assign the value of {rval} to {lval}.
  {lval} <- tm {tm}
                   - Assign the Prolog term {tm} to {lval}.
  {rval}           - Print the value of {rval}.
  tm {rval}        - Print the Prolog term residing in memory
                     at CellRef {rval}.
  alias {new} -> {old}
                   - Alias {old} as {new}.
  del {name}       - Delete the field, tmp var, or alias {name}.
  push {rval}      - Push the value of {rval} onto the heap.
  fields | f       - Print all the data fields of the VM.
  list {slice}
                   - Print a slice of memory.
  docs | doc | d   - Print the documentation for the current
                     instruction.
  next | n         - Advance to the next instruction.
  quit | q         - Quit the program, saving any field declarations.
  help | h | ?     - Print this help message.

  Expression Language:

  L-Values: values which represent a memory location which
            can be assigned to.
    {lval} ::= {field} | {tmp_var} | {rval}.* | {rval}[{rval}]

  R-Values: expressions which can evaluate to a base value ({val}).
    {rval} ::= {usize} | {i32} | {sym} | {tmp_var} | {field}
             | {rval}.& | {rval}.*
             | {rval}[{rval}] | {slice}
             | {cell_ref} | {cell}
             | {functor}

    {val}   ::= {usize} | {i32} | {sym} | {cell_ref} | {cell}
    {usize} ::= 0 | 1 | 2 | …
    {i32}   ::= +0 | -0 | +1 | -1 | +2 | -2 | …

    {slice} ::= {rval}[{idx};{len}]

    {idx} ::= {usize} | {i32}
            | - | +              // lowest/highest+1 index
    {len} ::= {usize} | {i32}
            | - | +              // min/max allowable length

    {cell}  ::= Int({i32}) | Sym({sym}) | Ref({cell_ref})
              | Rcd({cell_ref}) | Sig({functor})
              | Lst({cell_ref}) | Nil

    {functor}  ::= {rval}/{rval}
    {cell_ref} ::= @{usize}
    {field}    ::= example1 | ExAmPlE2 | …
    {tmp_var}  ::= .example1 | .ExAmPlE2 | …
    {sym} ::= :example1 | :ExAmPlE2 | :'example with spaces'
            | :'123' | …
",
            new = "<new>".style(bad_name()),
            old = "<old>".style(name()),
            name = "<name>".style(name()),
            lval = "<lval>".style(lval()),
            rval = "<rval>".style(rval()),
            tm = "<tm>".bright_green(),
            val = "<val>".style(val()),
            usize = "<usize>".style(val()),
            i32 = "<i32>".style(val()),
            slice = "<slice>".style(val()),
            idx = "<idx>".style(val()),
            len = "<len>".style(val()),
            cell = "<cell>".style(cell()),
            functor = "<functor>".style(val()),
            cell_ref = "<cell_ref>".style(val()),
            field = "<field>".style(name()),
            tmp_var = "<tmp_var>".style(name()),
            sym = "<sym>".style(val()),
        );
        println!(" {:-<80}", "");
    }
}
