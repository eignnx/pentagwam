use crate::{
    cell::{Cell, Functor},
    defs::CellRef,
    mem::Mem,
};

use super::Term;

#[derive(Default, Debug)]
pub struct Serializer {
    pub term_bodies_remaining: Vec<(CellRef, TermBody)>,
}

#[derive(Debug)]
pub struct TermBody {
    pub functor: Functor,
    pub args: Vec<Term>,
}

impl Serializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize(&mut self, syntax: &Term, mem: &mut Mem) -> CellRef {
        let start = mem.heap.len().into();
        self.term_bodies_remaining.clear();
        self.serialize_flat(syntax, mem);
        while !self.term_bodies_remaining.is_empty() {
            self.serialize_remainder(mem);
        }
        start
    }

    fn serialize_flat(&mut self, syntax: &Term, mem: &mut Mem) {
        match syntax {
            Term::Int(i) => {
                let _ = mem.push(Cell::Int(*i));
            }
            Term::Sym(s) => {
                let sym = mem.intern_sym(s);
                let _ = mem.push(Cell::Sym(sym));
            }
            Term::NamedVar(v) => {
                let _ = mem.push_var(v);
            }
            Term::FreshVar => {
                let _ = mem.push_fresh_var();
            }
            Term::Record(functor, args) => {
                let rcd_addr = mem.push(Cell::Rcd(u32::MAX.into())); // We'll come back to this.
                self.term_bodies_remaining.push((
                    rcd_addr,
                    TermBody {
                        functor: mem.intern_functor(functor, args.len() as u8),
                        args: args.clone(),
                    },
                ));
            }
        }
    }

    fn serialize_remainder(&mut self, mem: &mut Mem) {
        let term_bodies_remaining = self.term_bodies_remaining.drain(..).collect::<Vec<_>>();
        for (rcd_addr, TermBody { functor, args }) in term_bodies_remaining {
            let functor_addr = mem.push(Cell::Sig(functor));
            for arg in args {
                self.serialize_flat(&arg, mem);
            }
            mem.cell_write(rcd_addr, Cell::Rcd(functor_addr));
        }
    }
}
