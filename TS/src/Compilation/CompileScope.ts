export class CompileScope {
    // Map of name to address
    private variables: Map<string, number> = new Map();

    public getNumVariables(): number {
        return this.variables.size;
    }

    public getVariableAddress(name: string): number {
        if (this.variables.has(name)) {
            return this.variables.get(name) as number;
        }
        throw new Error(`Variable ${name} not found`);
    }

    public addVariable(name: string): number {
        if (this.variables.has(name)) {
            throw new Error(`Variable ${name} already exists`);
        }

        const address = this.getNumVariables();
        this.variables.set(name, address);
        return address;
    }
}
