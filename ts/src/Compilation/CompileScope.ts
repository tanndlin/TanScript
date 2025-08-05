import { CompilerError } from '../errors';
import { Register } from '../types';
import { registerOrder } from '../util';

export class CompileScope {
    // Map of name to address
    private parent: CompileScope | null = null;
    private variables: Map<string, number> = new Map();
    private static readonly registers: Map<Register, boolean> = new Map();
    private static readonly priorityRegistersUsed: Set<Register> = new Set();
    public static readonly data: string[] = [];
    public static uniqueIdCounter: number = 0;
    public offset: number;

    constructor(parent: CompileScope | null = null) {
        this.parent = parent;
        if (!parent) {
            registerOrder.forEach((reg) =>
                CompileScope.registers.set(reg, false),
            );
            this.offset = 0;
        } else {
            this.offset = parent.offset + parent.variables.size * 8; // Assuming 64-bit addressing
        }
    }

    public getVariableAddress(name: string): number {
        if (this.variables.has(name)) {
            return this.variables.get(name)! + this.offset;
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
        if (!req) {
            return CompileScope.LeaseRandomRegister();
        }

        // Check the list of non-allocatable registers
        if (!CompileScope.registers.has(req)) {
            if (CompileScope.priorityRegistersUsed.has(req)) {
                throw new CompilerError(
                    `Attempted to lease non-allocatable register (reg: ${req})`,
                );
            }

            CompileScope.priorityRegistersUsed.add(req);
            return req;
        }

        if (CompileScope.registers.get(req)) {
            throw new Error(
                `Attempted to lease allocated register (reg: ${req})`,
            );
        }

        CompileScope.registers.set(req, true);
        return req;
    }

    private static LeaseRandomRegister(): Register {
        const reg = CompileScope.registers.entries().find(([_, used]) => !used);

        if (!reg) {
            throw new Error('All registers leased');
        }

        CompileScope.registers.set(reg[0], true);
        return reg[0];
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
                CompileScope.registers.set(reg, false);
            } else {
                this.priorityRegistersUsed.delete(reg);
            }
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
        CompileScope.data.push(`\t${dataName} db ${fmt}, 10, 0`);
        return dataName;
    }

    public static GetUniqueId(): number {
        return CompileScope.uniqueIdCounter++;
    }
}
