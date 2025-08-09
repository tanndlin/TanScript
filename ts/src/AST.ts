import { printf } from './BuiltInFunctions';
import { CompileScope } from './Compilation/CompileScope';
import { NotImplementedError } from './errors';
import { Address, BooleanToken, Register, Token } from './types';
import { addressIsRegister, addressToASM } from './util';

abstract class ASTStmt {
    type!: Token;
    abstract compile(scope: CompileScope): string[];

    public debugString(): string {
        return this.type.toString();
    }
}

export abstract class ASTExpr {
    type!: Token;
    abstract compile(scope: CompileScope, dst: Address): string[];

    public debugString(): string {
        return this.type.toString();
    }
}

export type Stmt =
    | Program
    | ASTDeclaration
    | ASTBlock
    | ASTWhile
    | ASTFor
    | ASTIf
    | ASTReturn;

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
    | ASTNumber
    | ASTArrayAccess;

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
        const functions: string[] = globalScope
            .getFunctions()
            .map((func) => func.instructions.join('\n'));

        return [
            'BITS 64',
            '',
            'global main',
            'extern printf',
            'extern malloc, free',
            'extern ExitProcess',
            '\nSECTION .data',
            CompileScope.data.join('\n'),
            'SECTION .text\n',
            functions.join('\n\n'),
            '\nmain:',
            '\tsub rsp, 40',
            '\tpush rbp',
            '\tmov rbp, rsp',
            ...instructions.flatMap((i) => `\t${i}`),
            '\tadd rsp, 40',
            '\tpop rbp',
            '\txor rcx, rcx',
            '\tcall ExitProcess',
        ];
    }
}

export abstract class ASTDecorator extends ASTExpr {
    compile(_scope: CompileScope): string[] {
        throw new NotImplementedError('Method not implemented.');
    }
}

export class ASTLParen extends ASTExpr {
    type: Token.LPAREN = Token.LPAREN;

    constructor(public child: Expr) {
        super();
    }

    compile(scope: CompileScope, dst: Address): string[] {
        return this.child.compile(scope, dst);
    }
}

export class ASTIdentifier extends ASTExpr {
    type: Token.IDENTIFIER = Token.IDENTIFIER;

    constructor(private name: string) {
        super();
    }

    compile(scope: CompileScope, dst: Address): string[] {
        const address = scope.getVariableAddress(this.name);
        const asmAddress = addressToASM(address);
        if (addressIsRegister(dst)) {
            return [`mov ${dst}, ${asmAddress}`];
        }

        return CompileScope.LeaseRandomRegistersWithScope(
            (reg) => [
                `; load ${this.name}`,
                `mov ${addressToASM(reg)}, ${asmAddress}`,
                `mov ${addressToASM(dst)}, ${reg}`,
            ],
            1,
        );
    }

    public getName(): string {
        return this.name;
    }

    public override debugString(): string {
        return this.name;
    }
}

export class ASTAssign extends ASTStmt {
    type: Token.ASSIGN = Token.ASSIGN;

    constructor(
        public lValue: ASTIdentifier | ASTArrayAccess,
        public valueAST: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope): string[] {
        if (this.lValue.type === Token.LBRACKET) {
            const address = scope.getVariableAddress(this.lValue.getName());

            return CompileScope.LeaseRandomRegistersWithScope(
                (reg, ptr) => [
                    `; ${this.lValue.debugString()} = ${this.valueAST.debugString()}`,
                    ...this.valueAST.compile(scope, reg),
                    '; store index',
                    ...(this.lValue as ASTArrayAccess).index.compile(
                        scope,
                        ptr,
                    ),
                    `imul ${ptr}, ${ptr}, 8`,
                    `add ${ptr}, ${addressToASM(address)}`,
                    `mov [${ptr}], ${reg}`,
                ],
                2,
            );
        }

        const address = scope.getVariableAddress(this.lValue.getName());
        return [
            `; ${this.lValue.debugString()} = ${this.valueAST.debugString()}`,
            ...this.valueAST.compile(scope, address),
        ];
    }

    public getName(): string {
        return this.lValue.getName();
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
            scope.addVariable(this.child.getName());
            return [];
        }

        scope.addVariable(this.child.getName());
        return this.child.compile(scope);
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
        throw new NotImplementedError('Method not implemented.');
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

