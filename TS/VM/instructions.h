#include <stdio.h>

#include "opcodes.h"

typedef struct {
    enum Opcode opcode;
    int* operands;
    int numOperands;
} Instruction;

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
        case PUSH:
            return "PUSH";
        case POP:
            return "POP";
    }
    return "UNKNOWN";
}

void printInstruction(Instruction* instr) {
    printf("Instruction: %s", opcodeToString(instr->opcode));
    if (instr->operands != NULL) {
        printf(" -> ");
        for (int i = 0; i < instr->numOperands; i++) {
            printf("%d ", instr->operands[i]);
        }
    }

    printf("\n");
}
