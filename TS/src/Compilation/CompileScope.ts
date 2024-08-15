export class CompileScope {
    // Map of name to address
    private parent: CompileScope | null = null;

    private variables: Map<string, number> = new Map();

    constructor(parent: CompileScope | null = null) {
        this.parent = parent;
    }

    public getNumVariables(currentScope: boolean): number {
        if (currentScope) {
            return this.variables.size;
        }

        return (
            this.variables.size +
            (this.parent ? this.parent.getNumVariables(currentScope) : 0)
        );
    }

    public getVariableAddress(name: string): number {
        if (this.variables.has(name)) {
            return this.variables.get(name) as number;
        }

        if (this.parent) {
            return this.parent.getVariableAddress(name);
        }

        throw new Error(`Variable ${name} not found`);
    }

    public addVariable(name: string): number {
        if (this.variables.has(name)) {
            throw new Error(`Variable ${name} already exists`);
        }

        const address = this.getNumVariables(true);
        this.variables.set(name, address);
        return address;
    }
}
