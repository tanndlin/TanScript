import { Register } from '../types';
import { registerOrder } from '../util';

export class CompileScope {
    // Map of name to address
    private parent: CompileScope | null = null;
    private variables: Map<string, number> = new Map();
    private static readonly registersUsed: Set<Register> = new Set();
    private static readonly leaseableRegisters: Register[] = [];
    public static readonly data: string[] = [];
    public static uniqueIdCounter: number = 0;

    constructor(parent: CompileScope | null = null) {
        this.parent = parent;
        if (!parent) {
            CompileScope.leaseableRegisters.push(...registerOrder);
        }
    }

    public getVariableAddress(name: string): number {
        if (this.variables.has(name)) {
            return this.variables.get(name)!;
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

        const address = this.variables.size * 8; // Assuming 64-bit addressing
        this.variables.set(name, address);
        return address;
    }

    public getNumVariables(): number {
        return this.variables.size;
    }

    private static LeaseRegister(req?: Register): Register {
        if (req) {
            if (CompileScope.registersUsed.has(req)) {
                throw new Error(
                    `Attempted to lease allocated register (reg: ${req})`,
                );
            }

            CompileScope.registersUsed.add(req);
            const index = CompileScope.leaseableRegisters.indexOf(req);
            if (index !== -1) {
                CompileScope.leaseableRegisters.splice(index, 1);
            }
            return req;
        }

        if (CompileScope.leaseableRegisters.length === 0) {
            throw new Error('All registers leased');
        }

        const reg = CompileScope.leaseableRegisters.shift()!;
        CompileScope.registersUsed.add(reg);
        return reg;
    }

    private static LeaseRegisters(registers: Register[]): Register[] {
        return registers.map(CompileScope.LeaseRegister);
    }

    private static LeaseRandomRegisters(numRegs: number): Register[] {
        return Array.from({ length: numRegs }, () =>
            CompileScope.LeaseRegister(),
        );
    }

    public static ReleaseRegister(...registers: Register[]): void {
        for (const reg of registers) {
            if (registerOrder.includes(reg)) {
                CompileScope.leaseableRegisters.push(reg);
            }

            CompileScope.registersUsed.delete(reg);
        }
    }

    public static LeaseRegistersWithScope<T>(
        callback: (...regs: Register[]) => T,
        ...registers: Register[]
    ) {
        CompileScope.LeaseRegisters(registers);
        const ret = callback(...registers);
        CompileScope.ReleaseRegister(...registers);
        return ret;
    }

    public static LeaseRandomRegistersWithScope<T>(
        callback: (...regs: Register[]) => T,
        numRegisters: number = 1,
    ) {
        const regs = CompileScope.LeaseRandomRegisters(numRegisters);
        const ret = callback(...regs);
        CompileScope.ReleaseRegister(...regs);
        return ret;
    }

    public static addData(fmt: string) {
        const dataName = `data_${CompileScope.data.length}`;
        CompileScope.data.push(`\t${dataName} db ${fmt}, 0`);
        return dataName;
    }

    public static GetUniqueId(): number {
        return CompileScope.uniqueIdCounter++;
    }
}
