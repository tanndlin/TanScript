import { allFunctions, BuiltInFuncName } from './BuiltInFunctions';
import { CompileScope } from './Compilation/CompileScope';
import * as Instruction from './Compilation/Instruction';
import { NotImplementedError, RuntimeError, TannerError } from './errors';
import Scope from './Scope';
import {
    BooleanToken,
    ComparisonToken,
    IBooleanableAST,
    IHasChildren,
    INumberableAST,
    Iterable,
    IterableResolvable,
    Object,
    RuntimeValue,
    Token,
    TokenTypeable,
} from './types';

export class AST {
    constructor(private root: BlockASTNode) {}

    getRoot() {
        return this.root;
    }

    compile(): Instruction.Instruction[] {
        const globalScope = new CompileScope();
        return [this.root.compile(globalScope)].flat();
    }
}

export type ASTNode =
    | DecoratorASTNode
    | EOFASTNode
    | RParenASTNode
    | SemiASTNode
    | LParenASTNode
    | IdentifierASTNode
    | AssignASTNode
    | DeclarationASTNode
    | StringASTNode
    | BlockASTNode
    | ComparisonASTNode
    | BooleanASTNode
    | LessThanASTNode
    | LessEqASTNode
    | GreaterThanASTNode
    | GreaterEqASTNode
    | NotEqualASTNode
    | EqualASTNode
    | AndASTNode
    | OrASTNode
    | NotASTNode
    | WhileASTNode
    | ForASTNode
    | IfASTNode
    | FunctionDefASTNode
    | FunctionCallASTNode
    | ReturnASTNode
    | IterableASTNode
    | ListASTNode
    | ForEachASTNode
    | MathASTNode
    | AddASTNode
    | SubtractASTNode
    | MultiplyASTNode
    | DivideASTNode
    | IntegerDivideASTNode
    | ModASTNode
    | NumberASTNode
    | ObjectASTNode
    | AttributeASTNode
    | ObjectAccessAST
    | SignalAST
    | SignalAssignmentAST
    | SignalComputeAST
    | SignalComputeAssignmentAST
    | BaseASTNode;

export abstract class BaseASTNode implements TokenTypeable {
    public type: Token | undefined = undefined;

    abstract evaluate(scope: Scope): RuntimeValue;
    abstract compile(scope: CompileScope): Instruction.Instruction[];

    public isType(type: Token): boolean {
        return this.type === type;
    }

    public isOneOf(...types: Token[]): boolean {
        if (!this.type) {
            return false;
        }

        return types.includes(this.type);
    }
}

export class DecoratorASTNode extends BaseASTNode {
    evaluate(): RuntimeValue {
        throw new TannerError('Unexpected call to DecoratorASTNode.evaluate');
    }

    addChild(): void {
        throw new TannerError('Unexpected call to DecoratorASTNode.addChild');
    }

    getChildren(): BaseASTNode[] {
        return [];
    }

    compile(_scope: CompileScope): Instruction.Instruction[] {
        return [];
    }
}

export class EOFASTNode extends DecoratorASTNode {
    public type = Token.EOF;

    evaluate(): RuntimeValue {
        throw new TannerError('Unexpected call to EOF.evaluate');
    }
}

export class RParenASTNode extends DecoratorASTNode {
    public type = Token.RPAREN;

    evaluate(): RuntimeValue {
        throw new TannerError('Unexpected call to RParen.evaluate');
    }
}

export class SemiASTNode extends DecoratorASTNode {
    public type = Token.SEMI;

    evaluate(): RuntimeValue {
        throw new TannerError('Unexpected call to Semi.evaluate');
    }
}

export class LParenASTNode extends BaseASTNode {
    public type = Token.LPAREN;
    public child: BaseASTNode;

    constructor(child: BaseASTNode) {
        super();
        this.child = child;
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.child.evaluate(scope);
    }

    getChildren(): BaseASTNode[] {
        return [this.child];
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        return this.child.compile(scope);
    }
}

export class IdentifierASTNode extends BaseASTNode {
    public type = Token.IDENTIFIER;

    constructor(private name: string) {
        super();
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getVariable(this.name);
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const address = scope.getVariableAddress(this.name);
        return [new Instruction.LoadInstruction(address)];
    }

    public getName(): string {
        return this.name;
    }
}

