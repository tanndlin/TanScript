import { AssignASTNode } from './AST/AST';
import { FunctionDefASTNode } from './AST/ControlAST';
import { ComputedSignal, Signal } from './Signal';
import {
    TannerError,
    UndeclaredFunctionError,
    UndeclaredVariableError,
    UseBeforeDeclarationError,
} from './errors';
import { RuntimeValue } from './types';

export default class Scope {
    public globalScope: Scope | null;
    private parent: Scope | null;
    private variables: Map<string, any>;
    private signals: Map<string, Signal>;
    private scopes: Map<string, Scope>;

    private isGlobalScope: boolean;

    private returning: boolean = false;
    private returnedValue: RuntimeValue | null = null;

    constructor(globalScope: Scope | null, parent: Scope | null) {
        this.globalScope = globalScope ?? this;
        this.parent = parent;
        this.variables = new Map();
        this.scopes = new Map();
        this.signals = new Map();

        this.isGlobalScope = !globalScope;
    }

    getVariable<T>(name: string): T {
        if (this.variables.has(name)) return this.variables.get(name) as T;
        let scope: Scope | null = this.parent;
        while (scope) {
            if (scope.variables.has(name))
                return scope.variables.get(name) as T;
            scope = scope.parent;
        }

        // Check the global scope
        if (this.globalScope && this.globalScope.variables.has(name)) {
            return this.globalScope.variables.get(name) as T;
        }

        throw new UndeclaredVariableError(`Variable ${name} not found`);
    }

    addScope(name: string, scope: Scope) {
        this.scopes.set(name, scope);
        scope.parent = this;
    }

    addVariable(name: string, value: RuntimeValue) {
        if (this.variables.has(name)) {
            throw new TannerError(
                `Variable ${name} already declared. This should not happen unless the Engine is coded incorrectly.`
            );
        }

        this.variables.set(name, value);
    }

    setVariable(name: string, value: RuntimeValue) {
        // Check this scope and all parents for the variable
        let curScope: Scope | null = this;
        while (curScope) {
            if (curScope.variables.has(name)) {
                curScope.variables.set(name, value);
                return;
            }
            curScope = curScope.parent;
        }

        if (this.globalScope && this.globalScope.variables.has(name)) {
            this.globalScope.variables.set(name, value);
            return;
        }

        throw new UseBeforeDeclarationError(
            `Cannot set value for variable ${name} before declaration`
        );
    }

    addFunction(name: string, f: FunctionDefASTNode) {
        this.variables.set(name, f);
    }

    getFunction(name: string): FunctionDefASTNode {
        if (this.variables.has(name))
            return this.variables.get(name) as FunctionDefASTNode;
        let scope: Scope | null = this.parent;
        while (scope) {
            if (scope.variables.has(name))
                return scope.variables.get(name) as FunctionDefASTNode;
            scope = scope.parent;
        }

        if (this.globalScope && this.globalScope.variables.has(name)) {
            return this.globalScope.variables.get(name) as FunctionDefASTNode;
        }

        throw new UndeclaredFunctionError(`Function ${name} not found`);
    }

    setReturnValue(value: RuntimeValue) {
        this.returning = true;
        this.returnedValue = value;
        this.parent?.setReturnValue(value);
    }

    getReturnValue() {
        return this.returnedValue;
    }

    isReturning() {
        return this.returning;
    }

    getSignal(name: string): Signal {
        if (this.signals.has(name)) {
            return this.signals.get(name)!;
        }

        if (!this.parent) {
            throw new UndeclaredVariableError(`Signal ${name} not found`);
        }

        return this.parent.getSignal(name);
    }

    getSignalValue(name: string): RuntimeValue {
        const signal = this.getSignal(name);
        if (!signal)
            throw new UndeclaredVariableError(`Signal ${name} not found`);

        return signal.getValue();
    }

    setSignal(identifier: string, value: RuntimeValue) {
        let signal: Signal;
        try {
            signal = this.getSignal(identifier);
            signal.value = value;
            signal.markChildrenDirty();
        } catch (e) {
            signal = new Signal(identifier, value);
        }

        this.signals.set(identifier, signal);
    }

    setSignalCompute(identifier: string, assignAST: AssignASTNode) {
        let signal: Signal;
        try {
            signal = this.getSignal(identifier);
        } catch (e) {
            signal = new ComputedSignal(this, identifier, assignAST);
        }

        signal.markChildrenDirty();

        this.signals.set(identifier, signal);
    }
}
