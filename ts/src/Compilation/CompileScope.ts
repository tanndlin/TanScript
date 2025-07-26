import { Register } from '../types';
import { registerOrder } from '../util';

export class CompileScope {
    // Map of name to address
    private parent: CompileScope | null = null;
    private variables: Map<string, Register> = new Map();
    private static readonly registers: Register[] = [];
    public static readonly data: string[] = [];

    constructor(parent: CompileScope | null = null) {
        this.parent = parent;
        if (!parent) {
            CompileScope.registers.push(...registerOrder);
            console.log(`Registers: ${CompileScope.registers.join(', ')}`);
        }
    }

    public getVariableAddress(name: string): Register {
        if (this.variables.has(name)) {
            return this.variables.get(name)!;
        }

        if (this.parent) {
            return this.parent.getVariableAddress(name);
        }

        throw new Error(`Variable ${name} not found`);
    }

    public addVariable(name: string): Register {
        if (this.variables.has(name)) {
            throw new Error(`Variable ${name} already exists`);
        }

        const address = CompileScope.LeaseRegister();
        this.variables.set(name, address);
        return address;
    }

    public static LeaseRegister(req?: Register): Register {
        if (CompileScope.registers.length === 0) {
            throw new Error('No registers available');
        }

        if (req) {
            if (!CompileScope.registers.includes(req)) {
                throw new Error(`Register ${req} is not available`);
            }

            const index = CompileScope.registers.indexOf(req);
            CompileScope.registers.splice(index, 1);
            return req;
        }

        return CompileScope.registers.shift()!;
    }

    public static LeaseRegisters(numRegs: number): Register[] {
        return Array.from({ length: numRegs }, () =>
            CompileScope.LeaseRegister(),
        );
    }

    public static ReleaseRegister(register: Register): void {
        CompileScope.registers.push(register);
    }

    public static addData(fmt: string) {
        const dataName = `data_${CompileScope.data.length}`;
        CompileScope.data.push(`\t${dataName} db ${fmt}, 0`);
        return dataName;
    }
}