export class AssignASTNode extends BaseASTNode {
    public type = Token.ASSIGN;
    public identifier: IdentifierASTNode;
    public valueAST: BaseASTNode;

    constructor(left: IdentifierASTNode, right: BaseASTNode) {
        super();
        this.identifier = left;
        this.valueAST = right;
    }

    evaluate(scope: Scope, isSignal = false): RuntimeValue {
        const evaluatedValue = this.valueAST.evaluate(scope);
        if (!isSignal) {
            scope.setVariable(this.identifier.getName(), evaluatedValue);
        }
        return evaluatedValue;
    }

    getChildren(): BaseASTNode[] {
        return [this.identifier, this.valueAST];
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const address = scope.getVariableAddress(this.identifier.getName());
        const valueInstructions = [this.valueAST.compile(scope)].flat();

        return valueInstructions.concat(
            new Instruction.StoreInstruction(address),
        );
    }

    public getName(): string {
        return this.identifier.getName();
    }
}

export class DeclarationASTNode extends BaseASTNode {
    public type = Token.DECLERATION;
    public child: AssignASTNode | IdentifierASTNode;

    constructor(child: AssignASTNode | IdentifierASTNode) {
        super();
        this.child = child;
    }

    evaluate(scope: Scope): RuntimeValue {
        if (this.child.isType(Token.IDENTIFIER)) {
            scope.addVariable(this.child.getName(), undefined);
            return null;
        }

        if (this.child.isType(Token.ASSIGN)) {
            const { identifier, valueAST } = this.child as AssignASTNode;

            // Special case for lambdas
            // Yes this should be a token.lambda but I'm lazy
            if (valueAST.isType(Token.FUNCTION)) {
                valueAST.evaluate(scope);
                return undefined;
            }

            const evaluatedValue = valueAST.evaluate(scope);
            scope.addVariable(identifier.getName(), evaluatedValue);
            return evaluatedValue;
        }

        if (this.child.isOneOf(Token.SIGNAL_ASSIGN, Token.COMPUTE_ASSIGN)) {
            return this.child.evaluate(scope);
        }

        throw new TannerError(
            `Unexpectd AST Type as child for decl. Got: ${this.child.type}`,
        );
    }

    getChildren(): BaseASTNode[] {
        return [this.child];
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        // The allocation is already handled by hoisting in the block scope
        if (this.child instanceof IdentifierASTNode) {
            scope.addVariable(this.child.getName());
            return [];
        }

        scope.addVariable(this.child.identifier.getName());
        return this.child.compile(scope);
    }

    public getName(): string {
        return this.child.getName();
    }
}

export class StringASTNode extends BaseASTNode {
    public type = Token.STRING;

    constructor(private value: string) {
        super();
    }

    evaluate(): string {
        return this.value;
    }

    getChildren(): BaseASTNode[] {
        return [];
    }

    compile(_scope: CompileScope): Instruction.Instruction[] {
        return this.value
            .split('')
            .map((char) => new Instruction.PushInstruction(char.charCodeAt(0)))
            .reverse();
    }

    public getValue(): string {
        return this.value;
    }
}

export class BlockASTNode extends BaseASTNode {
    public type = Token.LCURLY;
    public children: ASTNode[];

    constructor(children: ASTNode[]) {
        super();
        this.children = children;
    }

    evaluate(scope: Scope): RuntimeValue {
        const newScope = new Scope(scope.globalScope, scope);
        let retValue: RuntimeValue = undefined;

        for (const statement of this.children) {
            if (scope.isReturning()) {
                return scope.getReturnValue();
            }
            retValue = statement.evaluate(newScope);
        }

        return retValue;
    }

    addChild(node: ASTNode): void {
        this.children.push(node);
    }

    getChildren(): ASTNode[] {
        return this.children;
    }

    setChildren(children: ASTNode[]): void {
        this.children = children;
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const blockScope = new CompileScope(scope);
        let instructions = this.children
            .map((child) => child.compile(blockScope))
            .flat()
            .filter(Boolean);

        const totalFunctionLengths = blockScope.getTotalFunctionSize();
        if (totalFunctionLengths > 0) {
            instructions = [
                new Instruction.JumpInstruction(totalFunctionLengths),
                ...instructions,
            ];
        }

        const numVars = blockScope.getNumVariables(true);
        if (numVars !== 0) {
            const allocs = new Instruction.AllocInstruction(numVars);
            const unallocs = new Instruction.AllocInstruction(-numVars);
            instructions = [allocs, ...instructions, unallocs];
        }

        return instructions;
    }
}

