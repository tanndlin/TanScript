import {
    AssignASTNode,
    BlockASTNode,
    DeclarationASTNode,
    IdentifierASTNode,
} from '../AST/AST';
import { ForEachASTNode, ListASTNode } from '../AST/IterableAST';
import { NumberASTNode } from '../AST/NumberAST';
import Engine from '../Engine';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { Token } from '../types';

describe('Iterable tests', () => {
    it('should lex a basic list', () => {
        const script = 'let x = [1,2,3];';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();

        expect(tokens).toHaveLength(12);
        expect(tokens[0].getType()).toBe(Token.DECLERATION);
        expect(tokens[1].getType()).toBe(Token.IDENTIFIER);
        expect(tokens[2].getType()).toBe(Token.ASSIGN);
        expect(tokens[3].getType()).toBe(Token.LBRACKET);
        expect(tokens[4].getType()).toBe(Token.NUMBER);
        expect(tokens[5].getType()).toBe(Token.COMMA);
        expect(tokens[6].getType()).toBe(Token.NUMBER);
        expect(tokens[7].getType()).toBe(Token.COMMA);
        expect(tokens[8].getType()).toBe(Token.NUMBER);
        expect(tokens[9].getType()).toBe(Token.RBRACKET);
        expect(tokens[10].getType()).toBe(Token.SEMI);
        expect(tokens[11].getType()).toBe(Token.EOF);
    });

    it('should parse a list', () => {
        const script = 'let x = [1,2,3];';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [decl] = root.getChildren();
        expect(decl).toBeInstanceOf(DeclarationASTNode);

        const [assign] = decl.getChildren();
        expect(assign).toBeInstanceOf(AssignASTNode);

        expect(assign.getChildren()).toHaveLength(2);
        expect(assign.getChildren()[0]).toBeInstanceOf(IdentifierASTNode);
        expect(assign.getChildren()[1]).toBeInstanceOf(ListASTNode);

        const list = assign.getChildren()[1] as ListASTNode;

        const items = list.getChildren();
        expect(items).toHaveLength(3);

        const [one, two, three] = items;
        expect(one).toBeInstanceOf(NumberASTNode);
        expect(two).toBeInstanceOf(NumberASTNode);
        expect(three).toBeInstanceOf(NumberASTNode);
    });

    it('should parse a for in loop', () => {
        const script = 'let x = [1,2,3]; foreach (i in x) { i; }';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [decl, forLoop] = root.getChildren();
        expect(decl).toBeInstanceOf(DeclarationASTNode);
        expect(forLoop).toBeInstanceOf(ForEachASTNode);

        const [loopingVarDecl, iter, block] = forLoop.getChildren();
        expect(loopingVarDecl).toBeInstanceOf(IdentifierASTNode);
        expect(iter).toBeInstanceOf(IdentifierASTNode);
        expect(block).toBeInstanceOf(BlockASTNode);

        expect(block.getChildren()[0]).toBeInstanceOf(IdentifierASTNode);
    });

    it('should allow inline array foreach', () => {
        const script = 'foreach (i in [1,2,3]) { i; }';
        const lexer = new Lexer(script);
        const tokens = lexer.getTokens();

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [forLoop] = root.getChildren();
        expect(forLoop).toBeInstanceOf(ForEachASTNode);

        const [loopingVarDecl, iter, block] = forLoop.getChildren();
        expect(loopingVarDecl).toBeInstanceOf(IdentifierASTNode);
        expect(iter).toBeInstanceOf(ListASTNode);
        expect(block).toBeInstanceOf(BlockASTNode);

        expect(block.getChildren()[0]).toBeInstanceOf(IdentifierASTNode);
    });

    it('should execute a foreach loop', () => {
        const script = 'let x = [1,2,3]; foreach (i in x) { i; }';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toBe(3);
    });

    it('Adding 2 lists should concatenate them', () => {
        const script = 'let x = [1,2,3]; let y = [4,5,6]; x + y;';
        const engine = new Engine(script);
        const result = engine.run();

        expect(result).toEqual([1, 2, 3, 4, 5, 6]);
    });
});