        const newScope = new CompileScope(scope);
        let instructions: string[] = this.children.flatMap((child) => {
            if (child instanceof ASTStmt) {
                return child.compile(newScope);
            }

            return child.compile(newScope, Register.R15);
        });

        const numVariables = newScope.getNumVariables();
        if (numVariables) {
            instructions = [
                '; {',
                `sub rsp, ${numVariables * 8}`,
                ...instructions,
                `add rsp, ${numVariables * 8}`,
                '; }',
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
        throw new NotImplementedError('Method not implemented.');
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

    compile(scope: CompileScope, dst: Address): string[] {
        if (!addressIsRegister(dst)) {
            throw new Error(
                `Destination address must be a register, got ${dst}`,
            );
        }

        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                `; ${this.left.debugString()} < ${this.right.debugString()}`,
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

    compile(scope: CompileScope, dst: Address): string[] {
        if (!addressIsRegister(dst)) {
            throw new Error(
                `Destination address must be a register, got ${dst}`,
            );
        }

        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                `; ${this.left.debugString()} <= ${this.right.debugString()}`,
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

    compile(scope: CompileScope, dst: Address): string[] {
        if (!addressIsRegister(dst)) {
            throw new Error(
                `Destination address must be a register, got ${dst}`,
            );
        }

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

    compile(scope: CompileScope, dst: Address): string[] {
        if (!addressIsRegister(dst)) {
            throw new Error(
                `Destination address must be a register, got ${dst}`,
            );
        }

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

    compile(scope: CompileScope, dst: Address): string[] {
        if (!addressIsRegister(dst)) {
            throw new Error(
                `Destination address must be a register, got ${dst}`,
            );
        }

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

    compile(scope: CompileScope, dst: Address): string[] {
        if (!addressIsRegister(dst)) {
            throw new Error(
                `Destination address must be a register, got ${dst}`,
            );
        }

        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const left = this.left.compile(scope, lReg);
            const right = this.right.compile(scope, rReg);
            return [
                `; ${this.left.debugString()} == ${this.right.debugString()}`,
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

    compile(scope: CompileScope, dst: Address): string[] {
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

    compile(scope: CompileScope, dst: Address): string[] {
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

    compile(scope: CompileScope, dst: Address): string[] {
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
        const id = CompileScope.GetUniqueId();
        return CompileScope.LeaseRandomRegistersWithScope(
            (reg) => [
                '; while',
                `loopstart${id}:`,
                ...this.condition.compile(scope, reg),
                `test ${reg}, ${reg}`,
                `jz loopend${id}`,
                ...this.block.compile(scope),
                `jmp loopstart${id}`,
                `loopend${id}:`,
            ],
            1,
        );
    }
}

export class ASTFor extends ASTStmt {
    public type: Token.FOR = Token.FOR;

    constructor(
        public init: Stmt | Expr,
        public condition: Expr,
        public update: Stmt | Expr,
        public block: ASTBlock,
    ) {
        super();
    }

    compile(scope: CompileScope): string[] {
        const id = CompileScope.GetUniqueId();
        return CompileScope.LeaseRandomRegistersWithScope(
            (reg, cmp) => [
                ...this.init.compile(scope, reg),
                `loopstart${id}:`,
                ...this.condition.compile(scope, cmp),
                `test ${cmp}, ${cmp}`,
                `jz loopend${id}`,
                ...this.block.compile(scope),
                ...this.update.compile(scope, reg),
                `jmp loopstart${id}`,
                `loopend${id}:`,
            ],
            2,
        );
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
        const id = CompileScope.GetUniqueId();

        return CompileScope.LeaseRandomRegistersWithScope((reg) => {
            const instructions: string[] = [
                '; if',
                ...this.condition.compile(scope, reg),
                `test ${reg}, ${reg}`,
                `jz else${id}`,
                ...this.block.compile(scope),
                `jmp endif${id}`,
                `else${id}:`,
            ];

            if (this.elseBlock) {
                instructions.push(...this.elseBlock.compile(scope));
            }

            instructions.push(`endif${id}:`);
            return instructions;
        }, 1);
    }
}

export class ASTFunctionDef extends ASTStmt {
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

    compile(scope: CompileScope): string[] {
        const newScope = new CompileScope(scope);
        const registers = [
            Register.RCX,
            Register.RDX,
            Register.R8,
            Register.R9,
        ];
        this.paramList.forEach((param, index) => {
            if (index < 4) {
                newScope.addVariable(param.getName(), registers[index]);
                return;
            }

            newScope.addVariable(param.getName(), -40 - (index - 4) * 8);
        });

        const instructions = [
            `${this.name}:`,
            '\tpush rbp',
            '\tmov rbp, rsp',
            ...this.block.compile(newScope).map((i) => `\t${i}`),
            '\tpop rbp',
            '\tret',
        ];

        scope.addFunction({
            name: this.name,
            numParams: this.paramList.length,
            instructions,
        });

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

    compile(scope: CompileScope, dst: Address): string[] {
        if (this.name === 'print') {
            return printf(scope, this.args);
        }

        const functionDef = scope.getFunction(this.name);
        const externalFunctions = ['malloc', 'free', 'printf'];
        if (!functionDef && !externalFunctions.includes(this.name)) {
            throw new Error(`Function ${this.name} not defined`);
        }

        if (functionDef && functionDef.numParams !== this.args.length) {
            throw new Error(
                `Function ${this.name} expects ${functionDef.numParams} parameters, but got ${this.args.length}`,
            );
        }

        return CompileScope.LeaseRegistersWithScope(
            (...regs: Register[]) => {
                return CompileScope.LeaseRandomRegistersWithScope((reg) => {
                    const instructions: string[] = [
                        `; call ${this.name}(${this.args
                            .map((a) => a.debugString())
                            .join(', ')})`,
                        'sub rsp, 32',
                    ];
                    for (let i = 0; i < Math.min(this.args.length, 4); i++) {
                        instructions.push(
                            ...this.args[i].compile(scope, reg),
                            `mov ${addressToASM(regs[i])}, ${reg}`,
                        );
                    }

                    if (this.args.length > 4) {
                        instructions.push(
                            `sub rsp, ${(this.args.length - 4) * 8}`,
                        );
                        for (let i = this.args.length - 1; i >= 4; i--) {
                            instructions.push(
                                ...this.args[i].compile(scope, reg),
                                `mov [rsp + ${(i - 4) * 8 + 32}], ${reg}`, // 32 to be above the shadow space
                            );
                        }
                    }

                    CompileScope.LeaseRegister(Register.RAX);
                    instructions.push(`call ${this.name}`);
                    instructions.push(
                        `mov ${addressToASM(dst)}, ${Register.RAX}`,
                    ); // Assuming the return value is in RAX
                    CompileScope.ReleaseRegister(Register.RAX);
                    if (this.args.length > 4) {
                        instructions.push(
                            `add rsp, ${(this.args.length - 4) * 8}`,
                        ); // Clean up the stack
                    }

                    instructions.push('add rsp, 32'); // Clean up the stack
                    return instructions;
                }, 1);
            },
            Register.RCX,
            Register.RDX,
            Register.R8,
            Register.R9,
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
        return [
            `; return ${this.valueAST.debugString()}`,
            ...this.valueAST.compile(scope, Register.RAX),
        ];
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

    compile(scope: CompileScope, dst: Address): string[] {
        if (this.right.type === Token.NUMBER) {
            return [
                `; ${this.left.debugString()} + ${this.right.debugString()}`,
                ...this.left.compile(scope, dst),
                `add QWORD ${addressToASM(dst)}, ${this.right.getValue()}`,
            ];
        }

        return CompileScope.LeaseRandomRegistersWithScope(
            (rReg) => [
                `; ${this.left.debugString()} + ${this.right.debugString()}`,
                ...this.left.compile(scope, dst),
                ...this.right.compile(scope, rReg),
                `add ${addressToASM(dst)}, ${rReg}`,
            ],
            1,
        );
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

    compile(scope: CompileScope, dst: Address): string[] {
        if (this.right.type === Token.NUMBER) {
            return [
                `; ${this.left.debugString()} - ${this.right.debugString()}`,
                ...this.left.compile(scope, dst),
                `sub QWORD ${addressToASM(dst)}, ${this.right.getValue()}`,
            ];
        }

        return CompileScope.LeaseRandomRegistersWithScope((rReg) => [
            ...this.left.compile(scope, dst),
            ...this.right.compile(scope, rReg),
            `sub ${addressToASM(dst)}, ${rReg}`,
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

    compile(scope: CompileScope, dst: Address): string[] {
        if (this.right.type === Token.NUMBER) {
            return [
                `; ${this.left.debugString()} * ${this.right.debugString()}`,
                ...this.left.compile(scope, dst),
                `imul ${addressToASM(dst)}, ${this.right.getValue()}`,
            ];
        }

        return CompileScope.LeaseRandomRegistersWithScope(
            (reg) => [
                ...this.left.compile(scope, dst),
                ...this.right.compile(scope, reg),
                `imul ${dst}, ${reg}`,
            ],
            1,
        );
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

    compile(scope: CompileScope, dst: Address): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((rReg) => {
            return CompileScope.LeaseRegistersWithScope(
                (rax, rdx) => [
                    ...this.left.compile(scope, rax),
                    ...this.right.compile(scope, rReg),
                    `xor ${rdx}, ${rdx}`, // Clear RDX for division
                    `div ${rReg}`,
                    `mov ${addressToASM(dst)}, ${rax}`,
                ],
                Register.RAX,
                Register.RDX,
            );
        }, 1);
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

    compile(scope: CompileScope, dst: Address): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const leftInstructions = this.left.compile(scope, lReg);
            const rightInstructions = this.right.compile(scope, rReg);

            return CompileScope.LeaseRegistersWithScope(
                (rax, rdx) => [
                    ...leftInstructions,
                    ...rightInstructions,
                    `mov ${addressToASM(rax)}, ${lReg}`,
                    `xor ${rdx}, ${rdx}'`, // Clear RDX for division
                    `div ${rReg}`,
                    `mov ${addressToASM(dst)}, ${rax}`,
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

    compile(scope: CompileScope, dst: Address): string[] {
        return CompileScope.LeaseRandomRegistersWithScope((lReg, rReg) => {
            const leftInstructions = this.left.compile(scope, lReg);
            const rightInstructions = this.right.compile(scope, rReg);

            return CompileScope.LeaseRegistersWithScope(
                (rax, rdx) => [
                    `; ${this.left.debugString()} % ${this.right.debugString()}`,
                    ...leftInstructions,
                    ...rightInstructions,
                    `mov ${addressToASM(rax)}, ${lReg}`,
                    `xor ${rdx}, ${rdx}`, // Clear RDX for division
                    `div ${rReg}`,
                    `mov ${addressToASM(dst)}, ${rdx}`, // RDX contains the remainder
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

    compile(_scope: CompileScope, dst: Address): string[] {
        return [`mov QWORD ${addressToASM(dst)}, ${this.value}`];
    }

    public getValue(): number {
        return this.value;
    }

    public override debugString(): string {
        return this.value.toString();
    }
}

export class ASTArrayAccess extends ASTExpr {
    public type: Token.LBRACKET = Token.LBRACKET;

    constructor(
        public array: ASTIdentifier,
        public index: Expr,
    ) {
        super();
    }

    compile(scope: CompileScope, dst: Address): string[] {
        const address = scope.getVariableAddress(this.array.getName());
        if (addressIsRegister(dst)) {
            return CompileScope.LeaseRandomRegistersWithScope(
                (ptr) => [
                    `; ${this.array.debugString()}[${this.index.debugString()}]`,
                    ...this.index.compile(scope, ptr),
                    `imul ${ptr}, ${ptr}, 8`,
                    `add ${ptr}, ${addressToASM(address)}`,
                    `mov ${addressToASM(dst)}, [${ptr}]`,
                ],
                1,
            );
        }

        return CompileScope.LeaseRandomRegistersWithScope(
            (ptr) => [
                `; ${this.array.debugString()}[${this.index.debugString()}]`,
                '; store index',
                ...this.index.compile(scope, ptr),
                '; add base address',
                `add ${ptr}, ${addressToASM(address)}`, // Assuming 64-bit addressing
                `mov ${addressToASM(dst)}, [${ptr}]`,
            ],
            1,
        );
    }

    public getName(): string {
        return this.array.getName();
    }

    public debugString(): string {
        return `${this.array.debugString()}[${this.index.debugString()}]`;
    }
}