export class ComparisonASTNode extends BaseASTNode implements IBooleanableAST {
    constructor(
        public type: ComparisonToken,
        public left: INumberableAST,
        public right: INumberableAST,
    ) {
        super();
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);
        const right = this.right.evaluate(scope);

        switch (this.type) {
            case Token.LESS:
                return left < right;
            case Token.LEQ:
                return left <= right;
            case Token.GREATER:
                return left > right;
            case Token.GEQ:
                return left >= right;
            case Token.AND:
                return !!left && !!right;
            case Token.OR:
                return !!left || !!right;
            case Token.EQUAL:
                return left === right;
            case Token.NEQ:
                return left !== right;

            default:
                throw new TannerError(`Unexpected token: ${this.type}`);
        }
    }

    getChildren(): IHasChildren[] {
        return [];
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const left = [this.left.compile(scope)].flat();
        const right = [this.right.compile(scope)].flat();

        const instructions = [...left, ...right];
        switch (this.type) {
            case Token.LESS:
                instructions.push(new Instruction.LessInstruction());
                break;
            case Token.LEQ:
                instructions.push(new Instruction.LeqInstruction());
                break;
            case Token.GREATER:
                instructions.push(new Instruction.GreaterInstruction());
                break;
            case Token.GEQ:
                instructions.push(new Instruction.GeqInstruction());
                break;
            case Token.EQUAL:
                instructions.push(new Instruction.EqInstruction());
                break;
            case Token.NEQ:
                instructions.push(new Instruction.NeqInstruction());
                break;
            case Token.AND:
                instructions.push(new Instruction.AndInstruction());
                break;
            case Token.OR:
                instructions.push(new Instruction.OrInstruction());
                break;
            default:
                throw new TannerError(`Unexpected token: ${this.type}`);
        }

        return instructions;
    }
}

export class BooleanASTNode extends BaseASTNode {
    public type: BooleanToken;

    constructor(type: BooleanToken) {
        super();
        this.type = type;
    }

    evaluate(): boolean {
        return this.isType(Token.TRUE);
    }

    compile(_scope: CompileScope): Instruction.Instruction[] {
        return [
            new Instruction.PushInstruction(this.isType(Token.TRUE) ? 1 : 0),
        ];
    }
}

export class LessThanASTNode extends ComparisonASTNode {
    public type: Token.LESS = Token.LESS;

    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.LESS, left, right);
    }
}

export class LessEqASTNode extends ComparisonASTNode {
    public type: Token.LEQ = Token.LEQ;

    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.LEQ, left, right);
    }
}

export class GreaterThanASTNode extends ComparisonASTNode {
    public type: Token.GREATER = Token.GREATER;

    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.GREATER, left, right);
    }
}

export class GreaterEqASTNode extends ComparisonASTNode {
    public type: Token.GEQ = Token.GEQ;

    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.GEQ, left, right);
    }
}

export class NotEqualASTNode extends ComparisonASTNode {
    public type: Token.NEQ = Token.NEQ;

    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.NEQ, left as INumberableAST, right as INumberableAST);
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);
        const right = this.right.evaluate(scope);

        return left !== right;
    }
}

export class EqualASTNode extends ComparisonASTNode {
    public type: Token.EQUAL = Token.EQUAL;

    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.EQUAL, left, right);
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);
        const right = this.right.evaluate(scope);

        return left === right;
    }
}

export class AndASTNode extends ComparisonASTNode {
    public type: Token.AND = Token.AND;

    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.AND, left, right);
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);

        // Short circuit
        if (!left) {
            return false;
        }

        const right = this.right.evaluate(scope);

        return !!left && !!right;
    }
}

export class OrASTNode extends ComparisonASTNode {
    public type: Token.OR = Token.OR;

    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.OR, left, right);
    }

    evaluate(scope: Scope): boolean {
        const left = this.left.evaluate(scope);

        // Short circuit
        if (left) {
            return true;
        }

        const right = this.right.evaluate(scope);

        return !!left || !!right;
    }
}

