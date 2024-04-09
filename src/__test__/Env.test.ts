import { AST, AddASTNode, BlockASTNode } from '../AST';
import Engine from '../Engine';
import Environment from '../Environment';
import {
    UseBeforeDeclarationError as AssignBeforeDeclarationError,
    UndeclaredVariableError,
} from '../errors';
import {
    AssignASTNode,
    DeclarationASTNode,
    IdentifierASTNode,
    NumberASTNode,
} from './../AST';

describe('Enviornment Basic Tests', () => {
    it('should run basic script', () => {
        const root = new BlockASTNode([]);
        const decl = new DeclarationASTNode();
        const assign = new AssignASTNode(
            new IdentifierASTNode('x'),
            new NumberASTNode('10')
        );

        const add = new AddASTNode(
            new IdentifierASTNode('x'),
            new NumberASTNode('5')
        );

        decl.addChild(assign);
        root.addChild(decl);
        root.addChild(add);

        const ast = new AST(root);
        const env = new Environment(ast);
        const result = env.evaluate();
        expect(result).toBe(15);
    });

    it('should store variables in scope', () => {
        const root = new BlockASTNode([]);
        const decl = new DeclarationASTNode();
        const assign = new AssignASTNode(
            new IdentifierASTNode('x'),
            new NumberASTNode('10')
        );

        decl.addChild(assign);
        root.addChild(decl);

        const ast = new AST(root);
        const env = new Environment(ast);
        env.evaluate();

        const scope = env.getGlobalScope();
        const x = scope.getVariable<number>('x');
        expect(x).toBe(10);
    });
});

describe.each([
    ['1 <= 2', true],
    ['1 >= 2', false],
    ['1 < 2', true],
    ['1 > 2', false],
])(
    'Environment Integration Tests for boolean operator %s',
    (script, expected) => {
        it('should evaluate correctly', () => {
            const engine = new Engine(script);
            const result = engine.run();
            expect(result).toBe(expected);
        });
    }
);

describe('Enviornment Integration Tests', () => {
    it('should not allow assignment before declaration', () => {
        const script = 'x = 10;';
        const engine = new Engine(script);
        expect(() => engine.run()).toThrow(AssignBeforeDeclarationError);
    });

    it('should throw error for undeclared variable', () => {
        const script = '1 + x;';
        const engine = new Engine(script);
        expect(() => engine.run()).toThrow(UndeclaredVariableError);
    });
});
