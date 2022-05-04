import { invoke } from '@tauri-apps/api/tauri';

export interface AnnotatedDisassembly {
    header: string | null;
    instructions: Array<AnnotatedInstruction>;
}

export interface AnnotatedInstruction {
    line: LineAnnotation | null;
    instruction: string;
}

export interface LineAnnotation {
    file: string;
    line: number;
}

export interface CompileShaderSuccessData {
    assembly: AnnotatedDisassembly;
    warning: string;
}
export type CompileShaderSuccess = { Success: CompileShaderSuccessData };
export interface CompileShaderFailureData {
    error: string;
}
export type CompileShaderFailure = { Failure: CompileShaderFailureData };

export type CompileShaderResult = CompileShaderSuccess | CompileShaderFailure;

export function compileShaderIsSuccess(
    result: CompileShaderResult,
): result is CompileShaderSuccess {
    return result.hasOwnProperty('Success');
}

export enum ShaderKindRaster {
    Vertex = 'Vertex',
    Fragment = 'Fragment',
    Geometry = 'Geometry',
    TesselationControl = 'TesselationControl',
    TesselationEvaluation = 'TesselationEvaluation',
}
export enum ShaderKindRay {
    RayGeneration = 'RayGeneration',
    AnyHit = 'AnyHit',
    ClosestHit = 'ClosestHit',
    Miss = 'Miss',
    Intersection = 'Intersection',
    Callable = 'Callable',
}
export enum ShaderKindCompute {
    Compute = 'Compute',
}
export enum ShaderKindMesh {
    Task = 'Task',
    Mesh = 'Mesh',
}

export type ShaderKind =
    | ShaderKindRaster
    | ShaderKindRay
    | ShaderKindCompute
    | ShaderKindMesh;

export enum TargetEnv {
    Vulkan = 'Vulkan',
    OpenGL = 'OpenGL',
}

export async function compileShader(
    source: string,
    shaderKind: ShaderKind,
    targetEnv: TargetEnv = TargetEnv.Vulkan,
    fileName = 'shader.glsl',
): Promise<CompileShaderResult> {
    return await invoke('compile_shader', {
        source,
        shaderKind,
        fileName,
        targetEnv,
    });
}
