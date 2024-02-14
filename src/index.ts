import * as fs from 'fs';
import util from 'util';
import Parser from './Parser';
import Lexer from './lexer';

const script = fs.readFileSync('script.tan', 'utf8');
const lexer = new Lexer(script);

const tokens = lexer.getTokens();

console.log('Tokens:', tokens);

const parser = new Parser(tokens);
const ast = parser.parse();

console.log(util.inspect(ast, { showHidden: false, depth: null }));
