import { ASTIdentifier, ASTNumber, ASTString, Expr } from './AST';
import { CompileScope } from './Compilation/CompileScope';
import { Register } from './types';
import { isMathType } from './util';

export function printf(scope: CompileScope, args: Expr[]): string[] {
    if (args.length === 0) {
        throw new Error('print function requires at least one argument');
    }

    const [firstArg, ...restArgs] = args;
    if (firstArg instanceof ASTString) {
        return compileFormatPrint(scope, firstArg, restArgs);
    } else {
        return compileBasicPrint(scope, firstArg);
    }
}

function compileFormatPrint(
    scope: CompileScope,
    formatString: ASTString,
    args: Expr[],
): string[] {
    CompileScope.LeaseRegister(Register.RCX);
    const regs: Register[] = CompileScope.LeaseRegisters(args.length);
    const argInstructions: string[] = args.flatMap((arg, index) => {
        return arg.compile(scope, regs[index]);
    });

    const dataName = CompileScope.addData(formatString.getValue());
    const instructions: string[] = [
        `mov rcx, ${dataName}`,
        ...argInstructions,
        'call printf',
    ];

    CompileScope.ReleaseRegister(Register.RCX);
    CompileScope.ReleaseRegister(...regs);

    return instructions;
}

function compileBasicPrint(scope: CompileScope, arg: Expr): string[] {
    CompileScope.LeaseRegister(Register.RCX);
    CompileScope.LeaseRegister(Register.RDX);

    if (
        arg instanceof ASTNumber ||
        arg instanceof ASTIdentifier ||
        isMathType(arg)
    ) {
        const dataName = CompileScope.addData('"%d", 10');
        const instructions: string[] = [
            `mov rcx, ${dataName}`,
            ...arg.compile(scope, Register.RDX),
            'call printf',
        ];

        CompileScope.ReleaseRegister(Register.RCX);
        CompileScope.ReleaseRegister(Register.RDX);
        return instructions;
    }

    throw new Error(`Unsupported argument type for print: ${arg.type}`);
}
