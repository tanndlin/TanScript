import { execFileSync, execSync } from 'child_process';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { writeInstructions } from '../util';
import { getTokens } from './Lexer.test';

const instructionsFromScript = (script: string) => {
    const lexer = new Lexer(script);
    const tokens = getTokens(lexer);
    const ast = new Parser(tokens).parse();

    return ast.compile();
};

describe('Basic Math Compilation', () => {
    it.each([
        ['printf("%d\n", 1 + 2)', 3],
        ['printf("%d\n", 69 + 420)', 489],
        ['printf("%d\n", 13 - 3)', 10],
        ['printf("%d\n", 13 * 13)', 169],
        ['printf("%d\n", 13 * 13 - 100)', 69],
        ['printf("%d\n", 10 / 2)', 5],
        ['printf("%d\n", 131 % 100)', 31],
        ['printf("%d\n", 131 % 10)', 1],
        ['printf("%d\n", 131 % 20)', 11],
    ])('should compile %s', (script, expectedOutput) => {
        const instructions = instructionsFromScript(script);
        writeInstructions(instructions, 'asm/test.asm');
        execSync('nasm -f win64 asm/test.asm -o test.o');
        execSync('gcc test.o -o test');
        const output = execFileSync('./test', { encoding: 'utf8' });
        expect(output.trim()).toBe(expectedOutput.toString());

        execSync('del test.o');
        execSync('del test');
    });
});
