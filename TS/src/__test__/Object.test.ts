import { AssignASTNode, DeclarationASTNode } from '../AST/AST';
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
        if (!(decl instanceof DeclarationASTNode)) {
            throw new Error('Expected declaration');
        }

        const { child } = decl;
        expect(child).toBeInstanceOf(AssignASTNode);
        if (!(child instanceof AssignASTNode)) {
            throw new Error('Expected assign');
        }

        const { valueAST } = child;
        expect(valueAST).toBeInstanceOf(ObjectASTNode);
        if (!(valueAST instanceof ObjectASTNode)) {
            throw new Error('Expected object');
        }

        expect(valueAST.attributes).toHaveLength(2);

        const [a, b] = valueAST.attributes;
        expect(a).toBeInstanceOf(AttributeASTNode);
        expect(b).toBeInstanceOf(AttributeASTNode);
        if (
            !(a instanceof AttributeASTNode) ||
            !(b instanceof AttributeASTNode)
        ) {
            throw new Error('Expected attribute');
        }

        const { valueAST: aValue } = a;
        const { valueAST: bValue } = b;

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

    it('math operations on object attributes', () => {
        const script = 'let obj = { a: 1, b: 2 };obj.a + obj.b;';
        const lexer = new Lexer(script);
        const parser = new Parser(lexer.getTokens());
        const ast = parser.parse();

        const env = new Environment(ast);
        const result = env.evaluate();

        expect(result).toBe(3);
    });
});
