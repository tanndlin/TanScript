import {
    AddInstruction,
    DivInstruction,
    MulInstruction,
    PushInstruction,
    SubInstruction,
} from '../Compilation/Instruction';
import Lexer from '../Lexer';
import Parser from '../Parser';

describe('Basic Math Compilation', () => {
    it('Should compile addition', () => {
        const script = '1 + 1';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const ast = new Parser(tokens).parse();

        const instructions = ast.compile();
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(1),
            new AddInstruction(),
        ]);
    });

    it('Should compile subtraction', () => {
        const script = '1 - 1';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const ast = new Parser(tokens).parse();

        const instructions = ast.compile();
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(1),
            new SubInstruction(),
        ]);
    });

    it('Should compile multiplication', () => {
        const script = '1 * 1';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const ast = new Parser(tokens).parse();

        const instructions = ast.compile();
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(1),
            new MulInstruction(),
        ]);
    });

    it('Should compile division', () => {
        const script = '1 / 1';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const ast = new Parser(tokens).parse();

        const instructions = ast.compile();
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(1),
            new DivInstruction(),
        ]);
    });
});

describe('Complex Math Compilation', () => {
    it('should compile chained addition', () => {
        const script = '1 + 2 + 3';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const ast = new Parser(tokens).parse();

        const instructions = ast.compile();
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(2),
            new AddInstruction(),
            new PushInstruction(3),
            new AddInstruction(),
        ]);
    });

    it('should respect pemdas', () => {
        const script = '1 + 2 * 3';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const ast = new Parser(tokens).parse();

        const instructions = ast.compile();
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(2),
            new PushInstruction(3),
            new MulInstruction(),
            new AddInstruction(),
        ]);
    });

    it('should respect parentheses first', () => {
        const script = '(1 + 2) * 3';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const ast = new Parser(tokens).parse();

        const instructions = ast.compile();
        expect(instructions).toEqual([
            new PushInstruction(1),
            new PushInstruction(2),
            new AddInstruction(),
            new PushInstruction(3),
            new MulInstruction(),
        ]);
    });

    it('should respect parentheses second', () => {
        const script = '3 * (1 + 2)';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();
        const ast = new Parser(tokens).parse();

        const instructions = ast.compile();
        expect(instructions).toEqual([
            new PushInstruction(3),
            new PushInstruction(1),
            new PushInstruction(2),
            new AddInstruction(),
            new MulInstruction(),
        ]);
    });
});
