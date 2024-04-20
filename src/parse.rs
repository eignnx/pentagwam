use std::{cell::RefCell, rc::Rc};

use chumsky::prelude::*;

use crate::{
    cell::{Cell, Functor, TaggedCell},
    defs::Sym,
    mem::Mem,
};

#[allow(clippy::let_and_return)]
pub fn parser(mem: Rc<RefCell<Mem>>) -> impl Parser<char, TaggedCell, Error = Simple<char>> {
    let mem1 = mem.clone();
    let sym = text::ident::<char, Simple<char>>()
        .padded()
        .map(move |s: String| mem1.borrow_mut().intern_sym(s));

    let mem2 = mem.clone();
    let term = recursive::<char, _, _, _, _>(move |term| {
        let int = text::int(10).map(|s: String| TaggedCell::Int(s.parse().unwrap()));

        let record = sym
            .clone()
            .then(
                term.clone()
                    .separated_by(just(','))
                    .allow_trailing()
                    .collect::<Vec<_>>()
                    .delimited_by(just('('), just(')')),
            )
            .map(move |(f, args): (Sym, Vec<TaggedCell>)| {
                let arity = args.len() as u8;
                let functor_idx = mem2
                    .borrow_mut()
                    .push(Cell::functor(Functor { sym: f, arity }));
                for arg in args {
                    mem2.borrow_mut().push(arg);
                }
                TaggedCell::Rcd(functor_idx)
            });

        let var_or_sym = chumsky::text::ident().validate(move |id: String, _span, _emit_err| {
            let mem = mem.clone();
            if id.chars().next().unwrap().is_ascii_uppercase() {
                TaggedCell::Ref(mem.borrow_mut().push_var())
            } else {
                TaggedCell::Sym(mem.borrow_mut().intern_sym(id))
            }
        });

        let atomic = int
            .or(term.delimited_by(just('('), just(')')))
            .or(record)
            .or(var_or_sym)
            .padded();

        // let op = |c| just(c).padded();
        //
        // let unary = op('-')
        //     .repeated()
        //     .foldr(atomic, |_op, rhs| Expr::Neg(Box::new(rhs)));

        // let product = unary.clone().foldl(
        //     choice((
        //         op('*').to(Expr::Mul as fn(_, _) -> _),
        //         op('/').to(Expr::Div as fn(_, _) -> _),
        //     ))
        //     .then(unary)
        //     .repeated(),
        //     |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
        // );

        // let sum = product.clone().foldl(
        //     choice((
        //         op('+').to(Expr::Add as fn(_, _) -> _),
        //         op('-').to(Expr::Sub as fn(_, _) -> _),
        //     ))
        //     .then(product)
        //     .repeated(),
        //     |lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)),
        // );

        // sum

        atomic
    });

    term
}

#[test]
fn test_parser() {
    let mem = Rc::new(RefCell::new(Mem::new()));
    let parser = parser(mem.clone());
    let input = "f(a, b, g(123, d))";
    let root = parser.parse(input).unwrap();
    match root {
        TaggedCell::Ref(idx) => {
            let mem = mem.borrow();
            // SAFETY: Assume parser returns index to tagged cell.
            let displayer = unsafe { mem.display_tagged_cell(idx) };
            assert_eq!(displayer.to_string(), input);
        }
        TaggedCell::Rcd(..) => {
            let idx = mem.borrow_mut().push(root);
            let mem = mem.borrow();
            // SAFETY: Assume parser returns index to tagged cell.
            let displayer = unsafe { mem.display_tagged_cell(idx) };
            assert_eq!(displayer.to_string(), input);
        }
        _ => panic!("expected a reference"),
    }
}
