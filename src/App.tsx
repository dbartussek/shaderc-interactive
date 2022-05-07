import React, { useRef, useState } from 'react';
import './App.css';
import {
    AnnotatedDisassembly,
    compileShader,
    compileShaderIsSuccess,
    CompileShaderOptions,
    LineAnnotation,
    ShaderKind,
    ShaderKindCompute,
    ShaderKindMesh,
    ShaderKindRaster,
    ShaderKindRay,
    TargetEnv,
} from './lib/shaderc';
import Editor, { Monaco } from '@monaco-editor/react';
import * as monaco from 'monaco-editor';
import { makeRainbowColors } from './lib/makeColors';

const TOP_BAR_HEIGHT = '50px';

function App() {
    const [rainbow, setRainbow] = useState(false);

    // Shader state
    const [shader, setShader] = useState('');
    const [shaderKind, setShaderKind] = useState<ShaderKind>(
        ShaderKindRaster.Vertex,
    );
    const [targetEnv, setTargetEnv] = useState(TargetEnv.Vulkan);

    // Response data by the backend
    const [assembly, setAssembly] = useState<AnnotatedDisassembly | null>(null);
    const [error, setError] = useState('');
    const [warning, setWarning] = useState('');

    // We decorate line matches in the editors. These are the decoration ids
    const disassemblyDecorationIds = useRef<Array<string>>([]);
    const sourceDecorationIds = useRef<Array<string>>([]);

    // Which source line decoration should be highlighted?
    const [highlightedLine, setHighlightedLine] = useState<number | null>(null);

    // Disassembly editor
    const editorDisassemblyRef =
        useRef<null | monaco.editor.IStandaloneCodeEditor>(null);
    const editorSourceRef = useRef<null | monaco.editor.IStandaloneCodeEditor>(
        null,
    );

    const paddingLengthLimit = useRef<null | HTMLInputElement>(null);
    const entryPoint = useRef<null | HTMLInputElement>(null);

    const editorDisassemblyPositionChanged = () => {
        const position = editorDisassemblyRef.current?.getPosition();
        if (!position) {
            return;
        }
        if (!assembly) {
            return;
        }
        const instruction = assembly.instructions[position.lineNumber - 1];
        if (!instruction) {
            return;
        }

        const sourceLineNumber = instruction.line?.line;
        if (sourceLineNumber === undefined) {
            return;
        }
        editorSourceRef.current?.revealLineInCenter(sourceLineNumber);

        setHighlightedLine(sourceLineNumber);
    };
    const editorDisassemblyPositionChangedRef = useRef(
        editorDisassemblyPositionChanged,
    );
    editorDisassemblyPositionChangedRef.current =
        editorDisassemblyPositionChanged;

    // Cursor position in source editor changed
    const editorSourcePositionChanged = () => {
        const sourceEditor = editorSourceRef.current;
        if (!sourceEditor) {
            return;
        }

        const position = sourceEditor.getPosition();
        if (!position) {
            return;
        }

        const sourceLineNumber = position.lineNumber;
        setHighlightedLine(sourceLineNumber);

        const disassemblyEditor = editorDisassemblyRef.current;
        if (!assembly || !disassemblyEditor) {
            return;
        }

        for (const [assemblyLineNumber, instruction] of Array.from(
            assembly.instructions.entries(),
        )) {
            if (instruction.line?.line === sourceLineNumber) {
                disassemblyEditor.revealLineInCenter(assemblyLineNumber + 1);
                break;
            }
        }
    };
    const editorSourcePositionChangedRef = useRef(editorSourcePositionChanged);
    editorSourcePositionChangedRef.current = editorSourcePositionChanged;

    const handleEditorDisassemblyDidMount = (
        editor: monaco.editor.IStandaloneCodeEditor,
        monaco: Monaco,
    ) => {
        editorDisassemblyRef.current = editor;
        editor.onDidChangeCursorPosition(() =>
            editorDisassemblyPositionChangedRef.current(),
        );
    };

    // Source editor
    const handleEditorSourceDidMount = (
        editor: monaco.editor.IStandaloneCodeEditor,
        monaco: Monaco,
    ) => {
        editorSourceRef.current = editor;
        editor.onDidChangeCursorPosition(() =>
            editorSourcePositionChangedRef.current(),
        );
    };

    // The assembly text is just all lines concatenated
    const assemblyText = assembly
        ? assembly.instructions
              .map(instruction => instruction.instruction)
              .join('\n')
        : '';

    const decorationsByLineAnnotation: { [key: string]: number } = {};

    const disassemblyDecorations: Array<monaco.editor.IModelDeltaDecoration> =
        [];
    if (assembly) {
        let decorationCounter = 0;

        for (let line = 0; line < assembly.instructions.length; line++) {
            const instruction = assembly.instructions[line];
            if (instruction.line) {
                const thisDecorationKey = JSON.stringify(instruction.line);

                // If this is a new source line, add a new decoration id
                if (
                    decorationsByLineAnnotation[thisDecorationKey] === undefined
                ) {
                    decorationCounter++;
                    decorationsByLineAnnotation[thisDecorationKey] =
                        decorationCounter;
                }

                const currentDecoration =
                    decorationsByLineAnnotation[thisDecorationKey];

                disassemblyDecorations.push({
                    range: new monaco.Range(line + 1, 1, line + 1, 1),
                    options: {
                        isWholeLine: true,
                        className: `colored-line-${currentDecoration}`,
                        linesDecorationsClassName: `colored-line-head-${currentDecoration}`,
                    },
                });
            }
        }
    }

    const sourceDecorations: Array<monaco.editor.IModelDeltaDecoration> = [];
    for (const [key, id] of Object.entries(decorationsByLineAnnotation)) {
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
                linesDecorationsClassName: `colored-line-head-${id}`,
            },
        });
    }

    // Generate line styles
    const styleSheet = makeRainbowColors(
        decorationsByLineAnnotation,
        highlightedLine,
        !rainbow,
    );

    // We want to apply our decorations after the text has been updated
    new Promise(resolve => setTimeout(resolve, 100)).then(() => {
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
    });

    const compile = async () => {
        const options: CompileShaderOptions = { targetEnv };
        const paddingLengthLimitCurrent = paddingLengthLimit.current?.value
            ? Number(paddingLengthLimit.current?.value)
            : null;
        if (
            typeof paddingLengthLimitCurrent == 'number' &&
            paddingLengthLimitCurrent >= 0
        ) {
            options.limitResultNameLength = Math.floor(
                paddingLengthLimitCurrent,
            );
        }
        const entryPointValue = entryPoint.current?.value;
        if (entryPointValue) {
            options.entryPoint = entryPointValue;
        }

        console.log(options);
        const result = await compileShader(shader, shaderKind, options);
        console.log(result);

        if (compileShaderIsSuccess(result)) {
            setAssembly(result.Success.assembly);
            setWarning(result.Success.warning);
            setError('');
        } else {
            setAssembly(null);
            setError(result.Failure.error);
            setWarning('');
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
            <div
                style={{ height: TOP_BAR_HEIGHT }}
                className='center-container'
            >
                <div className='center-element'>
                    <select
                        value={targetEnv}
                        onChange={v =>
                            setTargetEnv(v.target.value as TargetEnv)
                        }
                    >
                        {Array.from(Object.keys(TargetEnv)).map(target => {
                            return (
                                <option value={target} key={target}>
                                    {target}
                                </option>
                            );
                        })}
                    </select>
                    <select
                        value={shaderKind}
                        onChange={v =>
                            setShaderKind(v.target.value as ShaderKind)
                        }
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
                    <span>
                        <input
                            type='text'
                            placeholder='Entry point'
                            ref={entryPoint}
                            disabled={targetEnv !== TargetEnv.HLSL}
                        />
                    </span>
                    <span>
                        <input
                            type='number'
                            placeholder='Padding length limit'
                            ref={paddingLengthLimit}
                        />
                    </span>
                    <span>
                        <input
                            type='checkbox'
                            id='rainbow'
                            checked={rainbow}
                            onChange={e => setRainbow(e.target.checked)}
                        />
                        <label htmlFor='rainbow'>Rainbow colors</label>
                    </span>
                    <button onClick={compile}>Compile</button>
                </div>
            </div>

            <span style={{ background: 'gray' }}>
                <style>{styleSheet}</style>
                <table
                    cellPadding='0'
                    cellSpacing='0'
                    style={{ margin: '0 auto', padding: '0' }}
                >
                    <tbody>
                        <tr>
                            <td>
                                <Editor
                                    value={shader}
                                    onChange={v => setShader(v || '')}
                                    height={`calc(100vh - ${TOP_BAR_HEIGHT})`}
                                    width='49vw'
                                    theme={'vs-dark'}
                                    onMount={handleEditorSourceDidMount}
                                />
                            </td>
                            <td>
                                <Editor
                                    value={error || assemblyText}
                                    height={`calc(100vh - ${TOP_BAR_HEIGHT})`}
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

            {warning ? (
                <textarea
                    readOnly={true}
                    value={warning}
                    cols={120}
                    wrap='off'
                    style={{ height: '50vh', width: '90%' }}
                ></textarea>
            ) : undefined}
        </div>
    );
}

export default App;
