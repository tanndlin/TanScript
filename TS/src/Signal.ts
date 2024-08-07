import { SignalComputeAssignmentAST } from './AST/SignalAST';
import Scope from './Scope';
import { IChildrenEnumerable, RuntimeValue } from './types';
import { findSignals } from './util';

export class Signal {
    public type = 'signal';

    // List of all computed signals that rely on this signal
    public dependencies: ComputedSignal[] = [];

    constructor(
        public name: string,
        public value: RuntimeValue,
    ) {}

    getValue(): RuntimeValue {
        return this.value;
    }

    markChildrenDirty() {
        this.dependencies.forEach((dep) => {
            dep.isDirty = true;
            dep.markChildrenDirty();
        });
    }
}

export class ComputedSignal extends Signal {
    public type = 'computed';

    public dependsOn: Signal[] = [];

    public isDirty: boolean = false;

    private callback: () => RuntimeValue;

    constructor(
        scope: Scope,
        public name: string,
        assignAST: SignalComputeAssignmentAST,
    ) {
        super(name, assignAST.valueAST.evaluate(scope));

        const { valueAST } = assignAST;

        // Find all signals in the expression
        const signals = findSignals(valueAST as IChildrenEnumerable);
        signals.forEach((signalName) => {
            const signal = scope.getSignal(signalName);
            if (signal instanceof ComputedSignal) {
                this.dependsOn.push(...signal.dependsOn);
            }

            this.dependsOn.push(signal);
            signal.dependencies.push(this);
        });

        this.callback = () => valueAST.evaluate(scope);
    }

    getValue() {
        if (!this.isDirty) {
            return this.value;
        }
        // Clean all deps
        this.dependsOn.forEach((dep) => {
            dep.getValue();
        });

        this.value = this.callback();
        this.isDirty = false;

        return this.value;
    }

    markChildrenDirty(): void {
        // If this signal is already dirty, then all of its dependencies are dirty
        if (this.isDirty) {
            return;
        }

        super.markChildrenDirty();
    }
}
