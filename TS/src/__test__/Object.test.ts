import { DeclarationASTNode, IdentifierASTNode } from '../AST/AST';
import { NumberASTNode } from '../AST/NumberAST';
import Environment from '../Environment';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { AttributeAST, ObjectAST } from './../AST/ObjectAST';

describe('Object Tests', () => {
    it('should parse an object', () => {
        const script = 'let obj = { a: 1, b: 2 };';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();

        const root = ast.getRoot();
        const [decl] = root.getChildren();

        expect(decl).toBeInstanceOf(DeclarationASTNode);

        const [assign] = decl.getChildren();
        const [ident, obj] = assign.getChildren();

        expect(obj).toBeInstanceOf(ObjectAST);
        expect(obj.getChildren().length).toBe(2);

        const [a, b] = obj.getChildren();
        expect(a).toBeInstanceOf(AttributeAST);
        expect(b).toBeInstanceOf(AttributeAST);

        const [aName, aValue] = a.getChildren();
        const [bName, bValue] = b.getChildren();

        expect(aName).toBeInstanceOf(IdentifierASTNode);
        expect(aValue).toBeInstanceOf(NumberASTNode);

        expect(bName).toBeInstanceOf(IdentifierASTNode);
        expect(bValue).toBeInstanceOf(NumberASTNode);
    });

    it('should evaluate an object', () => {
        const script = 'let obj = { a: 1, b: 2 };';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();

        const root = ast.getRoot();
        const [decl] = root.getChildren();

        const env = new Environment(ast);
        env.evaluate();

        const scope = env.getGlobalScope();
        const obj = scope.getVariable<Object>('obj');

        expect(obj).toStrictEqual({ a: 1, b: 2 });
    });
});
