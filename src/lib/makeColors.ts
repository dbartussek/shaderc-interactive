import Prando from 'prando';
import { LineAnnotation } from './shaderc';
import { shuffle } from './fisherYates';

function makeRainbow(
    steps: number,
    bottom?: number,
    top?: number,
): Array<number> {
    const result: Array<number> = [];

    for (let i = 0; i < steps; i++) {
        result.push((i / steps) * (top ?? 1) + (bottom ?? 0));
    }

    return result;
}

function classStyle(
    id: number,
    hue: number,
    isHighlighted: boolean,
    random: Prando,
) {
    const localRandom = new Prando(random.next());

    const saturation = isHighlighted ? 15 : 100;
    const light = isHighlighted ? 35 : localRandom.next(13, 25);

    const lineDirectives = [
        `background: hsl(${hue}, ${saturation}%, ${light}%)`,
    ];
    const lineHeadDirectives = [];
    if (isHighlighted) {
        lineHeadDirectives.push(
            'background: lightblue',
            'width: 5px !important',
            'margin-left: 3px',
        );
    }

    const join = (a: Array<string>) => a.map(s => `${s};\n`).join('');

    return `.colored-line-${id} {\n${join(lineDirectives)}}
.colored-line-head-${id} {\n${join(lineHeadDirectives)}}`;
}

export function makeRainbowColors(
    decorationsByLineAnnotation: { [key: string]: number },
    highlightedLine: number | null,
    shuffled: boolean = true,
): string {
    const random = new Prando('makeShuffledColor');

    let colors = makeRainbow(
        Object.keys(decorationsByLineAnnotation).length,
        0,
        280,
    );
    if (shuffled) {
        colors = shuffle(colors, random);
    }

    let styles = [];

    for (const [index, [key, id]] of Array.from(
        Object.entries(decorationsByLineAnnotation).entries(),
    )) {
        const lineAnnotation: LineAnnotation = JSON.parse(key);
        const isHighlighted = highlightedLine === lineAnnotation.line;

        styles.push(classStyle(id, colors[index], isHighlighted, random));
    }

    return styles.join('\n\n');
}