export class NotASTNode extends BaseASTNode {
    public type: Token.NOT = Token.NOT;
    constructor(public child: ASTNode) {
        super();
    }

    evaluate(scope: Scope): boolean {
        return !this.child.evaluate(scope);
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        return [...this.child.compile(scope), new Instruction.NotInstruction()];
    }
}

export class WhileASTNode extends BaseASTNode {
    public type: Token.WHILE = Token.WHILE;
    public condition: BaseASTNode;

    public block: BaseASTNode;

    constructor(condition: BaseASTNode, block: BaseASTNode) {
        super();
        this.condition = condition;
        this.block = block;
    }

    evaluate(scope: Scope): RuntimeValue {
        let ret;

        while ((this.condition as ComparisonASTNode).evaluate(scope)) {
            ret = this.block.evaluate(scope);
        }

        return ret;
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const instructions: Instruction.Instruction[] = [];
        const condition = [this.condition.compile(scope)].flat();
        const block = [this.block.compile(scope)].flat();

        instructions.push(...condition);
        // Jump out of block if condition is false
        instructions.push(
            new Instruction.JumpFalseInstruction(block.length + 1),
        );
        instructions.push(...block);
        // Jump back to condition
        instructions.push(
            new Instruction.JumpInstruction(-instructions.length - 1),
        );

        return instructions;
    }
}

export class ForASTNode extends BaseASTNode {
    public type: Token.FOR = Token.FOR;

    constructor(
        public init: ASTNode,
        public condition: ASTNode,
        public update: ASTNode,
        public block: ASTNode,
    ) {
        super();
    }

    evaluate(scope: Scope): RuntimeValue {
        // Make a new scope for the looping variable
        const newScope = new Scope(scope.globalScope, scope);
        this.init.evaluate(newScope);

        let ret;
        while ((this.condition as ComparisonASTNode).evaluate(newScope)) {
            ret = this.block.evaluate(newScope);
            this.update.evaluate(newScope);
        }

        return ret;
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const instructions: Instruction.Instruction[] = [];
        const init = this.init.compile(scope);
        const condition = this.condition.compile(scope);
        const update = this.update.compile(scope);
        const block = this.block.compile(scope);

        instructions.push(...init);
        instructions.push(...condition);
        // Jump out of block if condition is false
        instructions.push(
            new Instruction.JumpFalseInstruction(
                block.length + update.length + 1,
            ),
        );
        instructions.push(...block);
        instructions.push(...update);
        // Jump back to condition
        instructions.push(
            new Instruction.JumpInstruction(
                -(instructions.length - init.length + 1),
            ),
        );

        return instructions;
    }
}

export class IfASTNode extends BaseASTNode {
    public type: Token.IF = Token.IF;

    constructor(
        public condition: ASTNode,
        public block: ASTNode,
        public elseBlock?: ASTNode,
    ) {
        super();

        this.condition = condition;
        this.block = block;
        this.elseBlock = elseBlock;
    }

    evaluate(scope: Scope): RuntimeValue {
        if ((this.condition as ComparisonASTNode).evaluate(scope)) {
            const ret = this.block.evaluate(scope);
            return ret;
        } else if (this.elseBlock) {
            return this.elseBlock.evaluate(scope);
        }
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const instructions: Instruction.Instruction[] = [];
        const condition = [this.condition.compile(scope)].flat();
        const block = [this.block.compile(scope)].flat();
        const elseBlock = this.elseBlock
            ? [this.elseBlock.compile(scope)].flat()
            : [];

        instructions.push(...condition);
        const jumpToElse = new Instruction.JumpFalseInstruction(
            block.length + (elseBlock.length ? 1 : 0),
        );
        instructions.push(jumpToElse);
        instructions.push(...block);
        if (elseBlock.length) {
            const jumpOverElse = new Instruction.JumpInstruction(
                elseBlock.length,
            );
            instructions.push(jumpOverElse);
        }

        instructions.push(...elseBlock);

        return instructions;
    }
}

export class FunctionDefASTNode extends BaseASTNode {
    public block: BlockASTNode;

    private paramList: IdentifierASTNode[];

    public type: Token.FUNCTION = Token.FUNCTION;

