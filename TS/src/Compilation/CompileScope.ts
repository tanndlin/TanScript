export class CompileScope {
    // Map of name to address
    private parent: CompileScope | null = null;

    private variables: Map<string, number> = new Map();

    private functions: Map<string, { lineNumber: number; length: number }> =
        new Map();

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

        const address = this.getNumVariables(false);
        this.variables.set(name, address);
        return address;
    }

    public addFunction(name: string, length: number): void {
        if (this.functions.has(name)) {
            throw new Error(`Function ${name} already exists`);
        }

        const currentNumberOfLines = this.getTotalFunctionSize();
        this.functions.set(name, { lineNumber: currentNumberOfLines, length });
    }

    public getFunction(value: string): { lineNumber: any; length: any } {
        if (this.functions.has(value)) {
            return this.functions.get(value)!;
        }

        if (this.parent) {
            return this.parent.getFunction(value);
        }

        throw new Error(`Function ${value} not found`);
    }

    public getTotalFunctionSize() {
        let sum = 0;
        for (const { length } of this.functions.values()) {
            sum += length;
        }

        return sum;
    }
}
