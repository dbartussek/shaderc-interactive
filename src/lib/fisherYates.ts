import Prando from 'prando';

export function shuffle<T>(array: Array<T>, rng?: Prando): Array<T> {
    const random = rng ?? new Prando();

    const result = [...array];
    for (let i = result.length - 1; 0 < i; --i) {
        const j = random.nextInt(0, i - 1);
        const temp = result[i];
        result[i] = result[j];
        result[j] = temp;
    }
    return result;
}
