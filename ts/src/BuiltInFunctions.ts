import { ASTIdentifier, ASTNumber, ASTString, Expr } from './AST';
import { CompileScope } from './Compilation/CompileScope';
import { Register } from './types';
import { isComparisonType, isMathType } from './util';

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
    if (args.length > 3) {
        throw new Error(
            'print function with format string can only take up to 3 arguments',
        );
    }

    return CompileScope.LeaseRegistersWithScope(
        (rcx: Register) =>
            CompileScope.LeaseRegistersWithScope(
                (...regs: Register[]) => {
                    const argInstructions: string[] = args.flatMap(
                        (arg, index) => {
                            return arg.compile(scope, regs[index]);
                        },
                    );

                    const dataName = CompileScope.addData(
                        `"${formatString.getValue()}"`,
                    );
                    const instructions: string[] = [
                        'sub rsp, 32',
                        `mov ${rcx}, ${dataName}`,
                        ...argInstructions,
                        'call printf',
                        'add rsp, 32',
                    ];

                    CompileScope.ReleaseRegister(Register.RCX);
                    CompileScope.ReleaseRegister(...regs);

                    return instructions;
                },
                Register.RDX,
                Register.R8,
                Register.R9,
            ),
        Register.RCX,
    );
}

function compileBasicPrint(scope: CompileScope, arg: Expr): string[] {
    if (
        arg instanceof ASTNumber ||
        arg instanceof ASTIdentifier ||
        isComparisonType(arg) ||
        isMathType(arg)
    ) {
        return CompileScope.LeaseRandomRegistersWithScope((resultReg) => {
            const argCompiled = arg.compile(scope, resultReg);

            return CompileScope.LeaseRegistersWithScope(
                (rcx: Register, rdx: Register) => [
                    'sub rsp, 32',
                    ...argCompiled,
                    `mov ${rdx}, ${resultReg}`, // Move the argument into RDX
                    `mov ${rcx}, ${CompileScope.addData('"%d", 10')}`,
                    'call printf',
                    'add rsp, 32',
                ],
                Register.RCX,
                Register.RDX,
            );
        });
    }

    throw new Error(`Unsupported argument type for print: ${arg.type}`);
}
