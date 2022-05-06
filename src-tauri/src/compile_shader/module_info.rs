use itertools::Itertools;
use rspirv::dr::{Instruction, Module, Operand};
use serde::{Deserialize, Serialize};
use spirv::{Op, StorageClass, Word};
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
        let mut vector_types = HashMap::<Word, (u32, String)>::new();
        let mut constants_int32 = HashMap::new();

        for instruction in module.all_inst_iter() {
            macro_rules! resolve_name {
                ($id:expr) => {{
                    names
                        .get(&$id)
                        .cloned()
                        .unwrap_or_else(|| ($id).to_string())
                }};
            }

            match instruction.class.opcode {
                Op::String => {
                    let id = instruction.result_id.unwrap();
                    let value = instruction.operands.get(0).unwrap().unwrap_literal_string();
                    strings.insert(id, value.to_string());
                },
                Op::Name => {
                    let id = instruction.operands.get(0).unwrap().unwrap_id_ref();
                    let name = instruction.operands.get(1).unwrap().unwrap_literal_string();
                    names.insert(id, name.to_string());
                },

                // Types
                Op::TypeVoid => {
                    names.insert(instruction.result_id.unwrap(), "void".to_string());
                },
                Op::TypeBool => {
                    names.insert(instruction.result_id.unwrap(), "bool".to_string());
                },
                Op::TypeAccelerationStructureNV => {
                    names.insert(
                        instruction.result_id.unwrap(),
                        "AccelerationStructure".to_string(),
                    );
                },
                Op::TypeInt => {
                    let bits = instruction.operands.get(0).unwrap().unwrap_literal_int32();
                    let signed = instruction.operands.get(1).unwrap().unwrap_literal_int32() == 1;
                    names.insert(
                        instruction.result_id.unwrap(),
                        format!("{}{}", if signed { "i" } else { "u" }, bits),
                    );
                },
                Op::TypeFloat => {
                    let bits = instruction.operands.get(0).unwrap().unwrap_literal_int32();
                    names.insert(instruction.result_id.unwrap(), format!("f{}", bits));
                },
                Op::TypeFunction => {
                    let function_return_type = instruction.operands.get(0).unwrap().unwrap_id_ref();
                    let function_return_type_name = resolve_name!(function_return_type);

                    let argument_type_names = instruction
                        .operands
                        .iter()
                        .skip(1)
                        .map(|operand| {
                            let id = operand.unwrap_id_ref();
                            resolve_name!(id)
                        })
                        .collect_vec();

                    names.insert(
                        instruction.result_id.unwrap(),
                        format!(
                            "fn({}){}",
                            argument_type_names.join(";"),
                            function_return_type_name
                        ),
                    );
                },
                Op::TypePointer => {
                    let ptr_type = instruction.operands.get(1).unwrap().unwrap_id_ref();
                    let ptr_type_name = resolve_name!(ptr_type);

                    let storage_class = instruction.operands.get(0).unwrap().unwrap_storage_class();
                    let storage_class_name = match storage_class {
                        StorageClass::UniformConstant => "UC",
                        StorageClass::Input => "I",
                        StorageClass::Uniform => "U",
                        StorageClass::Output => "O",
                        StorageClass::Workgroup => "W",
                        StorageClass::CrossWorkgroup => "CW",
                        StorageClass::Private => "P",
                        StorageClass::Function => "F",
                        StorageClass::Generic => "G",
                        StorageClass::PushConstant => "PC",
                        StorageClass::AtomicCounter => "ACtr",
                        StorageClass::Image => "I",
                        StorageClass::StorageBuffer => "SB",
                        StorageClass::CallableDataNV => "Call",
                        StorageClass::IncomingCallableDataNV => "ICall",
                        StorageClass::RayPayloadNV => "Ray",
                        StorageClass::HitAttributeNV => "Hit",
                        StorageClass::IncomingRayPayloadNV => "IRay",
                        StorageClass::ShaderRecordBufferNV => "SRB",
                        StorageClass::PhysicalStorageBuffer => "PSB",
                        StorageClass::CodeSectionINTEL => "Code",
                    };

                    names.insert(
                        instruction.result_id.unwrap(),
                        format!("p{}*{}", storage_class_name, ptr_type_name,),
                    );
                },
                // Vector, Matrix and Array types
                Op::TypeVector => {
                    let component = instruction.operands.get(0).unwrap().unwrap_id_ref();
                    let component_name = resolve_name!(component);

                    let count = instruction.operands.get(1).unwrap().unwrap_literal_int32();

                    vector_types.insert(
                        instruction.result_id.unwrap(),
                        (count, component_name.clone()),
                    );

                    names.insert(
                        instruction.result_id.unwrap(),
                        format!("{}x{}", count, component_name),
                    );
                },
                Op::TypeMatrix => {
                    let component = instruction.operands.get(0).unwrap().unwrap_id_ref();

                    let (rows, component_name) = vector_types.get(&component).unwrap();
                    let columns = instruction.operands.get(1).unwrap().unwrap_literal_int32();

                    names.insert(
                        instruction.result_id.unwrap(),
                        format!("{}x{}x{}", rows, columns, component_name),
                    );
                },
                Op::TypeArray => {
                    let component = instruction.operands.get(0).unwrap().unwrap_id_ref();
                    let component_name = resolve_name!(component);

                    let count = instruction.operands.get(1).unwrap().unwrap_id_ref();
                    let constant_count = constants_int32.get(&count).unwrap();

                    names.insert(
                        instruction.result_id.unwrap(),
                        format!("[{};{}]", component_name, constant_count),
                    );
                },
                Op::TypeRuntimeArray => {
                    let component = instruction.operands.get(0).unwrap().unwrap_id_ref();
                    let component_name = resolve_name!(component);

                    names.insert(
                        instruction.result_id.unwrap(),
                        format!("[{}]", component_name,),
                    );
                },

                // Constants
                Op::Constant => {
                    let result_type = instruction.result_type.unwrap();
                    let result_type_name = resolve_name!(result_type);

                    let value = instruction.operands.get(0).unwrap();

                    if let Operand::LiteralInt32(v) = value {
                        constants_int32.insert(instruction.result_id.unwrap(), *v);
                    }

                    names.insert(
                        instruction.result_id.unwrap(),
                        format!("{}{}", value, result_type_name),
                    );
                },
                Op::ConstantTrue => {
                    names.insert(instruction.result_id.unwrap(), "true".to_string());
                },
                Op::ConstantFalse => {
                    names.insert(instruction.result_id.unwrap(), "false".to_string());
                },
                Op::ConstantNull => {
                    names.insert(instruction.result_id.unwrap(), "null".to_string());
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
            .map(|name| format!("%{name}"))
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
