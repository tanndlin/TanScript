import * as AST from '../AST';

import Lexer from '../Lexer';
import Parser from '../Parser';
import { Token } from '../types';
import { getTokens } from './Lexer.test';

describe('Iterable tests', () => {
    it('should lex a basic list', () => {
        const script = 'let x = [1,2,3];';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);

        expect(tokens).toHaveLength(12);
        expect(tokens[0].getType()).toBe(Token.DECLARATION);
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
        const tokens = getTokens(lexer);

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [decl] = root.children;
        expect(decl).toBeInstanceOf(AST.ASTDeclaration);
        if (!(decl.type === Token.DECLARATION)) {
            throw new Error('Expected declaration');
        }

        const { child: assign } = decl;
        expect(assign).toBeInstanceOf(AST.ASTAssign);
        if (!(assign.type === Token.ASSIGN)) {
            throw new Error('Expected assign');
        }

        expect(assign.valueAST).toBeInstanceOf(AST.ASTList);
        const list = assign.valueAST as AST.ASTList;

        const { items } = list;
        expect(items).toHaveLength(3);

        const [one, two, three] = items;
        expect(one).toBeInstanceOf(AST.ASTNumber);
        expect(two).toBeInstanceOf(AST.ASTNumber);
        expect(three).toBeInstanceOf(AST.ASTNumber);
    });

    it('should parse a for in loop', () => {
        const script = 'let x = [1,2,3]; foreach (i in x) { i; }';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [decl, forLoop] = root.children;
        expect(decl).toBeInstanceOf(AST.ASTDeclaration);
        expect(forLoop).toBeInstanceOf(AST.ASTForEach);
        if (!(forLoop.type === Token.FOREACH)) {
            throw new Error('Expected for loop');
        }

        const { init, iterable, block } = forLoop;
        expect(init).toBeInstanceOf(AST.ASTIdentifier);
        expect(iterable).toBeInstanceOf(AST.ASTIdentifier);
        expect(block).toBeInstanceOf(AST.ASTBlock);

        expect(block.children[0]).toBeInstanceOf(AST.ASTIdentifier);
    });

    it('should allow inline array foreach', () => {
        const script = 'foreach (i in [1,2,3]) { i; }';
        const lexer = new Lexer(script);
        const tokens = getTokens(lexer);

        const parser = new Parser(tokens);
        const ast = parser.parse();
        const root = ast.getRoot();

        const [forLoop] = root.children;
        expect(forLoop).toBeInstanceOf(AST.ASTForEach);
        if (!(forLoop.type === Token.FOREACH)) {
            throw new Error('Expected for loop');
        }

        const { init, iterable, block } = forLoop;
        expect(init).toBeInstanceOf(AST.ASTIdentifier);
        expect(iterable).toBeInstanceOf(AST.ASTList);
        expect(block).toBeInstanceOf(AST.ASTBlock);

        expect(block.children[0]).toBeInstanceOf(AST.ASTIdentifier);
    });
});
