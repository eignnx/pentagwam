use std::collections::BTreeMap;

use chumsky::prelude::*;

use crate::{defs::CellRef, mem::Mem};

pub mod compile;
pub mod serialize;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Module {
    pub mod_name: String,
    pub predicates: BTreeMap<(String, u8), Vec<Clause>>,
}

impl Module {
    pub fn parser(mod_name: &str) -> impl Parser<char, Self, Error = Simple<char>> + '_ {
        Clause::parser_non_end_terminated()
            .repeated()
            .then_ignore(end())
            .map(move |clauses| {
                let mut predicates = BTreeMap::new();
                for clause in clauses {
                    let functor = clause.head.0.clone();
                    let arity = clause.head.1.len() as u8;
                    let key = (functor, arity);
                    predicates.entry(key).or_insert_with(Vec::new).push(clause);
                }
                Self {
                    mod_name: mod_name.to_owned(),
                    predicates,
                }
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Clause {
    pub head: (String, Vec<Term>),
    pub body: Vec<Term>,
}

impl Clause {
    pub fn parser() -> impl Parser<char, Self, Error = Simple<char>> {
        Self::parser_non_end_terminated().then_ignore(end())
    }

    pub fn parser_non_end_terminated() -> impl Parser<char, Self, Error = Simple<char>> {
        let term = Term::parser_non_end_terminated();
        term.clone()
            .try_map(|head, span| match head {
                Term::Record(functor, args) => Ok((functor, args)),
                _ => Err(Simple::custom(
                    span,
                    "Head of clause must be a compound term.",
                )),
            })
            .then(
                just(":-")
                    .padded()
                    .ignore_then(
                        term.clone()
                            .separated_by(just(',').padded())
                            .collect::<Vec<_>>(),
                    )
                    .or_not()
                    .map(Option::unwrap_or_default),
            )
            .then_ignore(just('.').padded())
            .map(move |(head, body)| Clause { head, body })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Int(i32),
    Sym(String),
    /// The name is `None` for anonymous variables (like `_`).
    Var(Option<String>),
    Record(String, Vec<Term>),
    Cons(Box<Term>, Box<Term>),
    Nil,
}

impl Term {
    pub fn parser() -> impl Parser<char, Term, Error = Simple<char>> + Clone {
        Self::parser_non_end_terminated().then_ignore(end())
    }

    pub fn parser_non_end_terminated() -> impl Parser<char, Term, Error = Simple<char>> + Clone {
        let sym = text::ident::<char, Simple<char>>().padded();

        recursive::<char, Term, _, _, _>(move |term| {
            let int = just('-')
                .labelled("negative int")
                .or_not()
                .then(text::int(10))
                .labelled("int")
                .map(|(sign, digits): (Option<_>, String)| {
                    let sign = if sign.is_some() { -1 } else { 1 };
                    let number = sign * digits.parse::<i32>().unwrap();
                    Term::Int(number)
                });

            let record = sym
                .then(
                    term.clone()
                        .separated_by(just(',').padded())
                        .allow_trailing()
                        .collect::<Vec<_>>()
                        .delimited_by(just('('), just(')')),
                )
                .map(move |(functor, args)| Term::Record(functor, args))
                .boxed();

            let var_or_sym: BoxedParser<'static, _, Term, _> = chumsky::text::ident()
                .validate(move |name: String, _span, _emit_err| {
                    let first_char = name.chars().next().unwrap();
                    if first_char.is_uppercase() || first_char == '_' {
                        if name == "_" {
                            Term::Var(None)
                        } else {
                            Term::Var(Some(name))
                        }
                    } else {
                        Term::Sym(name)
                    }
                })
                .boxed();

            // TODO: parse improper lists like `[a, b | 123]`
            let list = term
                .clone()
                .separated_by(just(',').padded())
                .delimited_by(just('['), just(']'))
                .map(|terms| {
                    terms.into_iter().rfold(Term::Nil, |cdr, car| {
                        Term::Cons(Box::new(car), Box::new(cdr))
                    })
                });

            term.delimited_by(just('('), just(')'))
                .or(int)
                .or(list)
                .or(record)
                .or(var_or_sym)
        })
        .padded()
    }

    pub fn serialize(&self, mem: &mut Mem) -> CellRef {
        serialize::Serializer::new().serialize(self.clone(), mem)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert2::{assert, let_assert};
    use test_log::test;

    #[test]
    fn test_term_parser() {
        let mut mem = Mem::new();
        let input = "f(a123, X64, _3, [], [1], [1, 2], goblin_stats(123, -99, spear))";
        let root = Term::parser().parse(input).unwrap().serialize(&mut mem);
        assert!(mem.display_term(root).to_string() == input);
    }

    #[test]
    fn test_clause_parser() {
        let input = "123 :- goblin(G), has_spear(G).";
        let_assert!(Err(_) = Clause::parser().parse(input));

        let input = "dangerous(G) :- goblin(G), has_spear(G).";
        let_assert!(Ok(clause) = Clause::parser().parse(input));
        assert!(clause.head.0 == "dangerous");
        let g = Term::Var(Some("G".to_owned()));
        assert!(clause.head.1 == vec![g.clone()]);
        assert!(
            clause.body
                == vec![
                    Term::Record("goblin".to_owned(), vec![g.clone()]),
                    Term::Record("has_spear".to_owned(), vec![g.clone()]),
                ]
        );
    }

    #[test]
    fn test_module_parser() {
        let input = r#"
            dangerous(G) :- goblin(G), has_spear(G).
            friendly(G) :- wizard(G), entity_name(G, gandalf).
            dangerous(W) :- witch_king(W).
            friendly(H) :- hobbit(H).
            friendly(E) :- elf(E), not_corrupt(E).
        "#;
        let_assert!(Ok(module) = Module::parser("test_mod").parse(input));
        assert!(module.mod_name == "test_mod");
        assert!(module.predicates.len() == 2);
        let_assert!(Some(clauses) = module.predicates.get(&("dangerous".to_owned(), 1)));
        assert!(clauses.len() == 2);
        let_assert!(Some(clauses) = module.predicates.get(&("friendly".to_owned(), 1)));
        assert!(clauses.len() == 3);
    }
}
