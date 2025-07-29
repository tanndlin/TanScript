import { printf } from './BuiltInFunctions';
import { CompileScope } from './Compilation/CompileScope';
import { NotImplementedError } from './errors';
import { BooleanToken, IterableResolvable, Register, Token } from './types';

abstract class ASTStmt {
    type!: Token;
    abstract compile(scope: CompileScope): string[];
}

export abstract class ASTExpr {
    type!: Token;
    abstract compile(scope: CompileScope, dst: Register): string[];
}

export type Stmt =
    | Program
    | ASTDeclaration
    | ASTBlock
    | ASTWhile
    | ASTFor
    | ASTIf
    | ASTReturn
    | ASTForEach;

export type Expr =
    | ASTAssign
    | ASTLParen
    | ASTIdentifier
    | ASTString
    | ASTMathType
    | ASTLessThan
    | ASTLessEq
    | ASTGreaterThan
    | ASTGreaterEq
    | ASTNotEqual
    | ASTEqual
    | ASTAnd
    | ASTOr
    | ASTBoolean
    | ASTNot
    | ASTFunctionDef
    | ASTFunctionCall
    | ASTIterable
    | ASTNumber
    | ASTObject
    | ASTAttribute
    | ObjectAccessAST;

export type AnyAST = Stmt | Expr;

export class Program extends ASTStmt {
    type: Token.PROGRAM = Token.PROGRAM;

    constructor(private root: ASTBlock) {
        super();
    }

    public isType(type: Token): boolean {
        return this.type === type;
    }

    public isOneOf(...types: Token[]): boolean {
        if (!this.type) {
            return false;
        }

        return types.includes(this.type);
    }

    getRoot() {
        return this.root;
    }

    compile(): string[] {
        const globalScope = new CompileScope();

        const instructions = this.root.compile(globalScope);

        return [
            'BITS 64',
            '',
            'global main',
            'extern printf',
            'extern ExitProcess',
            'SECTION .data',
            CompileScope.data.join('\n'),
            'SECTION .text',
            'main:',
            '\tsub rsp, 40',
            ...instructions.flatMap((i) => `\t${i}`),
            '\tadd rsp, 40',
            '\txor rcx, rcx',
            '\tcall ExitProcess',
        ];
    }
}

export abstract class ASTDecorator extends ASTExpr {
    compile(_scope: CompileScope): string[] {
        return [];
    }
}

export class ASTLParen extends ASTExpr {
    type: Token.LPAREN = Token.LPAREN;

    constructor(public child: Expr) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return this.child.compile(scope, dst);
    }
}

export class ASTIdentifier extends ASTExpr {
    type: Token.IDENTIFIER = Token.IDENTIFIER;

    constructor(private name: string) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        const address = scope.getVariableAddress(this.name) + 8;
        return [`mov ${dst}, [rbp - ${address}]`];
    }

    public getName(): string {
        return this.name;
    }
}

export class ASTAssign extends ASTExpr {
    type: Token.ASSIGN = Token.ASSIGN;

    constructor(
        public identifier: ASTIdentifier,
        public valueAST: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return this.valueAST.compile(scope, dst);
    }

    public getName(): string {
        return this.identifier.getName();
    }
}

export class ASTDeclaration extends ASTStmt {
    type: Token.DECLARATION = Token.DECLARATION;

    constructor(public child: ASTAssign | ASTIdentifier) {
        super();
    }

    compile(scope: CompileScope): string[] {
        // The allocation is already handled by hoisting in the block scope
        if (this.child.type === Token.IDENTIFIER) {
            const reg = scope.addVariable(this.child.getName());
            return [];
        }

        const address = scope.addVariable(this.child.getName()) + 8;
        // const reg = CompileScope.LeaseRegister();
        return CompileScope.LeaseRandomRegistersWithScope((reg: Register) => [
            ...this.child.compile(scope, reg),
            `mov [rbp - ${address}], ${reg}`,
        ]);
    }

    public getName(): string {
        return this.child.getName();
    }
}

export class ASTString extends ASTExpr {
    type: Token.STRING = Token.STRING;

    constructor(private value: string) {
        super();
    }

    compile(_scope: CompileScope): string[] {
        return [];
    }

    public getValue(): string {
        return this.value;
    }
}

export class ASTBlock extends ASTStmt {
    type: Token.LCURLY = Token.LCURLY;

    constructor(public children: (Stmt | Expr)[]) {
        super();
    }

    setChildren(children: (Stmt | Expr)[]) {
        this.children = children;
    }

    compile(scope: CompileScope): string[] {
        // Alloc stack space for local variables
        let instructions: string[] = this.children.flatMap((child) => {
            if (child instanceof ASTStmt) {
                return child.compile(scope);
            }

            return child.compile(scope, Register.R15);
        });

        const numVariables = scope.getNumVariables();
        if (numVariables) {
            instructions = [
                `sub rsp, ${numVariables * 8}`,
                ...instructions,
                `add rsp, ${numVariables * 8}`,
            ];
        }

        return instructions;
    }
}

export class ASTBoolean extends ASTExpr {
    public type: BooleanToken;

