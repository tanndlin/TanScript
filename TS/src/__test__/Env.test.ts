import * as AST from '../AST';
import Engine from '../Engine';
import Environment from '../Environment';
import {
    UseBeforeDeclarationError as AssignBeforeDeclarationError,
    UndeclaredVariableError,
} from '../errors';
import { INumberableAST } from '../types';

describe('Enviornment Basic Tests', () => {
    it('should run basic script', () => {
        const assign = new AST.AssignASTNode(
            new AST.IdentifierASTNode('x'),
            new AST.NumberASTNode(10),
        );

        const decl = new AST.DeclarationASTNode(assign);
        const add = new AST.AddASTNode(
            new AST.IdentifierASTNode('x') as INumberableAST,
            new AST.NumberASTNode(5),
        );

        const root = new AST.BlockASTNode([decl, add]);
        const ast = new AST.AST(root);
        const env = new Environment(ast);
        const result = env.evaluate();
        expect(result).toBe(15);
    });

    it('should store variables in scope', () => {
        const root = new AST.BlockASTNode([]);
        const assign = new AST.AssignASTNode(
            new AST.IdentifierASTNode('x'),
            new AST.NumberASTNode(10),
        );

        const decl = new AST.DeclarationASTNode(assign);

        root.addChild(decl);
        const ast = new AST.AST(root);
        const env = new Environment(ast);
        env.evaluate();

        const scope = env.getGlobalScope();
        const x = scope.getVariable<number>('x');
        expect(x).toBe(10);
    });

    it('should allow declaration with no assignment', () => {
        const root = new AST.BlockASTNode([]);
        const decl = new AST.DeclarationASTNode(new AST.IdentifierASTNode('x'));

        root.addChild(decl);
        const ast = new AST.AST(root);
        const env = new Environment(ast);
        env.evaluate();

        const scope = env.getGlobalScope();
        const x = scope.getVariable<number>('x');
        expect(x).toBe(undefined);
    });

    it('should allow reassignment', () => {
        const root = new AST.BlockASTNode([]);
        const decl = new AST.DeclarationASTNode(new AST.IdentifierASTNode('x'));
        const newAssign = new AST.AssignASTNode(
            new AST.IdentifierASTNode('x'),
            new AST.NumberASTNode(20),
        );

        root.addChild(decl);
        root.addChild(newAssign);
        const ast = new AST.AST(root);
        const env = new Environment(ast);
        env.evaluate();

        const scope = env.getGlobalScope();
        const x = scope.getVariable<number>('x');
        expect(x).toBe(20);
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
    },
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

    it.each([
        ['let x = 0; if (1 < 2) { x=10; } else { x=20; }', 10],
        ['let x = 0; if (2 > 1) { x=10; } else { x=20; }', 10],
        ['let x = 0; if (1 > 2) { x=10; } else { x=20; }', 20],
        ['let x = 0; if (2 < 1) { x=10; } else { x=20; }', 20],
    ])('should evaluate correctly for script %s', (script, expected) => {
        const engine = new Engine(script);
        const result = engine.run();
        expect(result).toBe(expected);

        const env = engine.getEnvironment();
        const scope = env.getGlobalScope();
        const x = scope.getVariable<number>('x');
        expect(x).toBe(expected);
    });
});
