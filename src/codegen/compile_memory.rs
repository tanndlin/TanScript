use crate::{
    ast::{AtomType, Expression, ExpressionKind, StructDefinition, UnaryOp},
    codegen::{
        CodegenError,
        address::{Address, Register, Scale},
        compile_scope::CompileScope,
        instruction::{Instruction, Size},
        register_handler::RegisterHandler,
    },
    error::ResultAt,
    type_check::Type,
    types::NumberType,
};

pub fn compile_struct_instantiation(
    struct_type_name: &str,
    fields: &[(String, Expression)],
    compile_scope: &mut CompileScope,
    dst: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let struct_def = compile_scope
        .lookup_struct(struct_type_name)
        .unwrap()
        .clone();

    if let Address::Register(dst) = dst {
        assert!(
            matches!(struct_def.size(), 1 | 2 | 4 | 8),
            "Only structs of size (1 | 2 | 4 | 8) can be put into a register",
        );

        return compile_struct_instantiation_to_reg(
            &struct_def,
            fields,
            compile_scope,
            dst,
            register_handler,
        );
    }

    let mut instructions = vec![];
    for (field_name, expr) in fields {
        let field_type = struct_def.get_field_type(field_name).unwrap().clone();
        let offset = u32::try_from(struct_def.get_byte_offset(field_name)).unwrap();
        let address = offset_address(dst, offset);

        // Allows for struct as-value in struct literal assignment
        if let Type::Struct(_) = field_type {
            instructions.extend(expr.compile(compile_scope, address, register_handler)?);
            continue;
        }

        instructions.extend(register_handler.lease_with_scope(|register_handler, reg| {
            let mut field = expr.compile(compile_scope, reg, register_handler)?;
            field.push(store_element(&field_type, address, reg));
            Ok(field)
        })?);
    }

    Ok(instructions)
}

fn compile_struct_instantiation_to_reg(
    struct_def: &StructDefinition,
    fields: &[(String, Expression)],
    compile_scope: &mut CompileScope,
    dst: Register,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    let mut instructions = vec![];
    for (i, (field_name, expr)) in fields.iter().enumerate() {
        let field_type = struct_def.get_field_type(field_name).unwrap().clone();
        let offset = u16::try_from(struct_def.get_byte_offset(field_name)).unwrap();

        if i == 0 {
            instructions.extend(expr.compile(compile_scope, dst, register_handler)?);
            instructions.extend(zero_extend(&field_type, expr, dst));
            if offset != 0 {
                instructions.push(Instruction::shl(dst, offset * 8));
            }
            continue;
        }

        instructions.extend(
            register_handler.lease_with_scope(|register_handler, field_reg| {
                let mut field = expr.compile(compile_scope, field_reg, register_handler)?;
                field.extend(zero_extend(&field_type, expr, field_reg));
                if offset != 0 {
                    field.push(Instruction::shl(field_reg, offset * 8));
                }
                field.push(Instruction::or(dst, field_reg));
                Ok(field)
            })?,
        );
    }

    Ok(instructions)
}

type WithAddress<'f> = dyn FnMut(
        &mut CompileScope,
        &mut RegisterHandler,
        Address,
        Type,
    ) -> Result<Vec<Instruction>, CodegenError>
    + 'f;

pub fn with_place_address(
    place: &Expression,
    compile_scope: &mut CompileScope,
    register_handler: &mut RegisterHandler,
    f: &mut WithAddress,
) -> Result<Vec<Instruction>, CodegenError> {
    match &place.kind {
        ExpressionKind::Atom(AtomType::Identifier(name)) => {
            let (address, t) = compile_scope.lookup(name).at(place.span)?.clone();
            f(compile_scope, register_handler, address, t)
        }
        ExpressionKind::Index(lhs, index) => {
            let element = (*place.etype).clone();
            register_handler.lease_with_scope(|register_handler, offset| {
                let mut instructions = index.compile(compile_scope, offset, register_handler)?;
                let size = u32::try_from(element.size()).unwrap();
                let scale = Scale::from_size(size).unwrap_or_else(|| {
                    instructions.push(Instruction::imul(offset, size));
                    Scale::One
                });
                instructions.extend(register_handler.lease_with_scope(
                    |register_handler, base| {
                        let mut instructions =
                            lhs.compile(compile_scope, base, register_handler)?;
                        let address = Address::Indexed {
                            base,
                            index: offset,
                            scale,
                            offset: 0,
                        };
                        instructions.extend(f(compile_scope, register_handler, address, element)?);
                        Ok(instructions)
                    },
                )?);
                Ok(instructions)
            })
        }
        ExpressionKind::Unary(UnaryOp::Deref, pointer) => {
            let element = (*place.etype).clone();
            register_handler.lease_with_scope(|register_handler, base| {
                let mut instructions = pointer.compile(compile_scope, base, register_handler)?;
                let address = Address::Deref { base, offset: 0 };
                instructions.extend(f(compile_scope, register_handler, address, element)?);
                Ok(instructions)
            })
        }
        ExpressionKind::StructAccess { base, field_name } => with_place_address(
            base,
            compile_scope,
            register_handler,
            &mut |compile_scope, register_handler, address, struct_type| {
                let Type::Struct(struct_def) = struct_type else {
                    panic!("({base}) is not a struct")
                };
                let field_type = struct_def.get_field_type(field_name).unwrap().clone();
                let field_offset = u32::try_from(struct_def.get_byte_offset(field_name)).unwrap();
                f(
                    compile_scope,
                    register_handler,
                    offset_address(address, field_offset),
                    field_type,
                )
            },
        ),
        _ => unreachable!("({place}) is not a place"),
    }
}

