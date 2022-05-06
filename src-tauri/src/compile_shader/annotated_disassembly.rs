use crate::compile_shader::module_info::{
    InstructionDisassembly, InstructionDisassemblyLengths, ModuleInfo,
};
use rspirv::{binary::Disassemble, dr::Module};
use serde::{Deserialize, Serialize};
use spirv::Op;

#[derive(Clone, Serialize, Deserialize)]
pub struct AnnotatedDisassembly {
    pub header: Option<String>,
    pub instructions: Vec<AnnotatedInstruction>,
    pub lengths: InstructionDisassemblyLengths,
    pub info: ModuleInfo,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AnnotatedInstruction {
    pub line: Option<LineAnnotation>,
    pub instruction: String,
    pub disassembly: InstructionDisassembly,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LineAnnotation {
    pub file: String,
    pub line: u32,
}

impl AnnotatedDisassembly {
    pub fn create(module: &Module, limit_result_name_length: Option<usize>) -> Self {
        let info = ModuleInfo::create(module);

        let header = module.header.as_ref().map(|h| h.disassemble());

        let mut line = None;
        let mut instructions = Vec::new();

        for instruction in module.all_inst_iter() {
            let mut add_instruction = true;

            match instruction.class.opcode {
                Op::Line => {
                    line = Some(LineAnnotation {
                        file: info
                            .strings
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
                Op::Source | Op::Name => {
                    add_instruction = false;
                },
                _ => (),
            }

            if add_instruction {
                instructions.push(AnnotatedInstruction {
                    line: line.clone(),
                    instruction: instruction.disassemble(),
                    disassembly: info.disassemble_instruction(instruction),
                });
            }
        }

        let lengths = InstructionDisassemblyLengths::for_instructions(
            instructions.iter().map(|instr| &instr.disassembly),
            limit_result_name_length,
        );

        for instr in instructions.iter_mut() {
            instr.instruction = lengths.format_instruction(&instr.disassembly, false);
        }

        Self {
            header,
            instructions,
            lengths,
            info,
        }
    }
}
