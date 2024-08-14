import { Opcode } from './Opcodes';

export abstract class Instruction {
    public opcode: Opcode;

    public operands: number[];

    constructor(opcode: Opcode, operands: number[]) {
        this.opcode = opcode;
        this.operands = operands;
    }

    public toString(): string {
        return `${this.opcode} ${this.operands.join(' ')}`;
    }
}

export class AddInstruction extends Instruction {
    constructor() {
        super(Opcode.ADDI, []);
    }
}

export class SubInstruction extends Instruction {
    constructor() {
        super(Opcode.SUBI, []);
    }
}

export class MulInstruction extends Instruction {
    constructor() {
        super(Opcode.MULI, []);
    }
}

export class DivInstruction extends Instruction {
    constructor() {
        super(Opcode.DIVI, []);
    }
}

export class PushInstruction extends Instruction {
    constructor(number: number) {
        super(Opcode.PUSH, [number]);
    }
}

export class PopInstruction extends Instruction {
    constructor() {
        super(Opcode.POP, []);
    }
}

export class LoadInstruction extends Instruction {
    constructor(number: number) {
        super(Opcode.LOAD, [number]);
    }
}

export class StoreInstruction extends Instruction {
    constructor(number: number) {
        super(Opcode.STORE, [number]);
    }
}

export class AllocInstruction extends Instruction {
    constructor(number: number) {
        super(Opcode.ALLOC, [number]);
    }
}