pub fn narrow_size(element: &Type) -> Option<(Size, bool)> {
    match element {
        Type::Int(int) if int.size() == 4 => Some((Size::Dword, int.is_signed())),
        Type::Int(int) if int.size() == 1 => Some((Size::Byte, int.is_signed())),
        Type::Boolean | Type::Char | Type::Enum(_) => Some((Size::Byte, false)),
        _ => None,
    }
}

pub fn extend(element: &Type, dst: Register, src: impl Into<Address>) -> Option<Instruction> {
    narrow_size(element).map(|(size, signed)| {
        if signed {
            Instruction::movsx(dst, src, size)
        } else {
            Instruction::movzx(dst, src, size)
        }
    })
}

fn zero_extend(element: &Type, value: &Expression, reg: Register) -> Option<Instruction> {
    if is_non_negative_literal(value) {
        return None;
    }

    narrow_size(element).map(|(size, _)| Instruction::movzx(reg, reg, size))
}

fn is_non_negative_literal(expr: &Expression) -> bool {
    match &expr.kind {
        ExpressionKind::Atom(AtomType::Number(n)) => !matches!(
            n,
            NumberType::I32(..=-1) | NumberType::I64(..=-1) | NumberType::Coercible(..=-1)
        ),
        ExpressionKind::Atom(AtomType::Char(_) | AtomType::Boolean(_)) => true,
        _ => false,
    }
}

/// Loading from memory
pub fn load_element(element: &Type, dst: Register, memory: Address) -> Instruction {
    extend(element, dst, memory).unwrap_or_else(|| Instruction::mov(dst, memory))
}

/// Storing to memory
pub fn store_element(element: &Type, memory: Address, src: Register) -> Instruction {
    narrow_size(element).map_or_else(
        || Instruction::mov(memory, src),
        |(size, _)| Instruction::store(memory, src, size),
    )
}

// This will have to take other types eventually
pub fn mem_copy(
    element: &StructDefinition,
    dst: Address,
    src: Address,
    register_handler: &mut RegisterHandler,
) -> Result<Vec<Instruction>, CodegenError> {
    if let Address::Register(dst) = dst {
        let size = match element.size() {
            1 => Size::Byte,
            2 => Size::Word,
            4 => Size::Dword,
            8 => Size::Qword,
            size => panic!(
                "Only structs of size (1 | 2 | 4 | 8) can be put into a register, got {size}"
            ),
        };
        return Ok(vec![Instruction::movzx(dst, src, size)]);
    }

    register_handler.lease_with_scope(|_, reg| {
        let mut instructions = vec![];
        for field in &element.fields {
            let offset = u32::try_from(element.get_byte_offset(&field.name)).unwrap();
            instructions.push(load_element(&field.ftype, reg, offset_address(src, offset)));
            instructions.push(store_element(
                &field.ftype,
                offset_address(dst, offset),
                reg,
            ));
        }

        Ok(instructions)
    })
}

fn offset_address(address: Address, offset: u32) -> Address {
    match address {
        Address::Stack(start) => Address::Stack(start - i32::try_from(offset).unwrap()),
        Address::StackArg(start) => Address::StackArg(start + i32::try_from(offset).unwrap()),
        Address::Indexed {
            base,
            index,
            scale,
            offset: start,
        } => Address::Indexed {
            base,
            index,
            scale,
            offset: start + offset,
        },
        Address::Deref {
            base,
            offset: start,
        } => Address::Deref {
            base,
            offset: start + offset,
        },
        Address::Register(_) => panic!("Cannot offset register ({address}) offset was: ({offset})"),
    }
}

#[cfg(test)]
mod test {
    use crate::codegen::compile::test::compile;

