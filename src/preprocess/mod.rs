mod preprocess_error;
mod references;
#[cfg(test)]
mod tests;

pub use preprocess_error::{PreprocessError, PreprocessErrorKind};

use std::{
    collections::{HashMap, HashSet},
    fs, io, mem,
    path::{Path, PathBuf},
    rc::Rc,
};

use crate::{
    ast::{Import, Program, Statement, StatementOrExpression},
    error::{At, CompilerError},
    lex::lex,
    parse::parse,
    source_map::SourceMap,
    types::Span,
};

pub fn resolve_imports(
    program: &mut Program,
    file: u32,
    sources: &mut SourceMap,
) -> Result<(), CompilerError> {
    let path = sources.get(file).path.clone();
    Resolver::default().resolve(program, &path, sources)?;
    Ok(())
}

#[derive(Clone)]
struct Item {
    origin: PathBuf,
    name: String,
    statement: StatementOrExpression,
}

#[derive(Default)]
struct Resolver {
    modules: HashMap<PathBuf, Rc<Vec<Item>>>,
    loading: Vec<PathBuf>,
}

impl Resolver {
    fn resolve(
        &mut self,
        program: &mut Program,
        path: &Path,
        sources: &mut SourceMap,
    ) -> Result<Vec<Item>, CompilerError> {
        let origin = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        let dir = path.parent().unwrap_or(Path::new(""));

        self.loading.push(origin.clone());
        let imported = self.import_all(mem::take(&mut program.imports), dir, sources);
        self.loading.pop();
        let mut items = imported?;

        let imported_count = items.len();
        for child in &program.block.children {
            let Some((name, span)) = definition(child) else {
                continue;
            };

            if items[..imported_count].iter().any(|i| i.name == *name) {
                return Err(PreprocessErrorKind::DuplicateDefinition(name.clone())
                    .at(span)
                    .into());
            }

            items.push(Item {
                origin: origin.clone(),
                name: name.clone(),
                statement: child.clone(),
            });
        }

        program.block.children.splice(
            0..0,
            items[..imported_count].iter().map(|i| i.statement.clone()),
        );
        Ok(items)
    }

    fn import_all(
        &mut self,
        imports: Vec<Import>,
        dir: &Path,
        sources: &mut SourceMap,
    ) -> Result<Vec<Item>, CompilerError> {
        let mut items: Vec<Item> = vec![];
        for import in imports {
            let module = self.load(&import, dir, sources)?;
            for item in select(&module, &import)? {
                if items
                    .iter()
                    .any(|i| i.origin == item.origin && i.name == item.name)
                {
                    continue;
                }

                if items.iter().any(|i| i.name == item.name) {
                    return Err(PreprocessErrorKind::DuplicateDefinition(item.name.clone())
                        .at(import.span)
                        .into());
                }

                items.push(item.clone());
            }
        }
        Ok(items)
    }

    fn load(
        &mut self,
        import: &Import,
        dir: &Path,
        sources: &mut SourceMap,
    ) -> Result<Rc<Vec<Item>>, CompilerError> {
        let path = dir
            .join(import.path.iter().collect::<PathBuf>())
            .with_extension("tan");
        let could_not_read = |e: io::Error| -> CompilerError {
            PreprocessErrorKind::CouldNotRead {
                path: path.clone(),
                reason: e.to_string(),
            }
            .at(import.span)
            .into()
        };

        let canonical = fs::canonicalize(&path).map_err(could_not_read)?;
        if self.loading.contains(&canonical) {
            return Err(PreprocessErrorKind::CircularImport(import.module_name())
                .at(import.span)
                .into());
        }
        if let Some(module) = self.modules.get(&canonical) {
            return Ok(module.clone());
        }

        let text = fs::read_to_string(&path).map_err(could_not_read)?;
        let file = sources.add(path.clone(), text);
        let mut program = parse(lex(&sources.get(file).text, file)?)?;

        let module = Rc::new(self.resolve(&mut program, &path, sources)?);
        self.modules.insert(canonical, module.clone());
        Ok(module)
    }
}

fn definition(child: &StatementOrExpression) -> Option<(&String, Span)> {
    let StatementOrExpression::Statement(statement) = child else {
        return None;
    };

    match statement.as_ref() {
        Statement::FunctionDefinition(d) => Some((&d.name, d.span)),
        Statement::StructDefinition(d) => Some((&d.name, d.span)),
        Statement::EnumDefinition(d) => Some((&d.name, d.span)),
        _ => None,
    }
}

fn select<'a>(module: &'a [Item], import: &Import) -> Result<Vec<&'a Item>, CompilerError> {
    let mut pending = vec![];
    for requested in &import.names {
        if !module.iter().any(|i| i.name == requested.name) {
            return Err(PreprocessErrorKind::UnknownItem {
                module: import.module_name(),
                name: requested.name.clone(),
            }
            .at(requested.span)
            .into());
        }
        pending.push(requested.name.clone());
    }

    let mut wanted = HashSet::new();
    while let Some(name) = pending.pop() {
        if !wanted.insert(name.clone()) {
            continue;
        }

        for item in module.iter().filter(|i| i.name == name) {
            item.statement.references(&mut pending);
        }
    }

    Ok(module.iter().filter(|i| wanted.contains(&i.name)).collect())
}
