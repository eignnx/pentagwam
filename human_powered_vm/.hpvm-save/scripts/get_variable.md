# Script for Instruction `get_variable`
Feel free to edit this file however you like.
Remember to use `$1`, `$2`, etc to refer to the instruction's parameters.

```r
@0[$1] <- $2.*
```

# Documentation
> This instruction represents a head argument that is an unbound variable.
> The instruction simply gets the value of register Ai and stores it in
> variable Vn.
> Vn := Ai