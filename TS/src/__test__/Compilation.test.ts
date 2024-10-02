import {
    AddInstruction,
    AllocInstruction,
    AndInstruction,
    DivInstruction,
    EqInstruction,
    GeqInstruction,
    GreaterInstruction,
    JumpFalseInstruction,
    JumpInstruction,
    JumpTrueInstruction,
    LeqInstruction,
    LessInstruction,
    LoadInstruction,
    ModInstruction,
    MulInstruction,
    NeqInstruction,
    OrInstruction,
    PopStackInstruction,
    PushInstruction,
    ReturnInstruction,
    StoreInstruction,
    SubInstruction,
    UnframeInstruction,
} from '../Compilation/Instruction';
import Lexer from '../Lexer';
import Parser from '../Parser';

const instructionsFromScript = (script: string) => {
    const lexer = new Lexer(script);
    const tokens = lexer.getTokens();
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
});

describe('Should short circuit boolean operations', () => {
    it('should short circuit and', () => {
        const instructions = instructionsFromScript('1 && 2');
        expect(instructions).toEqual([
            new PushInstruction(1),
            new JumpFalseInstruction(2),
            new PushInstruction(2),
            new AndInstruction(),
        ]);
    });

    it('should short circuit or', () => {
        const instructions = instructionsFromScript('1 || 2');
        expect(instructions).toEqual([
            new PushInstruction(1),
            new JumpTrueInstruction(2),
            new PushInstruction(2),
            new OrInstruction(),
        ]);
    });

    it('should short circuit and with variables', () => {
        const instructions = instructionsFromScript('let a = 1; a && 2');
        expect(instructions).toEqual([
            new AllocInstruction(1),
            new PushInstruction(1),
            new StoreInstruction(0),
            new LoadInstruction(0),
            new JumpFalseInstruction(2),
            new PushInstruction(2),
            new AndInstruction(),
            new AllocInstruction(-1),
        ]);
    });

    it('should short circuit or with variables', () => {
        const instructions = instructionsFromScript('let a = 1; a || 2');
        expect(instructions).toEqual([
            new AllocInstruction(1),
            new PushInstruction(1),
            new StoreInstruction(0),
            new LoadInstruction(0),
            new JumpTrueInstruction(2),
            new PushInstruction(2),
            new OrInstruction(),
            new AllocInstruction(-1),
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
