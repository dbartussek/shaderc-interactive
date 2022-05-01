use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use shaderc::{CompileOptions, Compiler, EnvVersion, ShaderKind, TargetEnv};

lazy_static! {
    static ref SHADERC: Compiler = Compiler::new().unwrap();
}

#[derive(Serialize, Deserialize)]
pub enum Compilation {
    Success { assembly: String, warning: String },
    Failure { error: String },
}

#[tauri::command]
pub fn compile_shader(source: &str, shader_kind: &str, file_name: &str) -> Compilation {
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

    options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2 as u32);
    // options.set_target_spirv(SpirvVersion::V1_6);

    options.set_generate_debug_info();

    let result = compiler.compile_into_spirv_assembly(
        source,
        shader_kind,
        file_name,
        "main",
        Some(&options),
    );

    match result {
        Ok(artifact) => {
            let assembly = artifact.as_text();

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
