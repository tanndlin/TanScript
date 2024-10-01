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

export class ModInstruction extends Instruction {
    constructor() {
        super(Opcode.MODI, []);
    }
}

export class LessInstruction extends Instruction {
    constructor() {
        super(Opcode.LESS, []);
    }
}

export class LeqInstruction extends Instruction {
    constructor() {
        super(Opcode.LEQ, []);
    }
}

export class EqInstruction extends Instruction {
    constructor() {
        super(Opcode.EQ, []);
    }
}

export class GeqInstruction extends Instruction {
    constructor() {
        super(Opcode.GEQ, []);
    }
}

export class GreaterInstruction extends Instruction {
    constructor() {
        super(Opcode.GREATER, []);
    }
}

export class AndInstruction extends Instruction {
    constructor() {
        super(Opcode.AND, []);
    }
}

export class OrInstruction extends Instruction {
    constructor() {
        super(Opcode.OR, []);
    }
}

export class NotInstruction extends Instruction {
    constructor() {
        super(Opcode.NOT, []);
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

export class FrameInstruction extends Instruction {
    constructor(offset = 0) {
        super(Opcode.FRAME, [offset]);
    }
}

export class UnframeInstruction extends Instruction {
    constructor() {
        super(Opcode.UNFRAME, []);
    }
}

export class StoreSPInstruction extends Instruction {
    constructor() {
        super(Opcode.STORESP, []);
    }
}

export class StoreSPOffsetInstruction extends Instruction {
    constructor() {
        super(Opcode.STORESPOFFSET, []);
    }
}

export class StoreStackInstruction extends Instruction {
    constructor() {
        super(Opcode.STORESTACK, []);
    }
}

export class GotoInstruction extends Instruction {
    constructor(number: number) {
        super(Opcode.GOTO, [number]);
    }
}

export class JumpInstruction extends Instruction {
    constructor(number: number) {
        super(Opcode.JUMP, [number]);
    }
}

export class JumpTrueInstruction extends Instruction {
    constructor(number: number) {
        super(Opcode.JMPT, [number]);
    }
}

export class JumpFalseInstruction extends Instruction {
    constructor(number: number) {
        super(Opcode.JMPF, [number]);
    }
}

export class PrintCInstruction extends Instruction {
    constructor() {
        super(Opcode.PRINTC, []);
    }
}

export class PrintIntInstruction extends Instruction {
    constructor() {
        super(Opcode.PRINTINT, []);
    }
}

export class PushStackInstruction extends Instruction {
    constructor() {
        super(Opcode.PUSHSTACK, []);
    }
}

export class PopStackInstruction extends Instruction {
    constructor() {
        super(Opcode.POPSTACK, []);
    }
}

export class ReturnInstruction extends Instruction {
    constructor() {
        super(Opcode.RETURN, []);
    }
}
