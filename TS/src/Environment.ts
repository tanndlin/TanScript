import { EOFASTNode, Program, SemiASTNode } from './AST';
import Scope from './Scope';
import { RuntimeValue } from './types';

export default class Environment {
    private globalScope: Scope;

    constructor(
        private ast: Program,
        private debug: boolean = false,
    ) {
        this.globalScope = new Scope(null, null);
    }

    public evaluate(): RuntimeValue {
        let retValue: RuntimeValue = null;

        const root = this.ast.getRoot();
        const statements = root.children;
        statements.forEach((statement, i) => {
            if (
                statement instanceof SemiASTNode ||
                statement instanceof EOFASTNode
            ) {
                return;
            }

            if (this.debug) {
                console.log(`Statement ${i + 1}:`);
            }
            retValue = statement.evaluate(this.globalScope);

            if (this.debug) {
                console.log(retValue);
            }
        });

        return retValue;
    }

    public getGlobalScope(): Scope {
        return this.globalScope;
    }
}
