import {
    ASTComparison,
    ASTIdentifier,
    ASTNumber,
    ASTString,
    Expr,
} from './AST';
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
    // const rcx = CompileScope.LeaseRegister(Register.RCX);
    // const regs: Register[] = CompileScope.LeaseRegisters(args.length);

    return CompileScope.LeaseRegistersWithScope((rcx: Register) =>
        CompileScope.LeaseRandomRegistersWithScope((...regs: Register[]) => {
            const argInstructions: string[] = args.flatMap((arg, index) => {
                return arg.compile(scope, regs[index]);
            });

            const dataName = CompileScope.addData(formatString.getValue());
            const instructions: string[] = [
                `mov ${rcx}, ${dataName}`,
                ...argInstructions,
                'call printf',
            ];

            CompileScope.ReleaseRegister(Register.RCX);
            CompileScope.ReleaseRegister(...regs);

            return instructions;
        }, args.length),
    );
}

function compileBasicPrint(scope: CompileScope, arg: Expr): string[] {
    if (
        arg instanceof ASTNumber ||
        arg instanceof ASTIdentifier ||
        arg instanceof ASTComparison ||
        isMathType(arg)
    ) {
        return CompileScope.LeaseRandomRegistersWithScope((resultReg) => {
            const argCompiled = arg.compile(scope, resultReg);

            return CompileScope.LeaseRegistersWithScope(
                (rcx: Register, rdx: Register) => [
                    ...argCompiled,
                    `mov ${rdx}, ${resultReg}`, // Move the argument into RDX
                    `mov ${rcx}, ${CompileScope.addData('"%d", 10')}`,
                    'call printf',
                ],
                Register.RCX,
                Register.RDX,
            );
        });
    }

    throw new Error(`Unsupported argument type for print: ${arg.type}`);
}