    #[test]
    fn compile_store_char_element_is_byte() {
        let asm = compile("let s: *char = malloc(4); s[0] = 'h';");
        assert!(asm.contains("BYTE ["), "{asm}");
    }

    #[test]
    fn compile_load_char_element_is_byte() {
        let asm = compile("let s: *char = malloc(4); let c: char = s[0];");
        assert!(asm.contains("movzx"), "{asm}");
        assert!(asm.contains("BYTE ["), "{asm}");
    }

    /// Lines that access memory through a register other than rbp/rsp
    fn derefs(asm: &str) -> Vec<&str> {
        asm.lines()
            .map(str::trim)
            .filter(|l| {
                l.split_once('[')
                    .is_some_and(|(_, rest)| !rest.starts_with("rbp") && !rest.starts_with("rsp"))
            })
            .collect()
    }

    fn leas(asm: &str) -> Vec<&str> {
        asm.lines()
            .map(str::trim)
            .filter(|l| l.starts_with("lea "))
            .collect()
    }

    #[test]
    fn compile_address_of_local_uses_lea() {
        let asm = compile("let x = 5; let p = &x;");
        assert!(leas(&asm).iter().any(|l| l.ends_with("[rbp - 8]")), "{asm}");
    }

    #[test]
    fn compile_address_of_struct_uses_lea() {
        let asm = compile("struct P { a: i64, b: i64, } let s = P { a: 1, b: 2 }; let p = &s;");
        assert!(!leas(&asm).is_empty(), "{asm}");
    }

    #[test]
    fn compile_address_of_field_uses_field_offset() {
        let asm = compile(
            "struct P { a: i64, b: i64, } let s = P { a: 1, b: 2 }; let pa = &s.a; let pb = &s.b;",
        );
        let leas = leas(&asm);
        assert_eq!(leas.len(), 2, "{asm}");
        assert_ne!(
            leas[0].split_once(',').unwrap().1,
            leas[1].split_once(',').unwrap().1,
            "{asm}"
        );
    }

    #[test]
    fn compile_address_of_arg_uses_lea() {
        let asm = compile("def f(x: i64): *i64 { return &x; }");
        assert!(leas(&asm).iter().any(|l| l.contains("[rsp + ")), "{asm}");
    }

    #[test]
    fn compile_deref_load() {
        let asm = compile("let x = 5; let p = &x; let y = *p;");
        assert!(derefs(&asm).iter().any(|l| l.starts_with("mov ")), "{asm}");
    }

    #[test]
    fn compile_deref_load_u8_is_byte() {
        let asm = compile("let x: u8 = 5; let p = &x; let y: u8 = *p;");
        assert!(
            derefs(&asm)
                .iter()
                .any(|l| l.starts_with("movzx") && l.contains("BYTE [")),
            "{asm}"
        );
    }

    #[test]
    fn compile_deref_load_u32_is_dword() {
        let asm = compile("let x: u32 = 5; let p = &x; let y: u32 = *p;");
        assert!(derefs(&asm).iter().any(|l| l.contains("DWORD [")), "{asm}");
    }

    #[test]
    fn compile_deref_store() {
        let asm = compile("let x = 5; let p = &x; *p = 7;");
        assert!(derefs(&asm).iter().any(|l| l.starts_with("mov [")), "{asm}");
    }

    #[test]
    fn compile_deref_store_u8_is_byte() {
        let asm = compile("let x: u8 = 5; let p = &x; *p = 7;");
        let derefs = derefs(&asm);
        assert!(derefs.iter().any(|l| l.contains("BYTE [")), "{asm}");
        assert!(!derefs.iter().any(|l| l.contains("QWORD [")), "{asm}");
    }

    #[test]
    fn compile_double_deref_loads_twice() {
        let asm = compile("let x = 5; let p = &x; let pp = &p; let y = **pp;");
        assert!(derefs(&asm).len() >= 2, "{asm}");
    }

    #[test]
    fn compile_field_through_pointer_uses_offset() {
        let asm = compile(
            "struct P { a: i64, b: i64, } let s = P { a: 1, b: 2 }; let p = &s; let v = (*p).b;",
        );
        assert!(derefs(&asm).iter().any(|l| l.contains(" + 8]")), "{asm}");
    }

    #[test]
    fn compile_nested_field_through_pointer_adds_offsets() {
        let asm = compile(
            "struct P { a: i64, b: i64, } struct Q { x: i64, p: P, } let r: *Q = malloc(24); let v = (*r).p.b;",
        );
        assert!(derefs(&asm).iter().any(|l| l.contains(" + 16]")), "{asm}");
    }

