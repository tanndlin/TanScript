import { RuntimeValue } from './types';

export const allFunctions = {
    print,
    sqrt,
};

export type BuiltInFuncName = keyof typeof allFunctions;

function print(...v: RuntimeValue[]) {
    const [toPrint] = v;

    console.log(toPrint);
    return toPrint;
}

function sqrt(...v: RuntimeValue[]) {
    return Math.sqrt(+v[0]!);
}
