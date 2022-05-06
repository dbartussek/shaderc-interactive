use itertools::Itertools;
use rspirv::dr::{Instruction, Module, Operand};
use serde::{Deserialize, Serialize};
use spirv::{Op, Word};
use std::collections::HashMap;

#[derive(Clone, Serialize, Deserialize)]
pub struct ModuleInfo {
    pub strings: HashMap<Word, String>,
    pub names: HashMap<Word, String>,
}

impl ModuleInfo {
    pub fn create(module: &Module) -> Self {
        let mut strings = HashMap::new();
        let mut names = HashMap::new();

        for instruction in module.all_inst_iter() {
            match instruction.class.opcode {
                Op::String => {
                    let id = instruction.result_id.unwrap();
                    let value = instruction.operands.get(0).unwrap().unwrap_literal_string();
                    strings.insert(id, value.to_string());
                },
                Op::Name => {
                    let id = instruction.operands.get(0).unwrap().unwrap_id_ref();
                    let name = instruction.operands.get(1).unwrap().unwrap_literal_string();
                    names.insert(id, format!("%{}", name));
                },
                _ => (),
            }
        }

        let names = names
            .iter()
            .map(|(id, name)| {
                let id = *id;
                let mut name = name.clone();

                let collision = names
                    .iter()
                    .find(|(collision_id, collision_name)| {
                        *collision_name == &name && id != **collision_id
                    })
                    .is_some();
                if collision {
                    name = format!("{name}_{id}");
                }

                (id, name)
            })
            .collect();

        Self { strings, names }
    }

    pub fn operand_name(&self, operand: Word) -> String {
        self.names
            .get(&operand)
            .cloned()
            .unwrap_or_else(|| format!("%{operand}"))
    }

    pub fn disassemble_instruction(&self, instruction: &Instruction) -> InstructionDisassembly {
        let result = instruction
            .result_id
            .map(|result| self.operand_name(result));
        let result_type = instruction
            .result_type
            .map(|result| self.operand_name(result));

        let name = format!("Op{}", instruction.class.opname);
        let operands = instruction
            .operands
            .iter()
            .map(|operand| match operand {
                Operand::IdRef(id) => self.operand_name(*id),
                _ => operand.to_string(),
            })
            .collect_vec();

        InstructionDisassembly {
            result,
            result_type,
            name,
            operands,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InstructionDisassembly {
    pub result: Option<String>,
    pub result_type: Option<String>,
    pub name: String,
    pub operands: Vec<String>,
}


#[derive(Clone, Serialize, Deserialize, Default)]
pub struct InstructionDisassemblyLengths {
    pub result: usize,
    pub result_type: usize,
    pub name: usize,
    pub operands: Vec<usize>,
}

impl InstructionDisassemblyLengths {
    pub fn for_instructions<'lt, It>(iterator: It) -> Self
    where
        It: IntoIterator<Item = &'lt InstructionDisassembly>,
    {
        let mut lengths = Self::default();

        for instr in iterator {
            lengths.result = lengths
                .result
                .max(instr.result.as_ref().map(|s| s.len()).unwrap_or(0));
            lengths.result_type = lengths
                .result_type
                .max(instr.result_type.as_ref().map(|s| s.len()).unwrap_or(0));

            lengths.name = lengths.name.max(instr.name.len());

            if lengths.operands.len() < instr.operands.len() {
                lengths.operands.resize(instr.operands.len(), 0);
            }

            for (operand_length, operand) in lengths.operands.iter_mut().zip(instr.operands.iter())
            {
                *operand_length = (*operand_length).max(operand.len());
            }
        }

        lengths
    }

    pub fn format_instruction(
        &self,
        instruction: &InstructionDisassembly,
        pad_operands: bool,
    ) -> String {
        let Self {
            result: result_len,
            result_type: mut result_type_len,
            name: mut name_len,
            operands: operands_len,
        } = self;

        let operands = if pad_operands {
            operands_len
                .iter()
                .zip(instruction.operands.iter())
                .map(|(length, operand)| format!("{:length$}", operand))
                .join(" ")
        } else {
            result_type_len = 0;
            name_len = 0;

            instruction.operands.iter().join(" ")
        };

        let result = instruction
            .result
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");
        let result_type = instruction
            .result_type
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("");
        let name = &instruction.name;

        format!("{result:result_len$} {name:name_len$} {result_type:result_type_len$} {operands}")
    }
}
