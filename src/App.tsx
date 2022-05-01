import React, { useState } from 'react';
import './App.css';
import {
    compileShader,
    compileShaderIsSuccess,
    ShaderKind,
    ShaderKindCompute,
    ShaderKindMesh,
    ShaderKindRaster,
    ShaderKindRay,
} from './lib/shaderc';
import Editor from '@monaco-editor/react';

const DEFAULT_SHADER = `#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 1) in vec2 fragTexCoord;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragTexCoord, 0.0, 1.0);
}`;

function App() {
    const [shader, setShader] = useState(DEFAULT_SHADER);
    const [shaderKind, setShaderKind] = useState<ShaderKind>(
        ShaderKindRaster.Fragment,
    );
    const [assembly, setAssembly] = useState('');
    const [error, setError] = useState('');

    const compile = async () => {
        const result = await compileShader(shader, shaderKind);
        console.log(result);

        if (compileShaderIsSuccess(result)) {
            setAssembly(result.Success.assembly);
            setError(result.Success.warning);
        } else {
            setAssembly('');
            setError(result.Failure.error);
        }
    };

    const createShaderOptions = (label: string, options: Array<string>) => {
        return (
            <optgroup label={label}>
                {options.map(option => (
                    <option value={option}>
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
            <span style={{ background: 'gray' }}>
                <Editor
                    value={shader}
                    onChange={v => setShader(v || '')}
                    height='45vh'
                    theme={'vs-dark'}
                />
            </span>

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

            {assembly ? (
                <textarea
                    readOnly={true}
                    value={assembly}
                    cols={120}
                    wrap='off'
                    style={{ height: '50vh', width: '90%' }}
                ></textarea>
            ) : undefined}
            <br />
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
