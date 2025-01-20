import {
    AddInstruction,
    AllocInstruction,
    DivInstruction,
    EqInstruction,
    FrameInstruction,
    GeqInstruction,
    GotoInstruction,
    GreaterInstruction,
    JumpFalseInstruction,
    JumpInstruction,
    LeqInstruction,
    LessInstruction,
    LoadInstruction,
    ModInstruction,
    MulInstruction,
    NeqInstruction,
    PopStackInstruction,
    PrintCInstruction,
    PrintIntInstruction,
    PushInstruction,
    PushStackInstruction,
    ReturnInstruction,
    StoreInstruction,
    SubInstruction,
    UnframeInstruction,
} from './../Compilation/Instruction';

import Lexer from '../Lexer';
import Parser from '../Parser';
import { getTokens } from './Lexer.test';

const instructionsFromScript = (script: string) => {
    const lexer = new Lexer(script);
    const tokens = getTokens(lexer);
    const ast = new Parser(tokens).parse();

    return ast.compile();
};

describe('Basic Math Compilation', () => {
    it.each([
        ['1 + 2', new AddInstruction()],
        ['1 - 2', new SubInstruction()],
        ['1 * 2', new MulInstruction()],
        ['1 / 2', new DivInstruction()],
        ['1 % 2', new ModInstruction()],
    ])('should compile %s', (script, expectedInstruction) => {
        const instructions = instructionsFromScript(script);
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(2),
            expectedInstruction,
        ]);
    });
});

describe('Complex Math Compilation', () => {
    it('should compile chained addition', () => {
        const instructions = instructionsFromScript('1 + 2 + 3');
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(2),
            new AddInstruction(),
            new PushInstruction(3),
            new AddInstruction(),
        ]);
    });

    it('should respect pemdas', () => {
        const instructions = instructionsFromScript('1 + 2 * 3');
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(2),
            new PushInstruction(3),
            new MulInstruction(),
            new AddInstruction(),
        ]);
    });

    it('should respect parentheses first', () => {
        const instructions = instructionsFromScript('(1 + 2) * 3');
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(2),
            new AddInstruction(),
            new PushInstruction(3),
            new MulInstruction(),
        ]);
    });

    it('should respect parentheses second', () => {
        const instructions = instructionsFromScript('3 * (1 + 2)');
        expect(instructions).toEqual([
            new PushInstruction(3),
            new PushInstruction(1),
            new PushInstruction(2),
            new AddInstruction(),
            new MulInstruction(),
        ]);
    });
});

describe('Variable compilation', () => {
    it('should compile variable assignment', () => {
        const instructions = instructionsFromScript('let a = 1');
        expect(instructions).toEqual([
            new AllocInstruction(1),
            new PushInstruction(1),
            new StoreInstruction(0),
            new AllocInstruction(-1),
        ]);
    });

    it('should be able to use variables', () => {
        const instructions = instructionsFromScript('let a = 1; a + 2');
        expect(instructions).toEqual([
            new AllocInstruction(1),
            new PushInstruction(1),
            new StoreInstruction(0),
            new LoadInstruction(0),
            new PushInstruction(2),
            new AddInstruction(),
            new AllocInstruction(-1),
        ]);
    });

    it('should be able to use multiple variables', () => {
        const instructions = instructionsFromScript(
            'let a = 1; let b = 2; a + b',
        );
        expect(instructions).toEqual([
            new AllocInstruction(2),
            new PushInstruction(1),
            new StoreInstruction(0),
            new PushInstruction(2),
            new StoreInstruction(1),
            new LoadInstruction(0),
            new LoadInstruction(1),
            new AddInstruction(),
            new AllocInstruction(-2),
        ]);
    });

    it('should throw when variable already declared', () => {
        expect(() => instructionsFromScript('let a = 0; let a = 1')).toThrow();
    });

    it('should throw when variable not declared', () => {
        expect(() => instructionsFromScript('a = 0;')).toThrow();
    });

    it('should throw when function does not exist not declared', () => {
        expect(() => instructionsFromScript('foo()')).toThrow();
    });
});

describe('Control flow compilation', () => {
    it('should compile if statements', () => {
        const instructions = instructionsFromScript('if (1) { 2 }');
        expect(instructions).toEqual([
            new PushInstruction(1),
            new JumpFalseInstruction(1),
            new PushInstruction(2),
        ]);
    });

    it('should compile if else statements', () => {
        const instructions = instructionsFromScript('if (1) { 2 } else { 3 }');
        expect(instructions).toEqual([
            new PushInstruction(1),
            new JumpFalseInstruction(2),
            new PushInstruction(2),
            new JumpInstruction(1),
            new PushInstruction(3),
        ]);
    });

    it('if should jump over longer blocks', () => {
        const instructions = instructionsFromScript(
            'if (1) { return 2; } else { return 3; }',
        );
        expect(instructions).toEqual([
            new PushInstruction(1),
            new JumpFalseInstruction(5),
            new PushInstruction(2),
            new ReturnInstruction(),
            new PopStackInstruction(),
            new UnframeInstruction(),
            new JumpInstruction(4),
            new PushInstruction(3),
            new ReturnInstruction(),
            new PopStackInstruction(),
            new UnframeInstruction(),
        ]);
    });

    it('should compile while loops', () => {
        const instructions = instructionsFromScript(
            'while (1) { print(1); } ;',
        );

        expect(instructions).toEqual([
            new PushInstruction(1),
            new JumpFalseInstruction(3),
            new PushInstruction(1),
            new PrintIntInstruction(),
            new JumpInstruction(-5),
        ]);
    });

    it('should compile a function', () => {
        const instructions = instructionsFromScript('def foo() {print(1);}');

        expect(instructions).toEqual([
            new JumpInstruction(4),
            new PushInstruction(1),
            new PrintIntInstruction(),
            new PopStackInstruction(),
            new UnframeInstruction(),
        ]);
    });

    it('should compile  calling a function', () => {
        const instructions = instructionsFromScript(
            'def foo() {print(1);} foo();',
        );

        expect(instructions).toEqual([
            new JumpInstruction(4),
            new PushInstruction(1),
            new PrintIntInstruction(),
            new PopStackInstruction(),
            new UnframeInstruction(),
            new FrameInstruction(5),
            new AllocInstruction(1),
            new AllocInstruction(-1),
            new PushStackInstruction(),
            new AllocInstruction(0),
            new GotoInstruction(0),
        ]);
    });
});

describe('Comparison operators', () => {
    it.each([
        ['1 == 1', new EqInstruction()],
        ['1 != 1', new NeqInstruction()],
        ['1 > 1', new GreaterInstruction()],
        ['1 >= 1', new GeqInstruction()],
        ['1 < 1', new LessInstruction()],
        ['1 <= 1', new LeqInstruction()],
    ])('should compile %s', (script, ...expectedInstruction) => {
        const instructions = instructionsFromScript(script);
        const expectedInstructions = [
            new PushInstruction(1),
            new PushInstruction(1),
            ...expectedInstruction,
        ];
        expect(instructions).toEqual(expectedInstructions);
    });
});

describe('Misc Tests', () => {
    it('should compile printing a string', () => {
        // Puts the string in backwards, then prints each character
        expect(instructionsFromScript('print("Hello");')).toEqual([
            new PushInstruction(111),
            new PushInstruction(108),
            new PushInstruction(108),
            new PushInstruction(101),
            new PushInstruction(72),
            new PrintCInstruction(),
            new PrintCInstruction(),
            new PrintCInstruction(),
            new PrintCInstruction(),
            new PrintCInstruction(),
        ]);
    });
});
