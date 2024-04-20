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

        let atomic = int
            .or(term.delimited_by(just('('), just(')')))
            .or(record)
            .or(sym.map(TaggedCell::Sym))
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

    // let decl = recursive(|decl| {
    //     let r#let = text::ascii::keyword("let")
    //         .ignore_then(ident)
    //         .then_ignore(just('='))
    //         .then(expr.clone())
    //         .then_ignore(just(';'))
    //         .then(decl.clone())
    //         .map(|((name, rhs), then)| Expr::Let {
    //             name,
    //             rhs: Box::new(rhs),
    //             then: Box::new(then),
    //         });

    //     let r#fn = text::ascii::keyword("fn")
    //         .ignore_then(ident)
    //         .then(ident.repeated().collect::<Vec<_>>())
    //         .then_ignore(just('='))
    //         .then(expr.clone())
    //         .then_ignore(just(';'))
    //         .then(decl)
    //         .map(|(((name, args), body), then)| Expr::Fn {
    //             name,
    //             args,
    //             body: Box::new(body),
    //             then: Box::new(then),
    //         });

    //     r#let.or(r#fn).or(expr).padded()
    // });

    // decl
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
