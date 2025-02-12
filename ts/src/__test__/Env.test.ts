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
        const assign = new AST.ASTAssign(
            new AST.ASTIdentifier('x'),
            new AST.ASTNumber(10),
        );

        const decl = new AST.ASTDeclaration(assign);
        const add = new AST.ASTAdd(
            new AST.ASTIdentifier('x') as INumberableAST,
            new AST.ASTNumber(5),
        );

        const root = new AST.ASTBlock([decl, add]);
        const ast = new AST.Program(root);
        const env = new Environment(ast);
        const result = env.evaluate();
        expect(result).toBe(15);
    });

    it('should store variables in scope', () => {
        const assign = new AST.ASTAssign(
            new AST.ASTIdentifier('x'),
            new AST.ASTNumber(10),
        );

        const decl = new AST.ASTDeclaration(assign);

        const root = new AST.ASTBlock([decl]);
        const ast = new AST.Program(root);
        const env = new Environment(ast);
        env.evaluate();

        const scope = env.getGlobalScope();
        const x = scope.getVariable<number>('x');
        expect(x).toBe(10);
    });

    it('should allow declaration with no assignment', () => {
        const decl = new AST.ASTDeclaration(new AST.ASTIdentifier('x'));

        const root = new AST.ASTBlock([decl]);
        const ast = new AST.Program(root);
        const env = new Environment(ast);
        env.evaluate();

        const scope = env.getGlobalScope();
        const x = scope.getVariable<number>('x');
        expect(x).toBe(undefined);
    });

    it('should allow reassignment', () => {
        const decl = new AST.ASTDeclaration(new AST.ASTIdentifier('x'));
        const newAssign = new AST.ASTAssign(
            new AST.ASTIdentifier('x'),
            new AST.ASTNumber(20),
        );

        const root = new AST.ASTBlock([decl, newAssign]);
        const ast = new AST.Program(root);
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
