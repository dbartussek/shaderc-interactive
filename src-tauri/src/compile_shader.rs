pub mod annotated_disassembly;

use crate::compile_shader::annotated_disassembly::AnnotatedDisassembly;
use lazy_static::lazy_static;
use rspirv::dr::load_words;
use serde::{Deserialize, Serialize};
use shaderc::{CompileOptions, Compiler, EnvVersion, ShaderKind, TargetEnv};

lazy_static! {
    static ref SHADERC: Compiler = Compiler::new().unwrap();
}

#[derive(Serialize, Deserialize)]
pub enum Compilation {
    Success {
        assembly: AnnotatedDisassembly,
        warning: String,
    },
    Failure {
        error: String,
    },
}

#[tauri::command]
pub fn compile_shader(
    source: &str,
    shader_kind: &str,
    file_name: &str,
    target_env: &str,
) -> Compilation {
    let compiler: &Compiler = &SHADERC;

    let shader_kind = match shader_kind {
        "Vertex" => ShaderKind::Vertex,
        "Fragment" => ShaderKind::Fragment,
        "Geometry" => ShaderKind::Geometry,
        "TesselationControl" => ShaderKind::TessControl,
        "TesselationEvaluation" => ShaderKind::TessEvaluation,

        "RayGeneration" => ShaderKind::RayGeneration,
        "AnyHit" => ShaderKind::AnyHit,
        "ClosestHit" => ShaderKind::ClosestHit,
        "Miss" => ShaderKind::Miss,
        "Intersection" => ShaderKind::Intersection,
        "Callable" => ShaderKind::Callable,

        "Compute" => ShaderKind::Compute,

        "Task" => ShaderKind::Task,
        "Mesh" => ShaderKind::Mesh,

        _ => {
            return Compilation::Failure {
                error: format!("Unknown shader kind {shader_kind}"),
            }
        },
    };

    let mut options = CompileOptions::new().unwrap();

    match target_env {
        "Vulkan" => {
            options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2 as u32);
        },
        "OpenGL" => {
            options.set_target_env(TargetEnv::OpenGL, EnvVersion::OpenGL4_5 as u32);
            options.set_auto_map_locations(true);
            options.set_auto_bind_uniforms(true);
        },
        _ => {
            return Compilation::Failure {
                error: format!("Unknown target environment: {}", target_env),
            }
        },
    }
    // options.set_target_spirv(SpirvVersion::V1_6);

    options.set_generate_debug_info();

    let result =
        compiler.compile_into_spirv(source, shader_kind, file_name, "main", Some(&options));

    match result {
        Ok(artifact) => {
            let module = load_words(artifact.as_binary()).unwrap();

            let assembly = AnnotatedDisassembly::create(&module);

            Compilation::Success {
                assembly,
                warning: artifact.get_warning_messages(),
            }
        },
        Err(e) => Compilation::Failure {
            error: e.to_string(),
        },
    }
}
