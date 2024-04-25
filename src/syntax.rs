use chumsky::prelude::*;

use crate::{defs::CellRef, mem::Mem};

pub mod serialize;

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
        serialize::Serializer::new().serialize(self, mem)
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
