import { DeclarationASTNode } from '../AST/AST';
import { NumberASTNode } from '../AST/NumberAST';
import Environment from '../Environment';
import Lexer from '../Lexer';
import Parser from '../Parser';
import { AttributeASTNode, ObjectASTNode } from './../AST/ObjectAST';

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

        expect(obj).toBeInstanceOf(ObjectASTNode);
        expect(obj.getChildren().length).toBe(2);

        const [a, b] = obj.getChildren();
        expect(a).toBeInstanceOf(AttributeASTNode);
        expect(b).toBeInstanceOf(AttributeASTNode);

        const [aValue] = a.getChildren();
        const [bValue] = b.getChildren();

        expect(a.getValue()).toBe('a');
        expect(aValue).toBeInstanceOf(NumberASTNode);

        expect(b.getValue()).toBe('b');
        expect(bValue).toBeInstanceOf(NumberASTNode);
    });

    it('should evaluate an object', () => {
        const script = 'let obj = { a: 1, b: 2 };';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();

        const env = new Environment(ast);
        env.evaluate();

        const scope = env.getGlobalScope();
        const obj = scope.getVariable<Object>('obj');

        expect(obj).toStrictEqual({ attributes: { a: 1, b: 2 }, methods: {} });
    });

    it("should evaluate an object's attributes", () => {
        const script = 'let obj = { a: 1, b: 2 };obj.a;';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();

        const env = new Environment(ast);
        const result = env.evaluate();

        expect(result).toBe(1);
    });
});