    constructor(
        private name: string,
        paramList: IdentifierASTNode[],
        block: BlockASTNode,
    ) {
        super();
        this.paramList = paramList;
        this.block = block;
    }

    evaluate(scope: Scope): RuntimeValue {
        scope.addFunction(this.name, this);
        return null;
    }

    callFunction(
        callersScope: Scope,
        params: BaseASTNode[],
        funcDef: FunctionDefASTNode,
    ): RuntimeValue {
        // Make sure the number of params line up
        if (params.length !== this.paramList.length) {
            throw new RuntimeError(
                `Function ${this.name} expected ${
                    this.paramList.length
                } params, got ${params.length}`,
            );
        }

        const newScope = new Scope(callersScope.globalScope, null);
        newScope.addFunction(this.name, funcDef);

        params.forEach((param, i) => {
            const expectedParam = this.paramList[i];
            const name = expectedParam.getName();

            newScope.addVariable(name, param.evaluate(callersScope));
        });

        return this.block.evaluate(newScope);
    }

    getParamList() {
        return this.paramList;
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const funcScope = new CompileScope(scope);
        this.paramList.forEach((param) => {
            funcScope.addVariable(param.getName());
        });

        // Add the scope to allow recusion, we will update the length after compilation of the body
        scope.addFunction(this.name, 0);

        const instructions: Instruction.Instruction[] = [];
        instructions.push(...[this.block.compile(funcScope)].flat());
        instructions.push(new Instruction.PopStackInstruction());
        instructions.push(new Instruction.UnframeInstruction());

        // Update the length of the function
        scope.addFunction(this.name, instructions.length);
        return instructions;
    }
}

export class FunctionCallASTNode extends BaseASTNode {
    public type: Token.IDENTIFIER = Token.IDENTIFIER;
    public args: ASTNode[];

    constructor(
        private name: string,
        args: ASTNode[],
    ) {
        super();
        this.args = args;
    }

    evaluate(scope: Scope): RuntimeValue {
        if (this.name in allFunctions) {
            const args = this.args.map((arg) =>
                arg.evaluate(scope),
            ) as RuntimeValue[];
            return allFunctions[this.name as BuiltInFuncName](...args);
        }

        const funcDef = scope.getFunction(this.name);
        return funcDef.callFunction(scope, this.args, funcDef);
    }

    getChildren(): ASTNode[] {
        return this.args;
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        // Check if this is a built in function
        // TODO: Make built in functions compile
        if (this.name === 'print') {
            const instructions: Instruction.Instruction[] = [];
            // Assume there is only one argument
            const isString = this.args[0] instanceof StringASTNode;
            const arg = [this.args[0].compile(scope)].flat();
            instructions.push(...arg);
            if (!isString) {
                return [...instructions, new Instruction.PrintIntInstruction()];
            }

            arg.forEach((_) => {
                instructions.push(new Instruction.PrintCInstruction());
            });
            return instructions;
        }

        const { lineNumber } = scope.getFunction(this.name);
        // Set the parameters
        const argSetup = this.args.flatMap((arg) => arg.compile(scope));

        return [
            new Instruction.FrameInstruction(5 + argSetup.length), // Set the return point for the pc
            new Instruction.AllocInstruction(1), // Skip over the push and alloc
            // Create the args in place
            ...argSetup,
            new Instruction.AllocInstruction(-(1 + this.args.length)), // Go back to push the BP
            new Instruction.PushStackInstruction(), // Offset stack for new frame
            new Instruction.AllocInstruction(this.args.length), // Move stack pointer so you cannot overwrite the args
            new Instruction.GotoInstruction(lineNumber), // Goto function
        ];
    }

    public getName() {
        return this.name;
    }
}

export class ReturnASTNode extends BaseASTNode {
    public type: Token.RETURN = Token.RETURN;
    public valueAST: BaseASTNode;

    constructor(value: BaseASTNode) {
        super();
        this.valueAST = value;
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.setReturnValue(this.valueAST.evaluate(scope));
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        return [
            this.valueAST.compile(scope),
            new Instruction.ReturnInstruction(),
            new Instruction.PopStackInstruction(),
            new Instruction.UnframeInstruction(),
        ].flat();
    }
}

class Iterator {
    protected items: RuntimeValue[];
    private index: number;

