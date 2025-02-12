import { ASTFunctionDef } from './AST';
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
    private scopes: Map<string, Scope>;
    private isGlobalScope: boolean;
    private returning: boolean = false;
    private returnedValue: RuntimeValue | null = null;

    constructor(globalScope: Scope | null, parent: Scope | null) {
        this.globalScope = globalScope ?? this;
        this.parent = parent;
        this.variables = new Map();
        this.scopes = new Map();

        this.isGlobalScope = !globalScope;
    }

    getVariable<T>(name: string): T {
        if (this.variables.has(name)) {
            return this.variables.get(name) as T;
        }
        let scope: Scope | null = this.parent;
        while (scope) {
            if (scope.variables.has(name)) {
                return scope.variables.get(name) as T;
            }
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
                `Variable ${name} already declared. This should not happen unless the Engine is coded incorrectly.`,
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
            `Cannot set value for variable ${name} before declaration`,
        );
    }

    addFunction(name: string, f: ASTFunctionDef) {
        this.variables.set(name, f);
    }

    getFunction(name: string): ASTFunctionDef {
        if (this.variables.has(name)) {
            return this.variables.get(name) as ASTFunctionDef;
        }
        let scope: Scope | null = this.parent;
        while (scope) {
            if (scope.variables.has(name)) {
                return scope.variables.get(name) as ASTFunctionDef;
            }
            scope = scope.parent;
        }

        if (this.globalScope && this.globalScope.variables.has(name)) {
            return this.globalScope.variables.get(name) as ASTFunctionDef;
        }

        throw new UndeclaredFunctionError(`Function ${name} not found`);
    }

    setReturnValue(value: RuntimeValue) {
        this.returning = true;
        this.returnedValue = value;
        this.parent?.setReturnValue(value);

        return value;
    }

    getReturnValue() {
        return this.returnedValue;
    }

    isReturning() {
        return this.returning;
    }
}
