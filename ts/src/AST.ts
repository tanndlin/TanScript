import { printf } from './BuiltInFunctions';
import { CompileScope } from './Compilation/CompileScope';
import { NotImplementedError, TannerError } from './errors';
import {
    BooleanToken,
    ComparisonToken,
    IterableResolvable,
    MathToken,
    Register,
    Token,
} from './types';

export abstract class ASTStmt {
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
    | ASTMath
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
            'SECTION .data',
            CompileScope.data.join('\n'),
            'SECTION .text',
            'main:',
            '\tsub rsp, 40',
            ...instructions.flatMap((i) => `\t${i}`),
            '\tadd rsp, 40',
            '\txor eax, eax',
            '\tret',
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
        const address = scope.getVariableAddress(this.name);
        return [];
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
        return [];
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

        const address = scope.addVariable(this.child.identifier.getName());
        const valueReg = CompileScope.LeaseRegister();
        const instructions = [
            ...this.child.compile(scope, valueReg),
            `mov ${address}, ${valueReg}`,
        ];

        CompileScope.ReleaseRegister(valueReg);
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
        return this.children.flatMap((child) => child.compile(scope));
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

export class ASTMath extends ASTExpr {
    constructor(
        public type: MathToken,
        public left: Expr,
        public right: Expr,
    ) {
        super();
        this.left = left;
        this.right = right;
    }

    handleAddition(left: any, right: any): any {
        // Special case to handle array concatenation
        if (left instanceof Array && right instanceof Array) {
            return left.concat(right);
        }

        return left + right;
    }

    compile(scope: CompileScope, dst: Register): string[] {
        const instructions = this.left
            .compile(scope, Register.RAX)
            .concat(this.right.compile(scope, Register.RBX));

        if (this.type === Token.PLUS) {
            instructions.push('ADD rax, rbx');
            instructions.push(`mov ${dst}, rax`);
        } else if (this.type === Token.MINUS) {
            instructions.push('SUB rax, rbx');
        } else if (this.type === Token.MULTIPLY) {
            instructions.push('MUL rax, rbx');
        } else if (this.type === Token.DIVIDE) {
            instructions.push('DIV rax, rbx');
        } else if (this.type === Token.MOD) {
            throw new NotImplementedError(
                'Modulus operation not implemented in compile',
            );
        } else {
            throw new TannerError('Unexpected call to ASTMath.compile');
        }

        return instructions.flat();
    }
}

export class ASTAdd extends ASTMath {
    type: Token.PLUS = Token.PLUS;

    constructor(left: Expr, right: Expr) {
        super(Token.PLUS, left, right);
    }
}

export class ASTSubtract extends ASTMath {
    type: Token.MINUS = Token.MINUS;

    constructor(left: Expr, right: Expr) {
        super(Token.MINUS, left, right);
    }
}

export class ASTMultiply extends ASTMath {
    type: Token.MULTIPLY = Token.MULTIPLY;

    constructor(left: Expr, right: Expr) {
        super(Token.MULTIPLY, left, right);
    }
}

export class ASTDivide extends ASTMath {
    type: Token.DIVIDE = Token.DIVIDE;

    constructor(left: Expr, right: Expr) {
        super(Token.DIVIDE, left, right);
    }
}

export class ASTIntegerDivide extends ASTMath {
    type: Token.INT_DIVIDE = Token.INT_DIVIDE;

    constructor(left: Expr, right: Expr) {
        super(Token.INT_DIVIDE, left, right);
    }
}

export class ASTMod extends ASTMath {
    type: Token.MOD = Token.MOD;

    constructor(left: Expr, right: Expr) {
        super(Token.MOD, left, right);
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
