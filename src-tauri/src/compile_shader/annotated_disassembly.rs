use rspirv::{binary::Disassemble, dr::Module};
use serde::{Deserialize, Serialize};
use spirv::Op;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct AnnotatedDisassembly {
    pub header: Option<String>,
    pub instructions: Vec<AnnotatedInstruction>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AnnotatedInstruction {
    pub line: Option<LineAnnotation>,
    pub instruction: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LineAnnotation {
    pub file: String,
    pub line: u32,
}

impl AnnotatedDisassembly {
    pub fn create(module: &Module) -> Self {
        let header = module.header.as_ref().map(|h| h.disassemble());

        let mut line = None;
        let mut instructions = Vec::new();
        let mut strings = HashMap::new();

        for instruction in module.all_inst_iter() {
            let mut add_instruction = true;

            match instruction.class.opcode {
                // Track strings so we can use them for file names
                Op::String => {
                    let id = instruction.result_id.unwrap();
                    let value = instruction.operands.get(0).unwrap().unwrap_literal_string();
                    strings.insert(id, value);
                },
                Op::Line => {
                    line = Some(LineAnnotation {
                        file: strings
                            .get(&instruction.operands.get(0).unwrap().unwrap_id_ref())
                            .unwrap()
                            .to_string(),
                        line: instruction.operands.get(1).unwrap().unwrap_literal_int32(),
                    });
                    add_instruction = false;
                },
                Op::Function => {
                    line = None;
                },
                Op::Source => {
                    add_instruction = false;
                },
                _ => (),
            }

            if add_instruction {
                instructions.push(AnnotatedInstruction {
                    line: line.clone(),
                    instruction: instruction.disassemble(),
                });
            }
        }

        Self {
            header,
            instructions,
        }
    }
}