    constructor(type: BooleanToken) {
        super();
        this.type = type;
    }

    compile(_scope: CompileScope): string[] {
        return [];
    }
}

export type ASTComparisonType =
    | ASTLessThan
    | ASTLessEq
    | ASTGreaterThan
    | ASTGreaterEq
    | ASTNotEqual
    | ASTEqual
    | ASTAnd
    | ASTOr;

export class ASTLessThan extends ASTExpr {
    public type: Token.LESS = Token.LESS;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                ...left,
                ...right,
                `cmp ${lReg}, ${rReg}`,
                `mov ${dst}, 0`,
                `setl ${dst}b`,
            ];
        }, 2);
    }
}

export class ASTLessEq extends ASTExpr {
    public type: Token.LEQ = Token.LEQ;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                ...left,
                ...right,
                `cmp ${lReg}, ${rReg}`,
                `mov ${dst}, 0`,
                `setle ${dst}b`,
            ];
        }, 2);
    }
}

export class ASTGreaterThan extends ASTExpr {
    public type: Token.GREATER = Token.GREATER;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                ...left,
                ...right,
                `cmp ${lReg}, ${rReg}`,
                `mov ${dst}, 0`,
                `setg ${dst}b`,
            ];
        }, 2);
    }
}

export class ASTGreaterEq extends ASTExpr {
    public type: Token.GEQ = Token.GEQ;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                ...left,
                ...right,
                `cmp ${lReg}, ${rReg}`,
                `mov ${dst}, 0`,
                `setge ${dst}b`,
            ];
        }, 2);
    }
}

export class ASTNotEqual extends ASTExpr {
    public type: Token.NEQ = Token.NEQ;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                ...left,
                ...right,
                `cmp ${lReg}, ${rReg}`,
                `mov ${dst}, 0`,
                `setne ${dst}b`,
            ];
        }, 2);
    }
}

export class ASTEqual extends ASTExpr {
    public type: Token.EQUAL = Token.EQUAL;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                ...left,
                ...right,
                `cmp ${lReg}, ${rReg}`,
                `mov ${dst}, 0`,
                `sete ${dst}b`,
            ];
        }, 2);
    }
}

export class ASTAnd extends ASTExpr {
    public type: Token.AND = Token.AND;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        const left = this.left.compile(scope, dst);
        return CompileScope.LeaseRandomRegistersWithScope((rReg) => {
            const right = this.right.compile(scope, rReg);
            return [...left, ...right, `and ${dst}, ${rReg}`];
        }, 1);
    }
}

export class ASTOr extends ASTExpr {
    public type: Token.OR = Token.OR;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        const left = this.left.compile(scope, dst);
        return CompileScope.LeaseRandomRegistersWithScope((rReg) => {
            const right = this.right.compile(scope, rReg);
            return [...left, ...right, `or ${dst}, ${rReg}`];
        }, 2);
    }
}

export class ASTNot extends ASTExpr {
    public type: Token.NOT = Token.NOT;
    constructor(public child: Expr) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return [...this.child.compile(scope, dst), `NOT ${dst}`];
    }
}

export class ASTWhile extends ASTStmt {
    public type: Token.WHILE = Token.WHILE;

    constructor(
        public condition: Expr,
        public block: ASTBlock,
    ) {
        super();
    }

    compile(scope: CompileScope): string[] {
        return [];
    }
}

export class ASTFor extends ASTStmt {
    public type: Token.FOR = Token.FOR;

    constructor(
        public init: Stmt,
        public condition: Expr,
        public update: Stmt,
        public block: ASTBlock,
    ) {
        super();
    }

    compile(scope: CompileScope): string[] {
        return [];
    }
}

export class ASTIf extends ASTStmt {
    public type: Token.IF = Token.IF;

    constructor(
        public condition: Expr,
        public block: ASTBlock,
        public elseBlock?: ASTBlock,
    ) {
        super();

        this.condition = condition;
        this.block = block;
        this.elseBlock = elseBlock;
    }

    compile(scope: CompileScope): string[] {
        return [];
    }
}

export class ASTFunctionDef extends ASTExpr {
    public type: Token.FUNCTION = Token.FUNCTION;

    constructor(
        private name: string,
        private paramList: ASTIdentifier[],
        public block: ASTBlock,
    ) {
        super();
        this.paramList = paramList;
        this.block = block;
    }

    getParamList() {
        return this.paramList;
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return [];
    }
}

export class ASTFunctionCall extends ASTExpr {
    public type: Token.IDENTIFIER = Token.IDENTIFIER;

    constructor(
        private name: string,
        public args: Expr[],
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        if (this.name === 'print') {
            return printf(scope, this.args);
        }

        throw new NotImplementedError(
            `Custom function calls not implemented: ${this.name}`,
        );
    }

    public getName() {
        return this.name;
    }
}

export class ASTReturn extends ASTStmt {
    public type: Token.RETURN = Token.RETURN;

    constructor(public valueAST: Expr) {
        super();
    }

    compile(scope: CompileScope): string[] {
        return [];
    }
}

