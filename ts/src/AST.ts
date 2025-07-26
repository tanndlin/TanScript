import { printf } from './BuiltInFunctions';
import { CompileScope } from './Compilation/CompileScope';
import { NotImplementedError, TannerError } from './errors';
import {
    BooleanToken,
    ComparisonToken,
    IterableResolvable,
    Register,
    Token,
} from './types';

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
    | ASTComparison
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
        const reg = CompileScope.LeaseRegister();
        const instructions = this.child.compile(scope, reg);
        instructions.push(`mov [rbp - ${address}], ${reg}`);
        CompileScope.ReleaseRegister(reg);
        return instructions;
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

    constructor(public children: Stmt[]) {
        super();
    }

    setChildren(children: Stmt[]) {
        this.children = children;
    }

    compile(scope: CompileScope): string[] {
        // Alloc stack space for local variables
        let instructions: string[] = this.children.flatMap((child) =>
            child.compile(scope),
        );

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

export class ASTComparison extends ASTExpr {
    constructor(
        public type: ComparisonToken,
        public left: Expr,
        public right: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Register): string[] {
        const [lReg, rReg] = CompileScope.LeaseRegisters(2);
        const left = [this.left.compile(scope, lReg)].flat();
        const right = [this.right.compile(scope, rReg)].flat();

        const instructions = [...left, ...right];
        switch (this.type) {
            case Token.LESS:
                instructions.push();
                break;
            case Token.LEQ:
                instructions.push();
                break;
            case Token.GREATER:
                instructions.push();
                break;
            case Token.GEQ:
                instructions.push();
                break;
            case Token.EQUAL:
                instructions.push();
                break;
            case Token.NEQ:
                instructions.push();
                break;
            case Token.AND:
                instructions.push();
                break;
            case Token.OR:
                instructions.push();
                break;
            default:
                throw new TannerError(`Unexpected token: ${this.type}`);
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

export class ASTLessThan extends ASTComparison {
    public type: Token.LESS = Token.LESS;

    constructor(left: Expr, right: Expr) {
        super(Token.LESS, left, right);
    }
}

export class ASTLessEq extends ASTComparison {
    public type: Token.LEQ = Token.LEQ;

    constructor(left: Expr, right: Expr) {
        super(Token.LEQ, left, right);
    }
}

export class ASTGreaterThan extends ASTComparison {
    public type: Token.GREATER = Token.GREATER;

    constructor(left: Expr, right: Expr) {
        super(Token.GREATER, left, right);
    }
}

export class ASTGreaterEq extends ASTComparison {
    public type: Token.GEQ = Token.GEQ;

    constructor(left: Expr, right: Expr) {
        super(Token.GEQ, left, right);
    }
}

export class ASTNotEqual extends ASTComparison {
    public type: Token.NEQ = Token.NEQ;

    constructor(left: Expr, right: Expr) {
        super(Token.NEQ, left, right);
    }
}

export class ASTEqual extends ASTComparison {
    public type: Token.EQUAL = Token.EQUAL;

    constructor(left: Expr, right: Expr) {
        super(Token.EQUAL, left, right);
    }
}

export class ASTAnd extends ASTComparison {
    public type: Token.AND = Token.AND;

    constructor(left: Expr, right: Expr) {
        super(Token.AND, left, right);
    }
}

export class ASTOr extends ASTComparison {
    public type: Token.OR = Token.OR;

    constructor(left: Expr, right: Expr) {
        super(Token.OR, left, right);
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
        const [lReg, rReg] = CompileScope.LeaseRegisters(2);
        const leftInstructions = this.left.compile(scope, lReg);
        const rightInstructions = this.right.compile(scope, rReg);

        const instructions = [
            ...leftInstructions,
            ...rightInstructions,
            `add ${lReg}, ${rReg}`,
            `mov ${dst}, ${lReg}`,
        ];

        CompileScope.ReleaseRegister(lReg, rReg);
        return instructions;
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
        const [lReg, rReg] = CompileScope.LeaseRegisters(2);
        const leftInstructions = this.left.compile(scope, lReg);
        const rightInstructions = this.right.compile(scope, rReg);

        const instructions = [
            ...leftInstructions,
            ...rightInstructions,
            `sub ${lReg}, ${rReg}`,
            `mov ${dst}, ${lReg}`,
        ];

        CompileScope.ReleaseRegister(lReg, rReg);
        return instructions;
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
        const [lReg, rReg] = CompileScope.LeaseRegisters(2);
        const leftInstructions = this.left.compile(scope, lReg);
        const rightInstructions = this.right.compile(scope, rReg);

        const rax = CompileScope.LeaseRegister(Register.RAX);
        const instructions: string[] = [
            ...leftInstructions,
            ...rightInstructions,
            `mov ${rax}, ${lReg}`,
            `mul ${rReg}`,
            `mov ${dst}, ${rax}`,
        ];

        CompileScope.ReleaseRegister(lReg, rReg, rax);
        return instructions;
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
        throw new NotImplementedError('Division not implemented yet.');
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
        throw new NotImplementedError('Integer division not implemented yet.');
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
        throw new NotImplementedError('Modulus not implemented yet.');
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
