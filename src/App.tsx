import React, { useRef, useState } from 'react';
import './App.css';
import {
    AnnotatedDisassembly,
    compileShader,
    compileShaderIsSuccess,
    LineAnnotation,
    ShaderKind,
    ShaderKindCompute,
    ShaderKindMesh,
    ShaderKindRaster,
    ShaderKindRay,
} from './lib/shaderc';
import Editor, { Monaco } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';

const DEFAULT_SHADER = `#version 460
#extension GL_EXT_ray_tracing : require

#define M_PI 3.1415926535897932384626433832795

layout(location = 0) rayPayloadEXT Payload {
  vec3 rayOrigin;
  vec3 rayDirection;
  vec3 previousNormal;

  vec3 directColor;
  vec3 indirectColor;
  int rayDepth;

  int rayActive;
}
payload;

layout(binding = 0, set = 0) uniform accelerationStructureEXT topLevelAS;
layout(binding = 1, set = 0) uniform Camera {
  vec4 position;
  vec4 right;
  vec4 up;
  vec4 forward;

  uint frameCount;
}
camera;

layout(binding = 4, set = 0, rgba32f) uniform image2D image;

float random(vec2 uv, float seed) {
  return fract(sin(mod(dot(uv, vec2(12.9898, 78.233)) + 1113.1 * seed, M_PI)) *
               43758.5453);
  ;
}

void main() {
  vec2 uv = gl_LaunchIDEXT.xy +
            vec2(random(gl_LaunchIDEXT.xy, 0), random(gl_LaunchIDEXT.xy, 1));
  uv /= vec2(gl_LaunchSizeEXT.xy);
  uv = (uv * 2.0f - 1.0f) * vec2(1.0f, -1.0f);

  payload.rayOrigin = camera.position.xyz;
  payload.rayDirection =
      normalize(uv.x * camera.right + uv.y * camera.up + camera.forward).xyz;
  payload.previousNormal = vec3(0.0, 0.0, 0.0);

  payload.directColor = vec3(0.0, 0.0, 0.0);
  payload.indirectColor = vec3(0.0, 0.0, 0.0);
  payload.rayDepth = 0;

  payload.rayActive = 1;

  for (int x = 0; x < 16; x++) {
    traceRayEXT(topLevelAS, gl_RayFlagsOpaqueEXT, 0xFF, 0, 0, 0,
                payload.rayOrigin, 0.001, payload.rayDirection, 10000.0, 0);
  }

  vec4 color = vec4(payload.directColor + payload.indirectColor, 1.0);

  if (camera.frameCount > 0) {
    vec4 previousColor = imageLoad(image, ivec2(gl_LaunchIDEXT.xy));
    previousColor *= camera.frameCount;

    color += previousColor;
    color /= (camera.frameCount + 1);
  }

  imageStore(image, ivec2(gl_LaunchIDEXT.xy), color);
}
`;

