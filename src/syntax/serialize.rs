use crate::{
    cell::{Cell, Functor},
    defs::CellRef,
    mem::Mem,
};

use super::Term;

#[derive(Default, Debug)]
pub struct Serializer {
    pub term_bodies_remaining: Vec<RemainderTask>,
}

#[derive(Debug)]
pub enum RemainderTask {
    BuildRcd {
        referee_addr: CellRef,
        functor: Functor,
        args: Vec<Term>,
    },
    BuildLst {
        referee_addr: CellRef,
        car: Term,
        cdr: Term,
    },
}

impl Serializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize(&mut self, syntax: Term, mem: &mut Mem) -> CellRef {
        let start = mem.heap.len().into();
        self.term_bodies_remaining.clear();
        self.serialize_flat(syntax, mem);
        while !self.term_bodies_remaining.is_empty() {
            self.serialize_remainder(mem);
        }
        start
    }

    fn serialize_flat(&mut self, syntax: Term, mem: &mut Mem) -> CellRef {
        match syntax {
            Term::Int(i) => mem.push(Cell::Int(i)),
            Term::Sym(s) => {
                let sym = mem.intern_sym(s);
                mem.push(Cell::Sym(sym))
            }
            Term::NamedVar(v) => mem.push_var(&v),
            Term::FreshVar => mem.push_fresh_var(),
            Term::Record(functor, args) => {
                let rcd_addr = mem.push(Cell::Rcd(u32::MAX.into())); // We'll come back to this.
                let functor = mem.intern_functor(functor, args.len() as u8);
                self.term_bodies_remaining.push(RemainderTask::BuildRcd {
                    referee_addr: rcd_addr,
                    functor,
                    args,
                });
                rcd_addr
            }
            Term::Nil => mem.push(Cell::Nil),
            Term::Cons(car, cdr) => {
                let lst_addr = mem.push(Cell::Lst(u32::MAX.into())); // We'll come back to this.
                self.term_bodies_remaining.push(RemainderTask::BuildLst {
                    referee_addr: lst_addr,
                    car: *car,
                    cdr: *cdr,
                });
                lst_addr
            }
        }
    }

    fn serialize_remainder(&mut self, mem: &mut Mem) {
        let term_bodies_remaining = self.term_bodies_remaining.drain(..).collect::<Vec<_>>();
        for task in term_bodies_remaining {
            match task {
                RemainderTask::BuildRcd {
                    referee_addr,
                    functor,
                    args,
                } => {
                    let functor_addr = mem.push(Cell::Sig(functor));
                    for arg in args {
                        self.serialize_flat(arg, mem);
                    }
                    mem.cell_write(referee_addr, Cell::Rcd(functor_addr));
                }
                RemainderTask::BuildLst {
                    referee_addr,
                    car,
                    cdr,
                } => {
                    let car_addr = self.serialize_flat(car, mem);
                    let _cdr_addr = self.serialize_flat(cdr, mem);
                    mem.cell_write(referee_addr, Cell::Lst(car_addr));
                }
            }
        }
    }
}
