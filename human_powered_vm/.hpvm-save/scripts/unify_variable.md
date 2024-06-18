> # unify_variable Vn
This instruction represents a head structure argument that is an
unbound variable. If the instruction is executed in "read" mode, it
simply gets the next argument from S and stores it in variable Vn. If
the instruction is executed in "write" mode, it pushes a new unbound
variable onto the heap, and stores a reference to it in variable Vn.

In read mode:

Vn := next_term(S)

In write mode:

Vn := next_term(H) = tag_ref(H)


# Script for instruction `unify_variable`
Feel free to edit this file however you like.
Remember to use `$1`, `$2`, etc to refer to the instruction's parameters.

```r
if mode == :read
    $1.* <- S.*
    S <- S[1].&
else
    push tm _
    $1 <- H
end
```