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
        ['print(1 + 2)', 3],
        ['print(69 + 420)', 489],
        ['print(13 - 3)', 10],
        ['print(13 * 13)', 169],
        ['print(13 * 13 - 100)', 69],
    ])('should compile %s', (script, expectedInstruction) => {
        const instructions = instructionsFromScript(script);
        writeInstructions(instructions, 'asm/test.asm');
        execSync('nasm -f win64 asm/test.asm -o test.o');
        execSync('gcc test.o -o test');
        const output = execFileSync('./test', { encoding: 'utf8' });
        expect(output.trim()).toBe(expectedInstruction.toString());
    });
});
