import { BuiltInFuncName, allFunctions } from '../BuiltInFunctions';
import { CompileScope } from '../Compilation/CompileScope';
import {
    AllocInstruction,
    FrameInstruction,
    GotoInstruction,
    Instruction,
    JumpFalseInstruction,
    JumpInstruction,
    PopStackInstruction,
    PrintCInstruction,
    PrintIntInstruction,
    PushStackInstruction,
    StoreInstruction,
    UnframeInstruction,
} from '../Compilation/Instruction';
import Scope from '../Scope';
import { RuntimeError } from '../errors';
import { IChildrenEnumerable, RuntimeValue, Token } from '../types';
import { ASTNode, BlockASTNode, IdentifierASTNode, StringASTNode } from './AST';
import { BooleanOpASTNode } from './BoolAST';

export class WhileASTNode extends ASTNode {
    public condition: ASTNode;

    public block: ASTNode;

    constructor(condition: ASTNode, block: ASTNode) {
        super(Token.WHILE);
        this.condition = condition;
        this.block = block;
    }

    evaluate(scope: Scope): RuntimeValue {
        let ret;

        while ((this.condition as BooleanOpASTNode).evaluate(scope)) {
            ret = this.block.evaluate(scope);
        }

        return ret;
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        const instructions = [];
        const condition = [this.condition.compile(scope)].flat();
        const block = [this.block.compile(scope)].flat();

        instructions.push(...condition);
        // Jump out of block if condition is false
        instructions.push(new JumpFalseInstruction(block.length + 1));
        instructions.push(...block);
        // Jump back to condition
        instructions.push(new JumpInstruction(-instructions.length - 1));

        return instructions;
    }
}

export class ForASTNode extends ASTNode {
    constructor(
        public init: ASTNode,
        public condition: ASTNode,
        public update: ASTNode,
        public block: ASTNode,
    ) {
        super(Token.FOR);
    }

    evaluate(scope: Scope): RuntimeValue {
        // Make a new scope for the looping variable
        const newScope = new Scope(scope.globalScope, scope);
        this.init.evaluate(newScope);

        let ret;
        while ((this.condition as BooleanOpASTNode).evaluate(newScope)) {
            ret = this.block.evaluate(newScope);
            this.update.evaluate(newScope);
        }

        return ret;
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        throw new RuntimeError('Unexpected call to ForASTNode.compile');
    }
}

export class IfASTNode extends ASTNode {
    public condition: ASTNode;

    public block: ASTNode;

    public elseBlock?: ASTNode;

    constructor(condition: ASTNode, block: ASTNode, elseBlock?: ASTNode) {
        super(Token.IF);

        this.condition = condition;
        this.block = block;
        this.elseBlock = elseBlock;
    }

    evaluate(scope: Scope): RuntimeValue {
        if ((this.condition as BooleanOpASTNode).evaluate(scope)) {
            const ret = this.block.evaluate(scope);
            return ret;
        } else if (this.elseBlock) {
            return this.elseBlock.evaluate(scope);
        }
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        throw new RuntimeError('Unexpected call to IfASTNode.compile');
    }
}

export class FunctionDefASTNode extends ASTNode {
    public block: BlockASTNode;

    private paramList: IdentifierASTNode[];

    constructor(
        name: string,
        paramList: IdentifierASTNode[],
        block: BlockASTNode,
    ) {
        super(Token.FUNCTION);
        this.value = name;

        this.paramList = paramList;
        this.block = block;
    }

    evaluate(scope: Scope): RuntimeValue {
        scope.addFunction(this.value, this);
        return null;
    }

    callFunction(
        callersScope: Scope,
        params: ASTNode[],
        funcDef: FunctionDefASTNode,
    ): RuntimeValue {
        // Make sure the number of params line up
        if (params.length !== this.paramList.length) {
            throw new RuntimeError(
                `Function ${this.getValue()} expected ${
                    this.paramList.length
                } params, got ${params.length}`,
            );
        }

        const newScope = new Scope(callersScope.globalScope, null);
        newScope.addFunction(this.value, funcDef);

        params.forEach((param, i) => {
            const expectedParam = this.paramList[i];
            const name = expectedParam.getValue();

            newScope.addVariable(name, param.evaluate(callersScope));
        });

        return this.block.evaluate(newScope);
    }

    getParamList() {
        return this.paramList;
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        const instructions = [new AllocInstruction(this.paramList.length)];
        this.paramList.forEach((param) => {
            scope.addVariable(param.getValue());
        });

        instructions.push(...[this.block.compile(scope)].flat());
        instructions.push(new PopStackInstruction());
        instructions.push(new UnframeInstruction());

        scope.addFunction(this.value, instructions.length);
        return instructions;
    }
}

export class FunctionCallASTNode extends ASTNode {
    public args: IChildrenEnumerable[];

    constructor(name: string, args: IChildrenEnumerable[]) {
        super(Token.IDENTIFIER);
        this.value = name;
        this.args = args;
    }

    evaluate(scope: Scope): RuntimeValue {
        if (this.value in allFunctions) {
            const args = this.args.map((arg) =>
                arg.evaluate(scope),
            ) as RuntimeValue[];
            return allFunctions[this.value as BuiltInFuncName](...args);
        }

        const funcDef = scope.getFunction(this.value);
        return funcDef.callFunction(scope, this.args, funcDef);
    }

    getChildren(): IChildrenEnumerable[] {
        return this.args;
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        // Check if this is a built in function
        // TODO: Make built in functions compile
        if (this.value === 'print') {
            const instructions = [];
            // Assume there is only one argument
            const isString = this.args[0] instanceof StringASTNode;
            const arg = [this.args[0].compile(scope)].flat();
            instructions.push(...arg);
            if (!isString) {
                return [...instructions, new PrintIntInstruction()];
            }

            arg.forEach((_) => {
                instructions.push(new PrintCInstruction());
            });
            return instructions;
        }

        const { lineNumber } = scope.getFunction(this.value);
        // Set the parameters
        const instructions = [];
        this.args.forEach((arg, i) => {
            // Push the arguments onto the stack
            instructions.push(...[arg.compile(scope)].flat());

            // Store the arguments in the correct place
            instructions.push(new StoreInstruction(lineNumber + i));
        });

        // 2 to offset for the next 2 instructions
        instructions.push(new FrameInstruction(2));
        instructions.push(new PushStackInstruction());
        instructions.push(new GotoInstruction(lineNumber));

        return instructions;
    }
}

export class ReturnASTNode extends ASTNode {
    public valueAST: ASTNode;

    constructor(value: ASTNode) {
        super(Token.RETURN);
        this.valueAST = value;
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.setReturnValue(this.valueAST.evaluate(scope));
    }

    compile(scope: CompileScope): Instruction | Instruction[] {
        throw new RuntimeError('Unexpected call to ReturnASTNode.compile');
    }
}
