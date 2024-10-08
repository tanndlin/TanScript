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
    ReturnInstruction,
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

    compile(scope: CompileScope): Instruction[] {
        const instructions: Instruction[] = [];
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

    compile(scope: CompileScope): Instruction[] {
        const instructions: Instruction[] = [];
        const init = this.init.compile(scope);
        const condition = this.condition.compile(scope);
        const update = this.update.compile(scope);
        const block = this.block.compile(scope);

        instructions.push(...init);
        instructions.push(...condition);
        // Jump out of block if condition is false
        instructions.push(
            new JumpFalseInstruction(block.length + update.length + 1),
        );
        instructions.push(...block);
        instructions.push(...update);
        // Jump back to condition
        instructions.push(
            new JumpInstruction(-(instructions.length - init.length + 1)),
        );

        return instructions;
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

    compile(scope: CompileScope): Instruction[] {
        const instructions: Instruction[] = [];
        const condition = [this.condition.compile(scope)].flat();
        const block = [this.block.compile(scope)].flat();
        const elseBlock = this.elseBlock
            ? [this.elseBlock.compile(scope)].flat()
            : [];

        instructions.push(...condition);
        // Jump to else block if condition is false
        instructions.push(
            new JumpFalseInstruction(block.length + (elseBlock.length ? 1 : 0)),
        );
        instructions.push(...block);
        // Jump to end of if statement
        if (elseBlock.length) {
            instructions.push(new JumpInstruction(elseBlock.length));
        }

        instructions.push(...elseBlock);

        return instructions;
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

    compile(scope: CompileScope): Instruction[] {
        const funcScope = new CompileScope(scope);
        this.paramList.forEach((param) => {
            funcScope.addVariable(param.getValue());
        });

        // Add the scope to allow recusion, we will update the length after compilation of the body
        scope.addFunction(this.value, 0);

        const instructions: Instruction[] = [];
        instructions.push(...[this.block.compile(funcScope)].flat());
        instructions.push(new PopStackInstruction());
        instructions.push(new UnframeInstruction());

        // Update the length of the function
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

    compile(scope: CompileScope): Instruction[] {
        // Check if this is a built in function
        // TODO: Make built in functions compile
        if (this.value === 'print') {
            const instructions: Instruction[] = [];
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
        const argSetup = this.args.flatMap((arg) => arg.compile(scope));

        return [
            new FrameInstruction(5 + argSetup.length), // Set the return point for the pc
            new AllocInstruction(1), // Skip over the push and alloc
            // Create the args in place
            ...argSetup,
            new AllocInstruction(-(1 + this.args.length)), // Go back to push the BP
            new PushStackInstruction(), // Offset stack for new frame
            new AllocInstruction(this.args.length), // Move stack pointer so you cannot overwrite the args
            new GotoInstruction(lineNumber), // Goto function
        ];
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

    compile(scope: CompileScope): Instruction[] {
        return [
            this.valueAST.compile(scope),
            new ReturnInstruction(),
            new PopStackInstruction(),
            new UnframeInstruction(),
        ].flat();
    }
}
