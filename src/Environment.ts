import { AST } from './AST/AST';
import Scope from './Scope';
import { RuntimeValue, Token } from './types';

export default class Environment {
    private globalScope: Scope;

    constructor(private ast: AST) {
        this.globalScope = new Scope(null);
    }

    public evaluate(): RuntimeValue {
        let retValue: RuntimeValue = null;

        const root = this.ast.getRoot();
        const statements = root.getChildren();
        statements.forEach((statement, i) => {
            if (statement.getType() === Token.EOF) return;

            console.log(`Statement ${i + 1}:`);
            retValue = statement.evaluate(this.globalScope);
            console.log(retValue);
        });

        return retValue;
    }

    public getGlobalScope(): Scope {
        return this.globalScope;
    }
}
