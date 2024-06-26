#![allow(unused)]
//! Fields that are automatically updated by the VM.

use pentagwam::defs::CellRef;

use crate::{
    human_powered_vm::{FieldData, HumanPoweredVm},
    vals::{val::Val, valty::ValTy},
};

use super::SaveData;

impl HumanPoweredVm {
    pub(super) fn update_builtin_fields(&mut self) {
        *self.heap_ptr_mut() = (self.mem.heap.len() - 1).into();
    }

    #[track_caller]
    pub fn instr_ptr(&self) -> usize {
        self.save
            .fields
            .get("instr_ptr")
            .expect("builtin `instr_ptr` field not found")
            .value
            .try_as_usize(&self.mem)
            .expect("builtin `instr_ptr` field is not a usize")
    }

    #[track_caller]
    pub fn instr_ptr_mut(&mut self) -> &mut usize {
        let Val::Usize(ref mut u) = self
            .save
            .fields
            .get_mut("instr_ptr")
            .expect("builtin `instr_ptr` field not found")
            .value
        else {
            panic!("builtin `instr_ptr` field is not a usize")
        };
        u
    }

    #[track_caller]
    pub(super) fn heap_ptr(&self) -> CellRef {
        self.save
            .fields
            .get("heap_ptr")
            .expect("builtin `heap_ptr` field not found")
            .value
            .try_as_cell_ref(&self.mem)
            .expect("builtin `heap_ptr` field is not a CellRef")
    }

    #[track_caller]
    pub(super) fn heap_ptr_mut(&mut self) -> &mut CellRef {
        let Val::CellRef(ref mut u) = self
            .save
            .fields
            .get_mut("heap_ptr")
            .expect("builtin `heap_ptr` field not found")
            .value
        else {
            panic!("builtin `heap_ptr` field is not a Usize")
        };
        u
    }
}

impl SaveData {
    pub(super) fn setup_builtin_fields(&mut self) {
        // Instruction pointer
        self.fields.insert(
            "instr_ptr".to_owned(),
            FieldData {
                value: Val::Usize(0),
                ty: ValTy::Usize,
                default: Some(Val::Usize(0)),
                aliases: ["ip", "P"].into_iter().map(ToOwned::to_owned).collect(),
            },
        );

        // Heap pointer
        self.fields.insert(
            "heap_ptr".to_owned(),
            FieldData {
                value: Val::CellRef(0.into()),
                ty: ValTy::CellRef,
                default: None,
                aliases: ["hp", "H"].into_iter().map(ToOwned::to_owned).collect(),
            },
        );
    }
}