export class ASTIterable extends ASTExpr {
    public type: Token.LBRACKET = Token.LBRACKET;

    constructor(public items: Expr[]) {
        super();
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }
}

export class ASTList extends ASTIterable {}

export class ASTForEach extends ASTStmt {
    public init: ASTDeclaration | ASTIdentifier;
    public iterable: IterableResolvable;
    public block: ASTBlock;
    public type: Token.FOREACH = Token.FOREACH;

    constructor(
        init: ASTDeclaration | ASTIdentifier,
        iterable: IterableResolvable,
        block: ASTBlock,
    ) {
        super();
        this.init = init;
        this.iterable = iterable;
        this.block = block;
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }
}

export type ASTMathType =
    | ASTAdd
    | ASTSubtract
    | ASTMultiply
    | ASTDivide
    | ASTIntegerDivide
    | ASTMod;

export class ASTAdd extends ASTExpr {
    type: Token.PLUS = Token.PLUS;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((rReg) => [
            ...this.left.compile(scope, dst),
            ...this.right.compile(scope, rReg),
            `add ${dst}, ${rReg}`,
        ]);
    }
}

export class ASTSubtract extends ASTExpr {
    type: Token.MINUS = Token.MINUS;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((rReg) => [
            ...this.left.compile(scope, dst),
            ...this.right.compile(scope, rReg),
            `sub ${dst}, ${rReg}`,
        ]);
    }
}

export class ASTMultiply extends ASTExpr {
    type: Token.MULTIPLY = Token.MULTIPLY;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const leftInstructions = this.left.compile(scope, lReg);
            const rightInstructions = this.right.compile(scope, rReg);

            return CompileScope.LeaseRegistersWithScope(
                (rax: Register) => [
                    ...leftInstructions,
                    ...rightInstructions,
                    `mov ${rax}, ${lReg}`,
                    `mul ${rReg}`,
                    `mov ${dst}, ${rax}`,
                ],
                Register.RAX,
            );
        }, 2);
    }
}

export class ASTDivide extends ASTExpr {
    type: Token.DIVIDE = Token.DIVIDE;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const leftInstructions = this.left.compile(scope, lReg);
            const rightInstructions = this.right.compile(scope, rReg);

            return CompileScope.LeaseRegistersWithScope(
                (rax, rdx) => [
                    ...leftInstructions,
                    ...rightInstructions,
                    `mov ${rax}, ${lReg}`,
                    `xor ${rdx}, ${rdx}`, // Clear RDX for division
                    `div ${rReg}`,
                    `mov ${dst}, ${rax}`,
                ],
                Register.RAX,
                Register.RDX,
            );
        }, 2);
    }
}

export class ASTIntegerDivide extends ASTExpr {
    type: Token.INT_DIVIDE = Token.INT_DIVIDE;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const leftInstructions = this.left.compile(scope, lReg);
            const rightInstructions = this.right.compile(scope, rReg);

            return CompileScope.LeaseRegistersWithScope(
                (rax, rdx) => [
                    ...leftInstructions,
                    ...rightInstructions,
                    `mov ${rax}, ${lReg}`,
                    `xor ${rdx}, ${rdx}'`, // Clear RDX for division
                    `div ${rReg}`,
                    `mov ${dst}, ${rax}`,
                ],
                Register.RAX,
                Register.RDX,
            );
        }, 2);
    }
}

export class ASTMod extends ASTExpr {
    type: Token.MOD = Token.MOD;

    constructor(
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const leftInstructions = this.left.compile(scope, lReg);
            const rightInstructions = this.right.compile(scope, rReg);

            return CompileScope.LeaseRegistersWithScope(
                (rax, rdx) => [
                    ...leftInstructions,
                    ...rightInstructions,
                    `mov ${rax}, ${lReg}`,
                    `xor ${rdx}, ${rdx}`, // Clear RDX for division
                    `div ${rReg}`,
                    `mov ${dst}, ${rdx}`, // RDX contains the remainder
                ],
                Register.RAX,
                Register.RDX,
            );
        }, 2);
    }
}

export class ASTNumber extends ASTExpr {
    public type: Token.NUMBER = Token.NUMBER;

    constructor(private value: number) {
        super();
    }

    compile(_scope: CompileScope, dst: Register): string[] {
        return [`mov ${dst}, ${this.value}`];
    }

    public getValue(): number {
        return this.value;
    }
}

export class ASTObject extends ASTExpr {
    public type: Token.LCURLY = Token.LCURLY;

    constructor(public attributes: ASTAttribute[]) {
        super();
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }
}

export class ASTAttribute extends ASTExpr {
    type: Token.PERIOD = Token.PERIOD;

    constructor(
        private name: string,
        public valueAST: Expr,
    ) {
        super();
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }

    public getName(): string {
        return this.name;
    }
}

export class ObjectAccessAST extends ASTExpr {
    public type: Token.IDENTIFIER = Token.IDENTIFIER;
    constructor(
        public objIdentifier: ASTIdentifier,
        public attribute: ASTIdentifier,
    ) {
        super();
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }
}
