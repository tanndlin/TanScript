use crate::{
    ast::{
        AtomType, BinaryOp, Block, Expression, ExpressionKind, Program,
        StatementOrExpression::{self},
        UnaryOp,
    },
    codegen::{
        CodegenError,
        address::{Address, Register},
        compile_call::compile_function_call,
        compile_memory::{
            compile_struct_instantiation, load_element, mem_copy, with_place_address,
        },
        compile_operators::{
            compile_binary, compile_cast, compile_logical_binary, compile_postfix,
            compile_postfix_update, compile_unary,
        },
        compile_scope::CompileScope,
        instruction::{Instruction, Operand},
        register_handler::{RegisterHandler, data_name},
        symbol_table::SymbolTable,
    },
    error::ResultAt,
    type_check::Type,
    types::{NumberType, Span},
};

impl Program {
    pub fn compile(&self) -> Result<String, CodegenError> {
        let mut global_scope = CompileScope::default();
        let mut register_handler = RegisterHandler::new();

        let mut symbol_table = SymbolTable::default();
        self.block.discover(&mut symbol_table);
        symbol_table.externs.insert("exit".to_string());
        let signatures = symbol_table
            .functions
            .values()
            .map(|def| &def.signature)
            .chain(
                symbol_table
                    .extern_declarations
                    .values()
                    .map(|def| &def.signature),
            );
        for signature in signatures {
            global_scope.insert_return_type(signature.name.clone(), signature.return_type.clone());
        }
        for def in symbol_table.structs.values() {
            global_scope.insert_struct(def.clone());
        }
        for def in symbol_table.enums.values() {
            global_scope.insert_enum(def.clone());
        }

        let instructions = render(
            &self
                .block
                .compile(&mut global_scope, &mut register_handler)?,
        );

        let functions = symbol_table
            .functions
            .values()
            .map(|def| {
                def.compile(&mut global_scope, &mut register_handler)
                    .map(|instructions| render(&instructions))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("\n");
        let globals = symbol_table
            .functions
            .keys()
            .map(|name| format!("global {name}"))
            .collect::<Vec<_>>()
            .join("\n");

        let extern_functions = symbol_table
            .externs
            .iter()
            .map(|e| format!("extern {e}"))
            .collect::<Vec<_>>()
            .join("\n");

        let data = register_handler
            .data
            .iter()
            .enumerate()
            .map(|(i, d)| format!("\t{}", format_data(&data_name(i), d)))
            .collect::<Vec<String>>()
            .join("\n");

        Ok(format!(
            "BITS 64

global main
{globals}
{extern_functions}

SECTION .rdata
{data}

SECTION .text

{functions}

main:
\tpush rbp
\tmov rbp, rsp
{instructions}
\tmov rsp, rbp
\tpop rbp
\tsub rsp, 40
\txor rcx, rcx
\tcall exit"
        ))
    }
}

pub fn render(instructions: &[Instruction]) -> String {
    instructions
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n")
}

impl Block {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let mut new_scope = CompileScope::new_block(compile_scope);

        let mut instructions = self
            .children
            .iter()
            .map(|child| child.compile(&mut new_scope, register_handler))
            .collect::<Result<Vec<_>, CodegenError>>()?
            .concat();

        let alloc_size = new_scope.stack_depth - compile_scope.stack_depth;
        // A trailing return, break or continue jumps away, so a dealloc after it is unreachable
        if alloc_size > 0 && !matches!(instructions.last(), Some(Instruction::Jmp { .. })) {
            instructions.push(Instruction::add(Register::RSP, alloc_size));
        }

        Ok(instructions)
    }
}

impl StatementOrExpression {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        match &self {
            StatementOrExpression::Statement(statement) => {
                statement.compile(compile_scope, register_handler)
            }
            StatementOrExpression::Expression(expression) => Ok([
                vec![Instruction::comment(expression.to_string())],
                expression.compile_discarded(compile_scope, register_handler)?,
            ]
            .concat()),
        }
    }
}

