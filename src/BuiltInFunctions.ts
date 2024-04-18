import { RuntimeValue } from './types';

export const allFunctions = {
    print,
};

export type BuiltInFuncName = keyof typeof allFunctions;

function print(...v: RuntimeValue[]) {
    const [toPrint] = v;

    console.log(toPrint);
    return toPrint;
}
