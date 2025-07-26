import * as AST from '../AST';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { Token } from '../types';
import { getTokens } from './Lexer.test';

describe('Object Tests', () => {
    it('should parse an object', () => {
        const script = 'let obj = { a: 1, b: 2 };';
        const lexer = new Lexer(script);
        const parser = new Parser(getTokens(lexer));
        const ast = parser.parse();

        const root = ast.getRoot();
        const [decl] = root.children;

        expect(decl).toBeInstanceOf(AST.ASTDeclaration);
        if (!(decl.type === Token.DECLARATION)) {
            throw new Error('Expected declaration');
        }

        const { child } = decl;
        expect(child).toBeInstanceOf(AST.ASTAssign);
        if (!(child.type === Token.ASSIGN)) {
            throw new Error('Expected assign');
        }

        const { valueAST } = child;
        expect(valueAST).toBeInstanceOf(AST.ASTObject);
        if (!(valueAST instanceof AST.ASTObject)) {
            throw new Error('Expected object');
        }

        expect(valueAST.attributes).toHaveLength(2);

        const [a, b] = valueAST.attributes;
        expect(a).toBeInstanceOf(AST.ASTAttribute);
        expect(b).toBeInstanceOf(AST.ASTAttribute);
        if (
            !(a instanceof AST.ASTAttribute) ||
            !(b instanceof AST.ASTAttribute)
        ) {
            throw new Error('Expected attribute');
        }

        const { valueAST: aValue } = a;
        const { valueAST: bValue } = b;

        expect(a.getName()).toBe('a');
        expect(aValue).toBeInstanceOf(AST.ASTNumber);

        expect(b.getName()).toBe('b');
        expect(bValue).toBeInstanceOf(AST.ASTNumber);
    });
});
