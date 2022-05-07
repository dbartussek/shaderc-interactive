pub mod annotated_disassembly;
pub mod module_info;

use crate::compile_shader::annotated_disassembly::AnnotatedDisassembly;
use lazy_static::lazy_static;
use rspirv::dr::load_words;
use serde::{Deserialize, Serialize};
use shaderc::{CompileOptions, Compiler, EnvVersion, ShaderKind, SourceLanguage, TargetEnv};

lazy_static! {
    static ref SHADERC: Compiler = Compiler::new().unwrap();
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileShaderOptions {
    #[serde(default)]
    file_name: Option<String>,

    #[serde(default)]
    target_env: Option<String>,

    #[serde(default)]
    limit_result_name_length: Option<usize>,

    #[serde(default)]
    entry_point: Option<String>,
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
    options: CompileShaderOptions,
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

    let mut compile_options = CompileOptions::new().unwrap();

    let mut is_hlsl = false;

    match options
        .target_env
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Vulkan")
    {
        "Vulkan" => {
            compile_options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2 as u32);
        },
        "OpenGL" => {
            compile_options.set_target_env(TargetEnv::OpenGL, EnvVersion::OpenGL4_5 as u32);
            compile_options.set_auto_map_locations(true);
            compile_options.set_auto_bind_uniforms(true);
        },
        "HLSL" => {
            compile_options.set_source_language(SourceLanguage::HLSL);
            is_hlsl = true;
        },
        unknown => {
            return Compilation::Failure {
                error: format!("Unknown target environment: {}", unknown),
            }
        },
    }

    compile_options.set_generate_debug_info();

    let file_name = options
        .file_name
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or(if is_hlsl {
            "shader.hlsl"
        } else {
            "shader.glsl"
        });

    let result = compiler.compile_into_spirv(
        source,
        shader_kind,
        file_name,
        options
            .entry_point
            .as_ref()
            .map(|s| s.as_str())
            .filter(|_| is_hlsl)
            .unwrap_or("main"),
        Some(&compile_options),
    );

    match result {
        Ok(artifact) => {
            let module = load_words(artifact.as_binary()).unwrap();

            let assembly = AnnotatedDisassembly::create(&module, options.limit_result_name_length);

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
