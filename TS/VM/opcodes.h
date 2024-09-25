enum Opcode {
    ADDI,
    SUBI,
    MULI,
    DIVI,
    MODI,
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
    UNFRAME,
    STORESP,
    STORESPOFFSET,
    STORESTACK,
    GOTO,
    JUMP,
    JMPT,
    JMPF,
    PRINTC,
    PRINTINT,
    PUSHSTACK,
    POPSTACK,
    RETURN,
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
        case MODI:
            return "MODI";
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
        case UNFRAME:
            return "UNFRAME";
        case STORESP:
            return "STORESP";
        case STORESPOFFSET:
            return "STORESPOFFSET";
        case STORESTACK:
            return "STORESTACK";
        case GOTO:
            return "GOTO";
        case JUMP:
            return "JUMP";
        case JMPT:
            return "JMPT";
        case JMPF:
            return "JMPF";
        case PRINTC:
            return "PRINTC";
        case PRINTINT:
            return "PRINTINT";
        case PUSHSTACK:
            return "PUSHSTACK";
        case POPSTACK:
            return "POPSTACK";
        case RETURN:
            return "RETURN";
    }
    return "UNKNOWN";
}