    constructor(items: RuntimeValue[]) {
        this.items = items;
        this.index = 0;
    }

    hasNext(): boolean {
        return this.index < this.items.length;
    }

    next(): RuntimeValue {
        return this.items[this.index++];
    }

    reset(): void {
        this.index = 0;
    }

    length(): number {
        return this.items.length;
    }
}

export class IterableASTNode extends BaseASTNode {
    public type: Token.LBRACKET = Token.LBRACKET;
    public items: BaseASTNode[];

    constructor(items: BaseASTNode[]) {
        super();
        this.items = items;
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.items.map((child) => child.evaluate(scope));
    }

    createIterator(scope: Scope): Iterator {
        return new Iterator(this.items.map((child) => child.evaluate(scope)));
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }
}

export class ListASTNode extends IterableASTNode {}

export class ForEachASTNode extends BaseASTNode {
    public init: DeclarationASTNode | IdentifierASTNode;
    public iterable: IterableResolvable;
    public block: BlockASTNode;
    public type: Token.FOREACH = Token.FOREACH;

    constructor(
        init: DeclarationASTNode | IdentifierASTNode,
        iterable: IterableResolvable,
        block: BlockASTNode,
    ) {
        super();
        this.init = init;
        this.iterable = iterable;
        this.block = block;
    }

    evaluate(scope: Scope): RuntimeValue {
        let iterator: Iterator;
        if (this.iterable instanceof IdentifierASTNode) {
            const items = scope.getVariable(
                this.iterable.getName(),
            ) as Iterable;
            iterator = new Iterator(items);
        } else {
            iterator = (this.iterable as IterableASTNode).createIterator(scope);
        }

        let ret;
        while (iterator.hasNext() && !scope.isReturning()) {
            const curItem = iterator.next();

            // Add the current item to the scope
            const newScope = new Scope(scope.globalScope, scope);
            newScope.addVariable(this.init.getName(), curItem);
            ret = this.block.evaluate(newScope);
        }

        return ret;
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }
}

export class MathASTNode
    extends BaseASTNode
    implements INumberableAST, IHasChildren
{
    public left: INumberableAST;
    public right: INumberableAST;

    constructor(
        public type: Token,
        left: INumberableAST,
        right: INumberableAST,
    ) {
        super();
        this.left = left;
        this.right = right;
    }

    evaluate(scope: Scope): number {
        const left = (this.left as INumberableAST).evaluate(scope);
        const right = (this.right as INumberableAST).evaluate(scope);

        switch (this.type) {
            case Token.PLUS:
                return this.handleAddition(left, right);
            case Token.MINUS:
                return left - right;
            case Token.MULTIPLY:
                return left * right;
            case Token.DIVIDE:
                return left / right;
            case Token.INT_DIVIDE:
                return Math.floor(left / right);
            case Token.MOD:
                return left % right;
            default:
                throw new TannerError(`Unexpected token: ${this.type}`);
        }
    }

    handleAddition(left: any, right: any): any {
        // Special case to handle array concatenation
        if (left instanceof Array && right instanceof Array) {
            return left.concat(right);
        }

        return left + right;
    }

    getChildren(): INumberableAST[] {
        return [this.left, this.right];
    }

    compile(scope: CompileScope): Instruction.Instruction[] {
        const instructions = this.left
            .compile(scope)
            .concat(this.right.compile(scope));

        if (this.type === Token.PLUS) {
            instructions.push(new Instruction.AddInstruction());
        } else if (this.type === Token.MINUS) {
            instructions.push(new Instruction.SubInstruction());
        } else if (this.type === Token.MULTIPLY) {
            instructions.push(new Instruction.MulInstruction());
        } else if (this.type === Token.DIVIDE) {
            instructions.push(new Instruction.DivInstruction());
        } else if (this.type === Token.MOD) {
            instructions.push(new Instruction.ModInstruction());
        } else {
            throw new TannerError('Unexpected call to MathASTNode.compile');
        }

        return instructions.flat();
    }
}

export class AddASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.PLUS, left, right);
    }
}

export class SubtractASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.MINUS, left, right);
    }
}

export class MultiplyASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.MULTIPLY, left, right);
    }
}

export class DivideASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.DIVIDE, left, right);
    }
}

export class IntegerDivideASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.INT_DIVIDE, left, right);
    }
}

