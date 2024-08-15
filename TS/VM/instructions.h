#include <stdio.h>

#include "opcodes.h"

typedef struct {
    enum Opcode opcode;
    int* operands;
    int numOperands;
} Instruction;

void printInstruction(Instruction instr) {
    printf("Instruction: %s", opcodeToString(instr.opcode));
    if (instr.operands != NULL) {
        printf(" . ");
        for (int i = 0; i < instr.numOperands; i++) {
            printf("%d ", instr.operands[i]);
        }
    }

    printf("\n");
}
