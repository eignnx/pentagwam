The `get_structure` instruction starts by dereferencing A1
and checking whether it is free
- If it is free, it sets the current mode to `Mode::Write`. This makes
the rest of the `get_structure` behave like `put_structure`, and it
makes the subsequent `unify_variable` instructions behave like
`set_variable`.
- If it is bound, it sets the current mode to `Mode::Read`. This makes
the rest of the `get_structure` and the subsequent `unify_variable`
instructions do matching against the existing term, instead of
constructing a new one.

# Alternate Explanation
This instruction marks the beginning of a structure (without embedded
substructures) occurring as a head argument. The instruction gets the
value of register Ai and dereferences it. If the result is a
reference to a variable, that variable is bound to a new structure
pointer pointing at the top of the heap, and the binding is trailed
if necessary, functor F is pushed onto the heap, and execution
proceeds in "write" mode. Otherwise, if the result is a structure and
its functor is identical to functor F, the pointer S is set to point
to the arguments of the structure, and execution proceeds in "read"
mode. Otherwise, backtracking occurs.

Script for instruction `get_structure`
Feel free to edit this file however you like.
Remember to use `$1`, `$2`, etc to refer to the instruction's parameters.

```r
$1.*
.reply <- ask bound or unbound?
if .reply == :bound
    mode <- :read
    .heap_f <- $1.*[0]
    if .heap_f == $2
        S <- $1.*[1].&
    end
else
    :todo
end
next
```