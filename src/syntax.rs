use chumsky::prelude::*;

use crate::{
    cell::{Cell, Functor},
    defs::CellRef,
    mem::Mem,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Syntax {
    Int(i32),
    Sym(String),
    NamedVar(String),
    FreshVar,
    Record(String, Vec<Syntax>),
}

impl Syntax {
    pub fn parser() -> impl Parser<char, Syntax, Error = Simple<char>> {
        let sym = text::ident::<char, Simple<char>>().padded();

        recursive::<char, Syntax, _, _, _>(move |term| {
            let int = just('-')
                .labelled("negative int")
                .or_not()
                .then(text::int(10))
                .labelled("int")
                .map(|(sign, digits): (Option<_>, String)| {
                    let sign = if sign.is_some() { -1 } else { 1 };
                    let number = sign * digits.parse::<i32>().unwrap();
                    Syntax::Int(number)
                });

            let record = sym
                .then(
                    term.clone()
                        .separated_by(just(',').padded())
                        .allow_trailing()
                        .collect::<Vec<_>>()
                        .delimited_by(just('('), just(')')),
                )
                .map(move |(functor, args)| Syntax::Record(functor, args))
                .boxed();

            let var_or_sym: BoxedParser<'static, _, Syntax, _> = chumsky::text::ident()
                .validate(move |name: String, _span, _emit_err| {
                    let first_char = name.chars().next().unwrap();
                    if first_char.is_uppercase() || first_char == '_' {
                        if name == "_" {
                            Syntax::FreshVar
                        } else {
                            Syntax::NamedVar(name)
                        }
                    } else {
                        Syntax::Sym(name)
                    }
                })
                .boxed();

            term.delimited_by(just('('), just(')'))
                .or(int)
                .or(record)
                .or(var_or_sym)
        })
        .padded()
        .then_ignore(end())
    }

    pub fn serialize(&self, mem: &mut Mem) -> CellRef {
        Serializer::new().serialize(self, mem)
    }
}

#[derive(Default, Debug)]
struct Serializer {
    term_bodies_remaining: Vec<(CellRef, TermBody)>,
}

#[derive(Debug)]
struct TermBody {
    functor: Functor,
    args: Vec<Syntax>,
}

impl Serializer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn serialize(&mut self, syntax: &Syntax, mem: &mut Mem) -> CellRef {
        let start = mem.heap.len().into();
        self.term_bodies_remaining.clear();
        self.serialize_flat(syntax, mem);
        while !self.term_bodies_remaining.is_empty() {
            self.serialize_remainder(mem);
        }
        start
    }

    fn serialize_flat(&mut self, syntax: &Syntax, mem: &mut Mem) {
        match syntax {
            Syntax::Int(i) => {
                let _ = mem.push(Cell::Int(*i));
            }
            Syntax::Sym(s) => {
                let sym = mem.intern_sym(s);
                let _ = mem.push(Cell::Sym(sym));
            }
            Syntax::NamedVar(v) => {
                let _ = mem.push_var(v);
            }
            Syntax::FreshVar => {
                let _ = mem.push_fresh_var();
            }
            Syntax::Record(functor, args) => {
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

#[cfg(test)]
use test_log::test;

#[test]
fn test_parser() {
    let mut mem = Mem::new();
    let input = "f(a123, X64, _3, goblin_stats(123, -99, spear))";
    let root = Syntax::parser().parse(input).unwrap().serialize(&mut mem);
    assert_eq!(mem.display_term(root).to_string(), input);
}
