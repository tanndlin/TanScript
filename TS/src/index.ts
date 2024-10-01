import util from 'util';
import Environment from './Environment';
import Lexer from './Lexer';
import Optimizer from './Optimizer';
import Parser from './Parser';
import { readScript, writeInstructions } from './util';

const DEBUG = true;
const OPTIMIZE = true;

const script = readScript('script.tan');
const lexer = new Lexer(script);
const tokens = lexer.getTokens();
if (DEBUG) {
    console.log('Tokens:', tokens);
}

const parser = new Parser(tokens);
let ast = parser.parse();
ast = OPTIMIZE ? Optimizer.optimize(ast) : ast;

const instructions = ast.compile();
if (DEBUG) {
    console.log('Instructions:', instructions);
}

writeInstructions(instructions);

if (DEBUG) {
    console.log(util.inspect(ast, { showHidden: false, depth: null }));
}

const env = new Environment(ast, DEBUG);

console.time('Execution time');
env.evaluate();
console.timeEnd('Execution time');
