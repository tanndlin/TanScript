enum Opcode {
    ADDI,
    SUBI,
    MULI,
    DIVI,
    LESS,
    LEQ,
    EQ,
    GEQ,
    GREATER,
    PUSH,
    POP,
    LOAD,
    STORE,
    ALLOC,
    FRAME,
    GOTO,
    JUMP,
    JMPT,
    JMPF,
};

char* opcodeToString(enum Opcode opcode) {
    switch (opcode) {
        case ADDI:
            return "ADDI";
        case SUBI:
            return "SUBI";
        case MULI:
            return "MULI";
        case DIVI:
            return "DIVI";
        case LESS:
            return "LESS";
        case LEQ:
            return "LEQ";
        case EQ:
            return "EQ";
        case GEQ:
            return "GEQ";
        case GREATER:
            return "GREATER";
        case PUSH:
            return "PUSH";
        case POP:
            return "POP";
        case LOAD:
            return "LOAD";
        case STORE:
            return "STORE";
        case ALLOC:
            return "ALLOC";
        case FRAME:
            return "FRAME";
        case GOTO:
            return "GOTO";
        case JUMP:
            return "JUMP";
        case JMPT:
            return "JMPT";
        case JMPF:
            return "JMPF";
    }
    return "UNKNOWN";
}