impl Expression {
    pub fn compile(
        &self,
        compile_scope: &mut CompileScope,
        dst: impl Into<Address>,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        let dst = dst.into();

        match &self.kind {
            ExpressionKind::Atom(a) => match a {
                AtomType::Number(n) => compile_number(*n, dst, register_handler),
                AtomType::Char(c) => Ok(vec![compile_char(*c, dst)]),
                AtomType::Identifier(name) => {
                    compile_variable(compile_scope, register_handler, name, self.span, dst)
                }
                AtomType::String(s) => compile_string(s, dst, register_handler),
                AtomType::Boolean(b) => Ok(vec![compile_boolean(*b, dst)]),
                AtomType::Null => Ok(vec![Instruction::mov(dst, NumberType::I64(0))]),
            },
            ExpressionKind::Binary(BinaryOp::And | BinaryOp::Or, ..) => {
                compile_logical_binary(self, compile_scope, dst, register_handler)
            }
            ExpressionKind::Binary(op, left, right) => {
                compile_binary(*op, left, right, compile_scope, dst, register_handler)
            }
            ExpressionKind::Index(..)
            | ExpressionKind::StructAccess { .. }
            | ExpressionKind::Unary(UnaryOp::Deref, _) => with_place_address(
                self,
                compile_scope,
                register_handler,
                &mut |_, register_handler, address, t| match t {
                    Type::Struct(struct_def) => {
                        mem_copy(&struct_def, dst, address, register_handler)
                    }
                    _ => move_into(dst, register_handler, |reg| load_element(&t, reg, address)),
                },
            ),
            ExpressionKind::Unary(op, child) => {
                compile_unary(*op, child, compile_scope, dst, register_handler)
            }
            ExpressionKind::Postfix(op, lhs) => {
                compile_postfix(*op, lhs, compile_scope, dst, register_handler)
            }
            ExpressionKind::FunctionCall(name, args) => {
                compile_function_call(compile_scope, register_handler, name, args, Some(dst))
            }
            ExpressionKind::Cast(inner, cast_type) => {
                compile_cast(inner, cast_type, compile_scope, dst, register_handler)
            }
            ExpressionKind::SizeOf(sized_type) => compile_number(
                NumberType::Coercible(i64::try_from(sized_type.size()).unwrap()),
                dst,
                register_handler,
            ),
            ExpressionKind::StructInstantiation {
                struct_type_name,
                fields,
            } => compile_struct_instantiation(
                struct_type_name,
                fields,
                compile_scope,
                dst,
                register_handler,
            ),
            ExpressionKind::EnumVariant {
                enum_name,
                variant_name,
            } => compile_enum(
                enum_name,
                variant_name,
                compile_scope,
                dst,
                register_handler,
            ),
        }
    }

    // For when the output is not directly read. mostly used as a side affect like a postfix
    pub fn compile_discarded(
        &self,
        compile_scope: &mut CompileScope,
        register_handler: &mut RegisterHandler,
    ) -> Result<Vec<Instruction>, CodegenError> {
        match &self.kind {
            ExpressionKind::Postfix(op, lhs) => {
                compile_postfix_update(*op, lhs, compile_scope, register_handler)
            }
            ExpressionKind::FunctionCall(name, args) => {
                compile_function_call(compile_scope, register_handler, name, args, None)
            }
            ExpressionKind::Unary(_, child) | ExpressionKind::Cast(child, _) => {
                child.compile_discarded(compile_scope, register_handler)
            }
            ExpressionKind::Binary(BinaryOp::Divide | BinaryOp::Modulo, ..) => register_handler
                .lease_with_scope(|register_handler, dst| {
                    self.compile(compile_scope, dst, register_handler)
                }),
            ExpressionKind::Binary(_, left, right) | ExpressionKind::Index(left, right) => {
                let mut instructions = left.compile_discarded(compile_scope, register_handler)?;
                instructions.extend(right.compile_discarded(compile_scope, register_handler)?);
                Ok(instructions)
            }
            ExpressionKind::Atom(_)
            | ExpressionKind::SizeOf(_)
            | ExpressionKind::StructInstantiation { .. }
            | ExpressionKind::StructAccess { .. }
            | ExpressionKind::EnumVariant { .. } => Ok(vec![]),
        }
    }
}