    #[test]
    fn compile_field_store_through_pointer_uses_field_size() {
        let asm = compile(
            "struct P { a: i64, b: u8, } let s = P { a: 1, b: 2 }; let p = &s; (*p).b = 3;",
        );
        assert!(
            derefs(&asm)
                .iter()
                .any(|l| l.contains("BYTE [") && l.contains(" + 8]")),
            "{asm}"
        );
    }

    const NODE: &str = "struct N<T> { value: T, next: *N<T>, } ";

    #[test]
    fn compile_unused_generic_struct() {
        compile(&format!("{NODE}let x = 1;"));
    }

    #[test]
    fn compile_generic_field_offset_depends_on_type_argument() {
        let asm = compile(&format!(
            "{NODE}let n: *N<u8> = calloc(1, 9); let next = (*n).next;"
        ));
        assert!(derefs(&asm).iter().any(|l| l.contains(" + 1]")), "{asm}");

        let asm = compile(&format!(
            "{NODE}let n: *N<i64> = calloc(1, 16); let next = (*n).next;"
        ));
        assert!(derefs(&asm).iter().any(|l| l.contains(" + 8]")), "{asm}");
    }

    #[test]
    fn compile_generic_field_store_uses_type_argument_size() {
        let asm = compile(&format!(
            "{NODE}let n: *N<u8> = calloc(1, 9); (*n).value = 3;"
        ));
        assert!(derefs(&asm).iter().any(|l| l.contains("BYTE [")), "{asm}");
    }

    #[test]
    fn compile_two_instantiations_of_one_generic() {
        let asm = compile(&format!(
            "{NODE}let a: *N<u8> = calloc(1, 9); let b: *N<i64> = calloc(1, 16); let x = (*a).next; let y = (*b).next;"
        ));
        let derefs = derefs(&asm);
        assert!(derefs.iter().any(|l| l.contains(" + 1]")), "{asm}");
        assert!(derefs.iter().any(|l| l.contains(" + 8]")), "{asm}");
    }

    #[test]
    fn compile_double_deref_through_generic_self_reference() {
        let asm = compile(&format!(
            "{NODE}let n: *N<u8> = calloc(1, 9); let m = (*(*n).next).next;"
        ));
        assert!(derefs(&asm).iter().any(|l| l.contains(" + 1]")), "{asm}");
    }

    #[test]
    fn compile_index_through_generic_self_reference() {
        let asm = compile(&format!(
            "{NODE}let n: *N<u8> = calloc(1, 9); let v = (*n).next[0].value;"
        ));
        assert!(derefs(&asm).iter().any(|l| l.contains("BYTE [")), "{asm}");
    }

    #[test]
    fn compile_generic_struct_store_in_function() {
        compile(&format!(
            "struct T {{ k: u8, d: u64, }} {NODE}def push(n: *N<T>, v: T) {{ (*n).value = v; }} let n: *N<T> = calloc(1, 17); push(n, T {{ k: 1, d: 2 }});"
        ));
    }

    #[test]
    fn compile_struct_store_through_pointer_field() {
        compile(
            "struct T { k: u8, d: u64, } struct N { v: T, n: *N, } let v = T { k: 1, d: 2 }; let p: *N = calloc(1, 17); (*p).v = v;",
        );
    }

    #[test]
    fn compile_struct_store_through_pointer_field_in_function() {
        compile(
            "struct T { k: u8, d: u64, } struct N { v: T, n: *N, } def f(v: T) { let p: *N = calloc(1, 17); (*p).v = v; } f(T { k: 1, d: 2 });",
        );
    }

    #[test]
    fn compile_struct_store_through_index() {
        compile(
            "struct T { k: u8, d: u64, } let ts: *T = calloc(2, 9); let t = T { k: 1, d: 2 }; ts[1] = t;",
        );
    }

    #[test]
    fn compile_index_into_odd_sized_struct_uses_valid_scale() {
        let asm = compile(
            "struct T { k: u8, d: u64, } let ts: *T = calloc(2, 9); let i = 1; let k = ts[i].k;",
        );
        for line in asm.lines().filter(|l| !l.trim_start().starts_with(';')) {
            if let Some((_, scale)) = line.split_once(" * ") {
                let scale = scale.trim_end_matches(']');
                assert!(
                    matches!(scale, "1" | "2" | "4" | "8"),
                    "invalid scale in `{line}`\n{asm}"
                );
            }
        }
    }

    #[test]
    fn compile_passing_address_as_arg() {
        let asm = compile("def f(p: *i64): i64 { return *p; } let x = 5; f(&x);");
        assert!(!leas(&asm).is_empty(), "{asm}");
        assert!(!derefs(&asm).is_empty(), "{asm}");
    }
}