export class ModASTNode extends MathASTNode {
    constructor(left: INumberableAST, right: INumberableAST) {
        super(Token.MOD, left, right);
    }
}

export class NumberASTNode extends BaseASTNode implements INumberableAST {
    public static TYPE = Token.NUMBER;
    public type: Token.NUMBER = Token.NUMBER;

    constructor(private value: number) {
        super();
    }

    getChildren(): IHasChildren[] {
        return [];
    }

    evaluate(): number {
        return this.value;
    }

    compile(_scope: CompileScope): Instruction.Instruction[] {
        return [new Instruction.PushInstruction(this.value)];
    }

    public getValue(): number {
        return this.value;
    }

    public static isNumberAST(ast: BaseASTNode): ast is NumberASTNode {
        return ast.type === this.TYPE;
    }
}

export class ObjectASTNode extends BaseASTNode {
    public type: Token.LCURLY = Token.LCURLY;
    public attributes: AttributeASTNode[];

    constructor(attributes: AttributeASTNode[]) {
        super();
        this.attributes = attributes;
    }

    evaluate(scope: Scope): RuntimeValue {
        const obj: Object = { attributes: {}, methods: {} };
        this.attributes.forEach((attribute: AttributeASTNode) => {
            obj.attributes[attribute.getName()] = attribute.evaluate(scope);
        });

        return obj;
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }
}

export class AttributeASTNode extends BaseASTNode {
    public valueAST: BaseASTNode;

    constructor(
        private name: string,
        valueAST: BaseASTNode,
    ) {
        super();
        this.valueAST = valueAST;
    }

    evaluate(scope: Scope): RuntimeValue {
        return this.valueAST.evaluate(scope);
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }

    public getName(): string {
        return this.name;
    }
}

export class ObjectAccessAST extends BaseASTNode {
    public objIdentifier: IdentifierASTNode;
    public attribute: IdentifierASTNode;

    public type: Token.IDENTIFIER = Token.IDENTIFIER;
    constructor(
        objIdentifier: IdentifierASTNode,
        attribute: IdentifierASTNode,
    ) {
        super();
        this.objIdentifier = objIdentifier;
        this.attribute = attribute;
    }

    evaluate(scope: Scope): RuntimeValue {
        const obj = scope.getVariable<Object>(this.objIdentifier.getName());

        return obj.attributes[this.attribute.getName()];
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }
}

export class SignalAST extends BaseASTNode {
    public type: Token.SIGNAL = Token.SIGNAL;
    private name: string;

    constructor(name: string) {
        super();
        this.name = name.replace('#', '');
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getSignalValue(this.name);
    }

    compile(_scope: CompileScope): never {
        throw new NotImplementedError('Method not implemented.');
    }

    public getName(): string {
        return this.getName();
    }
}

export class SignalAssignmentAST extends AssignASTNode {
    public type: Token.SIGNAL_ASSIGN = Token.SIGNAL_ASSIGN;
    public name: string;

    constructor(identifier: IdentifierASTNode, value: BaseASTNode) {
        super(identifier, value);

        this.name = identifier.getName().replace('#', '');
    }

    public evaluate(scope: Scope): RuntimeValue {
        const value = this.valueAST.evaluate(scope);

        scope.setSignal(this.name, value);
        return value;
    }
}

export class SignalComputeAST extends BaseASTNode {
    public type: Token.SIGNAL = Token.SIGNAL;
    private value: string;

    constructor(name: string) {
        super();
        this.value = name.replace('$', '');
    }

    evaluate(scope: Scope): RuntimeValue {
        return scope.getSignalValue(this.value);
    }

    compile(_scope: CompileScope): Instruction.Instruction[] {
        throw new NotImplementedError('Method not implemented.');
    }
}

export class SignalComputeAssignmentAST extends AssignASTNode {
    public type: Token.COMPUTE_ASSIGN = Token.COMPUTE_ASSIGN;
    public name: string;

    constructor(left: IdentifierASTNode, right: BaseASTNode) {
        super(left, right);

        this.name = this.identifier.getName();
        this.name = this.identifier.getName().replace('$', '');
    }

    public evaluate(scope: Scope): RuntimeValue {
        scope.setSignalCompute(this.name, this);
    }
}