fn compile_enum(
    enum_name: &str,
    variant_name: &str,
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let enum_def = compile_scope.lookup_enum(enum_name).unwrap();
    let index = enum_def.variant_index(variant_name).unwrap();
    compile_number(
        NumberType::U8(u8::try_from(index).unwrap()),
        dst,
        register_handler,
    )
}

fn compile_string(
    s: &str,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let handle = register_handler.add_data(s);
    move_into(dst, register_handler, |reg| {
        Instruction::mov(reg, Operand::Label(handle))
    })
}

pub fn move_into(
    dst: Address,
    register_handler: &mut RegisterHandler,
    instruction: impl FnOnce(Register) -> Instruction,
) -> Result<Vec<Instruction>, CodegenError> {
    match dst {
        Address::Register(reg) => Ok(vec![instruction(reg)]),
        _ => register_handler
            .lease_with_scope(|_, reg| Ok(vec![instruction(reg), Instruction::mov(dst, reg)])),
    }
}

fn compile_number(
    n: NumberType,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    if fits_imm32(n) {
        Ok(vec![Instruction::mov(dst, n)])
    } else {
        move_into(dst, register_handler, |reg| Instruction::mov(reg, n))
    }
}

fn compile_char(c: char, dst: Address) -> Instruction {
    Instruction::mov(dst, c)
}

fn compile_variable(
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    name: &str,
    span: Span,
    dst: Address,
) -> Result<Vec<Instruction>, CodegenError> {
    let address = compile_scope.get_variable(name).at(span)?;
    if dst == address {
        return Ok(vec![]);
    }

    if let Type::Struct(struct_def) = compile_scope.get_type(name).at(span)? {
        return mem_copy(struct_def, dst, address, register_handler);
    }

    match address {
        Address::Register(_) => Ok(vec![Instruction::mov(dst, address)]),
        _ => move_into(dst, register_handler, |reg| Instruction::mov(reg, address)),
    }
}

fn compile_boolean(b: bool, dst: Address) -> Instruction {
    let n = NumberType::I64(i64::from(b));
    Instruction::mov(dst, n)
}

pub fn as_operand(expr: &Expression, scope: &CompileScope) -> Option<Operand> {
    match &expr.kind {
        ExpressionKind::Atom(AtomType::Number(n)) if fits_imm32(*n) => Some(Operand::Immediate(*n)),
        ExpressionKind::Atom(AtomType::Char(c)) => Some(Operand::from(*c)),
        ExpressionKind::Atom(AtomType::Identifier(name)) => {
            scope.get_variable(name).ok().map(Operand::from)
        }
        ExpressionKind::EnumVariant {
            enum_name,
            variant_name,
        } => {
            let index = scope.lookup_enum(enum_name)?.variant_index(variant_name)?;
            Some(Operand::Immediate(NumberType::U8(
                u8::try_from(index).ok()?,
            )))
        }
        _ => None,
    }
}

fn fits_imm32(n: NumberType) -> bool {
    match n {
        NumberType::U8(_) | NumberType::I32(_) => true,
        NumberType::U32(n) => i32::try_from(n).is_ok(),
        NumberType::U64(n) => i32::try_from(n).is_ok(),
        NumberType::I64(n) | NumberType::Coercible(n) => i32::try_from(n).is_ok(),
    }
}

