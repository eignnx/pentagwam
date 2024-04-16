#[repr(u64)]
pub enum Cell {
    Var(u32),
    Int(i32),
    Sym(u32),
    Functor(u32),
}
