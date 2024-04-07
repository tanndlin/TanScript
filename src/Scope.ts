import {
    TannerError,
    UndeclaredVariableError,
    UseBeforeDeclarationError,
} from './errors';
import { RuntimeValue } from './types';

export default class Scope {
    private parent: Scope | null;
    private variables: Map<string, any>;
    private scopes: Map<string, Scope>;

    constructor(parent: Scope | null) {
        this.parent = parent;
        this.variables = new Map();
        this.scopes = new Map();
    }

    getVariable<T>(name: string): T {
        if (this.variables.has(name)) return this.variables.get(name) as T;
        let scope: Scope | null = this.parent;
        while (scope) {
            if (scope.variables.has(name))
                return scope.variables.get(name) as T;
            scope = scope.parent;
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
        if (!this.variables.has(name)) {
            throw new UseBeforeDeclarationError(
                `Cannot set value for variable ${name} before declaration`
            );
        }

        this.variables.set(name, value);
    }
}