fn format_data(name: &str, s: &str) -> String {
    let mut parts = vec![];
    let mut run = String::new();
    for c in s.chars() {
        if c == ' ' || (c.is_ascii_graphic() && c != '"') {
            run.push(c);
            continue;
        }

        if !run.is_empty() {
            parts.push(format!("\"{run}\""));
            run.clear();
        }

        let mut buf = [0; 4];
        parts.extend(c.encode_utf8(&mut buf).bytes().map(|b| b.to_string()));
    }

    if !run.is_empty() {
        parts.push(format!("\"{run}\""));
    }
    parts.push("0".to_string());

    format!("{name} db {}", parts.join(", "))
}

#[cfg(test)]
pub mod test {
    use crate::{lex::lex, parse::parse, type_check::check_types};

    pub fn compile(input: &str) -> String {
        let mut ast = parse(lex(input, 0).unwrap()).unwrap();
        check_types(&mut ast).unwrap();
        ast.compile().unwrap()
    }

    #[test]
    fn compile_boolean_literals() {
        let asm = compile("let a = true; let b = false;");
        assert!(asm.contains("mov QWORD [rbp - 8], 1"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 0"), "{asm}");
    }

    #[test]
    fn compile_discarded_postfix_skips_old_value() {
        let asm = compile("let i = 0; i++;");
        assert!(!asm.contains(", [rbp - 8]"), "{asm}");
    }

    #[test]
    fn compile_discarded_call_keeps_result_in_rax() {
        let asm = compile("def five(): i32 { 5 } five();");
        assert!(!asm.contains(", rax"), "{asm}");
    }

    #[test]
    fn compile_declared_extern_call() {
        let asm = compile("extern def puts(s: *char): i32; let n = puts(\"hi\");");
        assert!(asm.contains("extern puts"), "{asm}");
        assert!(asm.contains("call puts"), "{asm}");
    }

    #[test]
    fn compile_boolean_is_not_a_variable_lookup() {
        compile("if (true) { printf(\"yes\"); }");
        compile("let a = !false;");
    }

    #[test]
    fn compile_char_literals() {
        let asm = compile("let a: char = 'a'; let b: char = 'Z'; let c: char = '0';");
        assert!(asm.contains("mov QWORD [rbp - 8], 97"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 90"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 24], 48"), "{asm}");
        assert!(!asm.contains("db "), "{asm}");
    }

    #[test]
    fn compile_char_escapes() {
        let asm = compile(r"let a: char = '\n'; let b: char = '\0'; let c: char = '\'';");
        assert!(asm.contains("mov QWORD [rbp - 8], 10"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 16], 0"), "{asm}");
        assert!(asm.contains("mov QWORD [rbp - 24], 39"), "{asm}");
    }

    #[test]
    fn compile_char_arithmetic() {
        compile("let c: char = '7'; let d = c - '0';");
        compile("let c: char = 'a'; c += 1;");
        compile("let c: char = 'a'; c++;");
    }

    #[test]
    fn compile_char_comparison() {
        compile("let c: char = 'a'; if (c == 'a') { printf(\"yes\"); }");
        compile("let c: char = 'a'; let lower = c >= 'a' && c <= 'z';");
    }

    #[test]
    fn compile_char_as_function_arg() {
        let asm = compile("putchar('h');");
        assert!(asm.contains("104"), "{asm}");
        compile("printf(\"%c%c\", 'h', 'i');");
    }

    #[test]
    fn compile_char_function_param_and_return() {
        compile("def next(c: char): char { c + 1 } next('a');");
    }

    #[test]
    fn compile_large_literal_to_stack_goes_through_register() {
        let asm = compile("let big: u32 = 4000000000;");
        assert!(!asm.contains("], 4000000000"), "{asm}");
        assert!(asm.contains(", 4000000000"), "{asm}");
    }

    #[test]
    fn compile_small_literal_to_stack_is_immediate() {
        let asm = compile("let small: u32 = 5;");
        assert!(asm.contains("], 5"), "{asm}");
    }
}
