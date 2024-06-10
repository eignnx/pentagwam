use super::HumanPoweredVm;

impl HumanPoweredVm {
    pub(super) fn print_help(&self) {
        bunt::println!(
            "\
{:-^80}",
            "COMMAND DOCUMENTATION"
        );
        bunt::println!(
            "\
Commands:
  {[red]rval} <- {[magenta]lval} - Assign the value of {[red]rval} to {[magenta]lval}.
  {[magenta]lval} <- tm {[#137d2c]tm}
                   - Assign the Prolog term {[#137d2c]tm} to {[magenta]lval}.
  {[red]rval}           - Print the value of {[red]rval}.
  tm {[red]rval}        - Print the Prolog term residing in memory
                     at CellRef {[red]rval}.
  alias {[cyan+dimmed]new} -> {[cyan]old}
                   - Alias {[cyan]old} as {[cyan+dimmed]new}.
  del {[cyan]name}       - Delete the field, tmp var, or alias {[cyan]name}.
  push {[red]rval}      - Push the value of {[red]rval} onto the heap.
  fields | f       - Print all the data fields of the VM.
  list {[#db7900]slice}
                   - Print a slice of memory.
  docs | doc | d   - Print the documentation for the current
                     instruction.
  next | n         - Advance to the next instruction.
  quit | q         - Quit the program, saving any field declarations.
  help | h | ?     - Print this help message.

  Expression Language:

  L-Values: values which represent a memory location which
            can be assigned to.
    {[magenta]lval} ::= {[cyan]field} | {[cyan]tmp_var} | {[red]rval}.* | {[red]rval}[{[red]rval}]

  R-Values: expressions which can evaluate to a base value ({[yellow]val}).
    {[red]rval} ::= {[yellow]usize} | {[yellow]i32} | {[yellow]sym} | {[cyan]tmp_var} | {[cyan]field}
             | {[red]rval}.& | {[red]rval}.*
             | {[red]rval}[{[red]rval}] | {[#db7900]slice}
             | {[yellow]cell_ref} | {[blue]cell}
             | {[#c9db00]functor}

    {[yellow]val}   ::= {[yellow]usize} | {[yellow]i32} | {[yellow]sym} | {[yellow]cell_ref} | {[blue]cell}
    {[yellow]usize} ::= 0 | 1 | 2 | …
    {[yellow]i32}   ::= +0 | -0 | +1 | -1 | +2 | -2 | …

    {[#db7900]slice} ::= {[red]rval}[{[#db0071]idx};{[#db00b7]len}]

    {[#db0071]idx} ::= {[yellow]usize} | {[yellow]i32}
            | - | +              // lowest/highest+1 index
    {[#db00b7]len} ::= {[yellow]usize} | {[yellow]i32}
            | - | +              // min/max allowable length

    {[blue]cell}  ::= Int({[yellow]i32}) | Sym({[yellow]sym}) | Ref({[yellow]cell_ref})
              | Rcd({[yellow]cell_ref}) | Sig({[yellow]functor})
              | Lst({[yellow]cell_ref}) | Nil

    {[#c9db00]functor}  ::= {[red]rval}/{[red]rval}
    {[yellow]cell_ref} ::= @{[yellow]usize}
    {[cyan]field}    ::= example1 | ExAmPlE2 | …
    {[cyan]tmp_var}  ::= .example1 | .ExAmPlE2 | …
    {[yellow]sym} ::= :example1 | :ExAmPlE2 | :'example with spaces'
            | :'123' | …
",
            new = "<new>",
            old = "<old>",
            name = "<name>",
            lval = "<lval>",
            rval = "<rval>",
            tm = "<tm>",
            val = "<val>",
            usize = "<usize>",
            i32 = "<i32>",
            slice = "<slice>",
            idx = "<idx>",
            len = "<len>",
            cell = "<cell>",
            functor = "<functor>",
            cell_ref = "<cell_ref>",
            field = "<field>",
            tmp_var = "<tmp_var>",
            sym = "<sym>",
        );
        bunt::println!(" {:-<80}", "");
    }
}