function App() {
    const [shader, setShader] = useState(DEFAULT_SHADER);
    const [shaderKind, setShaderKind] = useState<ShaderKind>(
        ShaderKindRay.RayGeneration,
    );
    const [assembly, setAssembly] = useState<AnnotatedDisassembly | null>(null);
    const [error, setError] = useState('');

    const disassemblyDecorationIds = useRef<Array<string>>([]);
    const sourceDecorationIds = useRef<Array<string>>([]);

    const editorDisassemblyRef =
        useRef<null | monaco.editor.IStandaloneCodeEditor>(null);
    const handleEditorDisassemblyDidMount = (
        editor: monaco.editor.IStandaloneCodeEditor,
        monaco: Monaco,
    ) => {
        editorDisassemblyRef.current = editor;
    };
    const editorSourceRef = useRef<null | monaco.editor.IStandaloneCodeEditor>(
        null,
    );
    const handleEditorSourceDidMount = (
        editor: monaco.editor.IStandaloneCodeEditor,
        monaco: Monaco,
    ) => {
        editorSourceRef.current = editor;
    };

    const assemblyText = assembly
        ? assembly.instructions
              .map(instruction => instruction.instruction)
              .join('\n')
        : '';

    const decorationsByKey: { [key: string]: number } = {};

    const disassemblyDecorations: Array<monaco.editor.IModelDeltaDecoration> =
        [];
    let styleSheet = '';
    if (assembly) {
        let currentDecoration = 0;
        let decorationKey = '';
        let styles = [];

        for (let line = 0; line < assembly.instructions.length; line++) {
            const instruction = assembly.instructions[line];
            if (instruction.line) {
                const thisDecorationKey = JSON.stringify(instruction.line);
                if (thisDecorationKey !== decorationKey) {
                    decorationKey = thisDecorationKey;
                    currentDecoration = currentDecoration + 1;
                    styles.push(`.colored-line-${currentDecoration} {
                        background: hsl(${Math.random() * 360}, 100%, ${
                        13 + Math.random() * 10
                    }%);
                    }`);

                    decorationsByKey[thisDecorationKey] = currentDecoration;
                }

                disassemblyDecorations.push({
                    range: new monaco.Range(line + 1, 1, line + 1, 1),
                    options: {
                        isWholeLine: true,
                        className: `colored-line-${currentDecoration}`,
                    },
                });
            }
        }

        styleSheet = styles.join('\n\n');
    }

    const sourceDecorations: Array<monaco.editor.IModelDeltaDecoration> = [];
    for (const [key, id] of Object.entries(decorationsByKey)) {
        const lineAnnotation: LineAnnotation = JSON.parse(key);
        sourceDecorations.push({
            range: new monaco.Range(
                lineAnnotation.line,
                1,
                lineAnnotation.line,
                1,
            ),
            options: {
                isWholeLine: true,
                className: `colored-line-${id}`,
            },
        });
    }

    // We want to apply our decorations after the text has been updated
    Promise.resolve().then(() => {
        disassemblyDecorationIds.current =
            editorDisassemblyRef.current?.deltaDecorations(
                disassemblyDecorationIds.current,
                disassemblyDecorations,
            ) || [];
        sourceDecorationIds.current =
            editorSourceRef.current?.deltaDecorations(
                sourceDecorationIds.current,
                sourceDecorations,
            ) || [];
        console.log(
            editorSourceRef.current,
            sourceDecorations,
            decorationsByKey,
        );
    });

    const compile = async () => {
        const result = await compileShader(shader, shaderKind);
        console.log(result);

        if (compileShaderIsSuccess(result)) {
            setAssembly(result.Success.assembly);
            setError(result.Success.warning);
        } else {
            setAssembly(null);
            setError(result.Failure.error);
        }
    };

    const createShaderOptions = (label: string, options: Array<string>) => {
        return (
            <optgroup label={label}>
                {options.map(option => (
                    <option value={option} key={option}>
                        {
                            // Match an uppercase letter followed by lowercase letters
                            (option.match(/\p{Lu}\p{Ll}*/gu) ?? []).join(' ')
                        }
                    </option>
                ))}
            </optgroup>
        );
    };

    return (
        <div className='App'>
            <div>
                <select
                    value={shaderKind}
                    onChange={v => setShaderKind(v.target.value as ShaderKind)}
                >
                    {createShaderOptions(
                        'Raster',
                        Array.from(Object.keys(ShaderKindRaster)),
                    )}
                    {createShaderOptions(
                        'Ray',
                        Array.from(Object.keys(ShaderKindRay)),
                    )}
                    {createShaderOptions(
                        'Compute',
                        Array.from(Object.keys(ShaderKindCompute)),
                    )}
                    {createShaderOptions(
                        'Mesh',
                        Array.from(Object.keys(ShaderKindMesh)),
                    )}
                </select>
                <button onClick={compile}>Compile</button>
            </div>

            <span style={{ background: 'gray' }}>
                <style>{styleSheet}</style>
                <table style={{ margin: '0 auto' }}>
                    <tbody>
                        <tr>
                            <td>
                                <Editor
                                    value={shader}
                                    onChange={v => setShader(v || '')}
                                    height='90vh'
                                    width='49vw'
                                    theme={'vs-dark'}
                                    onMount={handleEditorSourceDidMount}
                                />
                            </td>
                            <td>
                                <Editor
                                    value={assemblyText}
                                    height='90vh'
                                    width='50vw'
                                    theme={'vs-dark'}
                                    options={{ readOnly: true }}
                                    onMount={handleEditorDisassemblyDidMount}
                                ></Editor>
                            </td>
                        </tr>
                    </tbody>
                </table>
            </span>

            {error ? (
                <textarea
                    readOnly={true}
                    value={error}
                    cols={120}
                    wrap='off'
                    style={{ height: '50vh', width: '90%' }}
                ></textarea>
            ) : undefined}
        </div>
    );
}

export default App;
