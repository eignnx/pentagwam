// During development:
#![allow(unreachable_code, unused, clippy::diverging_sub_expression)]

use std::collections::HashMap;

use super::{Clause, Module, Term};
use crate::{
    bc::instr::{Arg, Constant, Instr, LabelledInstr, Reg, Slot},
    cell::Functor,
    defs::Sym,
};

#[cfg(test)]
mod tests;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, PartialEq)]
pub enum Error {
    NonCallableGoalInCallPosition(Term),
    InvalidNumberOfArgumentsForPredicate {
        name: String,
        expected_len: i32,
        actual_len: usize,
    },
}

#[derive(Debug, Default)]
pub struct CompilerState {
    vars_to_regs: HashMap<String, Slot>,
    symbol_interner: HashMap<String, Sym>,
}

impl CompilerState {
    fn intern_symbol(&mut self, text: &str) -> Sym {
        if let Some(&sym) = self.symbol_interner.get(text) {
            sym
        } else {
            let sym = Sym::new(self.symbol_interner.len());
            self.symbol_interner.insert(text.to_owned(), sym);
            sym
        }
    }

    pub fn compile_module(&mut self, module: &Module, out: &mut Vec<LabelledInstr>) -> Result<()> {
        for pred in &module.predicates {
            let ((_functor, _arity), clauses) = pred;
            for clause in clauses {
                self.compile_clause(clause, out);
            }
        }
        Ok(())
    }

    pub fn compile_clause(&mut self, clause: &Clause, out: &mut Vec<LabelledInstr>) -> Result<()> {
        let (fname, params) = &clause.head;
        for (param_id, param_tm) in params.iter().enumerate() {
            let param_reg = Arg(param_id as u8);
            let _ = self.compile_param(param_tm, param_reg, out);
        }

        match &clause.body[..] {
            [] => {}
            [goal] => self.compile_single_goal_clause_body(goal, out)?,
            goals => self.compile_multi_goal_clause_body(goals, out)?,
        };

        for tm in &clause.body {
            todo!()
        }

        Ok(())
    }

    /// Use `get_*` and `unify_*` instructions to compile a parameter.
    fn compile_param(
        &mut self,
        param_tm: &Term,
        param_reg: Arg,
        out: &mut Vec<LabelledInstr>,
    ) -> Result<()> {
        match param_tm {
            Term::Int(i) => {
                out.push(Instr::GetConst(param_reg, Constant::Int(*i)).into());
                Ok(())
            }
            Term::Sym(s) => {
                let sym = self.intern_symbol(s);
                out.push(Instr::GetConst(param_reg, Constant::Sym(sym)).into());
                Ok(())
            }
            // Anonymous (fresh) variables
            Term::Var(None) => {
                out.push(Instr::GetVoid.into());
                Ok(())
            }
            Term::Var(Some(var_name)) => {
                match self.vars_to_regs.get(var_name) {
                    // If the variable has already been encountered, look up its existing
                    // slot assignment.
                    Some(existing_slot_assignment) => {
                        out.push(Instr::GetValue(*existing_slot_assignment, param_reg).into());
                        Ok(())
                    }
                    // Otherwise choose a slot and save it there.
                    None => {
                        let fresh_slot = Reg(self.vars_to_regs.len() as u8).into();
                        self.vars_to_regs.insert(var_name.clone(), fresh_slot);
                        out.push(Instr::GetVariable(fresh_slot, todo!()).into());
                        Ok(())
                    }
                }
            }
            Term::Record(functor_name, params) => {
                let functor_sym = self.intern_symbol(functor_name);
                let functor = Functor {
                    sym: functor_sym,
                    arity: params.len() as u8,
                };
                out.push(Instr::GetStructure(param_reg, functor).into());
                // for (param_id_new, param) in params.iter().enumerate() {
                //     self.compile_param(param_id, param, out);
                // }
                todo!()
            }
            Term::Cons(car, cdr) => {
                // out.push(Instr::GetList(reg).into());
                // self.compile_param(param_id, car, out);
                // self.compile_param(param_id + 1, cdr, out);
                todo!()
            }
            Term::Nil => {
                out.push(Instr::GetNil(param_reg).into());
                Ok(())
            }
        }
    }

    fn compile_single_goal_clause_body(
        &mut self,
        goal: &Term,
        out: &mut Vec<LabelledInstr>,
    ) -> Result<()> {
        match goal {
            Term::Var(_) => todo!(),
            Term::Record(name, args) => self.compile_single_goal_record(name, args, out),
            Term::Cons(_, _) | Term::Int(_) | Term::Sym(_) | Term::Nil => {
                Err(Error::NonCallableGoalInCallPosition(goal.clone()))
            }
        }
    }

    fn compile_multi_goal_clause_body(
        &self,
        goals: &[Term],
        #[allow(clippy::ptr_arg)] _out: &mut Vec<LabelledInstr>,
    ) -> Result<()> {
        todo!()
    }

    fn compile_single_goal_record(
        &mut self,
        name: &str,
        args: &[Term],
        out: &mut Vec<LabelledInstr>,
    ) -> Result<()> {
        // First: put args.
        for arg in args {
            self.put_arg(arg, out)?;
        }

        let functor = Functor {
            sym: self.intern_symbol(name),
            arity: args.len() as u8,
        };
        let functor_label: usize = self.assign_functor_label(functor);
        out.push(Instr::Execute(functor_label).into());
        for (arg_id, arg) in args.iter().enumerate() {
            self.compile_param(arg, Arg(arg_id as u8), out)?;
        }
        Ok(())
    }

    fn put_arg(&mut self, arg: &Term, out: &mut Vec<LabelledInstr>) -> Result<()> {
        match arg {
            Term::Int(i) => {
                out.push(Instr::PutConst(Constant::Int(*i), todo!()).into());
                Ok(())
            }
            Term::Sym(s) => {
                let sym = self.intern_symbol(s);
                out.push(Instr::PutConst(Constant::Sym(sym), todo!()).into());
                Ok(())
            }
            Term::Var(None) => {
                // out.push(Instr::PutVoid.into());
                Ok(())
            }
            Term::Var(Some(v)) => match self.vars_to_regs.get(v) {
                Some(slot) => {
                    // out.push(Instr::PutValue(*slot, todo!()).into());
                    Ok(())
                }
                None => {
                    let slot = Reg(self.vars_to_regs.len() as u8).into();
                    self.vars_to_regs.insert(v.clone(), slot);
                    out.push(Instr::PutVariable(slot, todo!()).into());
                    Ok(())
                }
            },
            Term::Record(_, _) => todo!(),
            Term::Cons(_, _) => todo!(),
            Term::Nil => todo!(),
        }
    }

    fn assign_functor_label(&self, functor: Functor) -> usize {
        todo!()
    }
}
