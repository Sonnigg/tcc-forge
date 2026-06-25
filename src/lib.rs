use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Write as IoWrite;
use std::process::{Command, Output};

/// Comprehensive C data types including pointer modifiers and arrays.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CType {
    Int,
    UnsignedInt,
    Long,
    UnsignedLong,
    Char,
    Float,
    Double,
    Void,
    Ptr(Box<CType>),
    Const(Box<CType>),
    Array(Box<CType>, usize),
    Struct(String),
    Enum(String),
    Custom(String),
}

impl CType {
    pub fn ptr(self) -> Self {
        CType::Ptr(Box::new(self))
    }

    pub fn to_const(self) -> Self {
        CType::Const(Box::new(self))
    }
}

impl Display for CType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            CType::Int => write!(f, "int"),
            CType::UnsignedInt => write!(f, "unsigned int"),
            CType::Long => write!(f, "long"),
            CType::UnsignedLong => write!(f, "unsigned long"),
            CType::Char => write!(f, "char"),
            CType::Float => write!(f, "float"),
            CType::Double => write!(f, "double"),
            CType::Void => write!(f, "void"),
            CType::Ptr(ty) => write!(f, "{}*", ty),
            CType::Const(ty) => write!(f, "const {}", ty),
            CType::Struct(name) => write!(f, "struct {}", name),
            CType::Enum(name) => write!(f, "enum {}", name),
            CType::Custom(s) => write!(f, "{}", s),
            CType::Array(ty, _) => write!(f, "{}", ty), // Array syntax handling is split during declaration serialization
        }
    }
}

/// Represents a C preprocessor `#define` macro definition.
#[derive(Clone, Debug)]
pub struct CMacro {
    pub name: String,
    pub args: Option<Vec<String>>,
    pub value: String,
}

/// Represents a C `struct` layout declaration.
#[derive(Clone, Debug)]
pub struct CStruct {
    pub name: String,
    pub fields: Vec<(CType, String)>,
    pub is_typedef: bool,
}

/// Represents a C `enum` declaration.
#[derive(Clone, Debug)]
pub struct CEnum {
    pub name: String,
    pub variants: Vec<(String, Option<i64>)>,
    pub is_typedef: bool,
}

/// An indentation-aware code block wrapper tracking control statements.
#[derive(Clone, Debug, Default)]
pub struct Block {
    lines: Vec<String>,
}

impl Block {
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends a structured variable declaration. Handles arrays natively.
    pub fn declare(&mut self, ctype: CType, name: &str, init: &str) -> &mut Self {
        match &ctype {
            CType::Array(inner_ty, size) => {
                self.lines.push(format!("{} {}[{}] = {};", inner_ty, name, size, init));
            }
            _ => {
                self.lines.push(format!("{} {} = {};", ctype, name, init));
            }
        }
        self
    }

    pub fn assign(&mut self, target: &str, expr: &str) -> &mut Self {
        self.lines.push(format!("{} = {};", target, expr));
        self
    }

    pub fn call(&mut self, func: &str, args: &[&str]) -> &mut Self {
        self.lines.push(format!("{}({});", func, args.join(", ")));
        self
    }

    pub fn ret(&mut self, expr: &str) -> &mut Self {
        self.lines.push(format!("return {};", expr));
        self
    }

    pub fn break_stmt(&mut self) -> &mut Self {
        self.lines.push("break;".to_string());
        self
    }

    pub fn raw(&mut self, raw_code: &str) -> &mut Self {
        self.lines.push(raw_code.to_string());
        self
    }

    /// Emits an conditional block logic structure.
    pub fn if_stmt<F>(&mut self, condition: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut Block),
    {
        self.lines.push(format!("if ({}) {{", condition));
        self.embed_sub_block(f);
        self.lines.push("}".to_string());
        self
    }

    /// Emits an execution loop block structure.
    pub fn while_loop<F>(&mut self, condition: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut Block),
    {
        self.lines.push(format!("while ({}) {{", condition));
        self.embed_sub_block(f);
        self.lines.push("}".to_string());
        self
    }

    /// Emits a structured classic standard loop.
    pub fn for_loop<F>(&mut self, init: &str, cond: &str, post: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut Block),
    {
        self.lines.push(format!("for ({}; {}; {}) {{", init, cond, post));
        self.embed_sub_block(f);
        self.lines.push("}".to_string());
        self
    }

    /// Emits a structured multi-branch evaluation matrix.
    pub fn switch_stmt<F>(&mut self, expression: &str, f: F) -> &mut Self
    where
        F: FnOnce(&mut Block),
    {
        self.lines.push(format!("switch ({}) {{", expression));
        self.embed_sub_block(f);
        self.lines.push("}".to_string());
        self
    }

    pub fn case(&mut self, label: &str) -> &mut Self {
        self.lines.push(format!("case {}:", label));
        self
    }

    pub fn default_case(&mut self) -> &mut Self {
        self.lines.push("default:".to_string());
        self
    }

    /// Internal layout helper executing inner closure contexts while indenting lines.
    fn embed_sub_block<F>(&mut self, f: F)
    where
        F: FnOnce(&mut Block),
    {
        let mut sub_block = Block::new();
        f(&mut sub_block);
        for line in sub_block.lines {
            self.lines.push(format!("    {}", line));
        }
    }
}

/// Structural engine mapping top-level C function scopes.
#[derive(Clone, Debug)]
pub struct CFunction {
    name: String,
    return_type: CType,
    args: Vec<(CType, String)>,
    root_block: Block,
}

impl CFunction {
    pub fn new(name: &str, return_type: CType) -> Self {
        Self {
            name: name.to_string(),
            return_type,
            args: Vec::new(),
            root_block: Block::new(),
        }
    }

    pub fn arg(mut self, ctype: CType, name: &str) -> Self {
        self.args.push((ctype, name.to_string()));
        self
    }

    /// Safe proxy exposing internal variable block mutations.
    pub fn body<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Block),
    {
        f(&mut self.root_block);
        self
    }
}

/// The top-level compilation unit deployment module.
#[derive(Default, Clone, Debug)]
pub struct CModule {
    sys_includes: Vec<String>,
    local_includes: Vec<String>,
    macros: Vec<CMacro>,
    enums: Vec<CEnum>,
    structs: Vec<CStruct>,
    functions: Vec<CFunction>,
}

impl CModule {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn include_sys(&mut self, header: &str) -> &mut Self {
        self.sys_includes.push(header.to_string());
        self
    }

    pub fn include_local(&mut self, header: &str) -> &mut Self {
        self.local_includes.push(header.to_string());
        self
    }

    pub fn define_macro(&mut self, name: &str, args: Option<Vec<&str>>, value: &str) -> &mut Self {
        self.macros.push(CMacro {
            name: name.to_string(),
            args: args.map(|v| v.into_iter().map(String::from).collect()),
            value: value.to_string(),
        });
        self
    }

    pub fn add_enum(&mut self, c_enum: CEnum) -> &mut Self {
        self.enums.push(c_enum);
        self
    }

    pub fn add_struct(&mut self, c_struct: CStruct) -> &mut Self {
        self.structs.push(c_struct);
        self
    }

    pub fn add_function(&mut self, func: CFunction) -> &mut Self {
        self.functions.push(func);
        self
    }

    /// Serializes entire AST structures into rigorous, human-readable structural C sources.
    pub fn generate(&self) -> String {
        let mut out = String::new();

        // 1. Structural Imports
        for inc in &self.sys_includes { out.push_str(&format!("#include <{}>\n", inc)); }
        for inc in &self.local_includes { out.push_str(&format!("#include \"{}\"\n", inc)); }
        if !self.sys_includes.is_empty() || !self.local_includes.is_empty() { out.push('\n'); }

        // 2. High-Level Global Macros
        for m in &self.macros {
            if let Some(ref args) = m.args {
                out.push_str(&format!("#define {}({}) {}\n", m.name, args.join(", "), m.value));
            } else {
                out.push_str(&format!("#define {} {}\n", m.name, m.value));
            }
        }
        if !self.macros.is_empty() { out.push('\n'); }

        // 3. Enumeration Type Matrices
        for e in &self.enums {
            let mut lines = Vec::new();
            for (var, val) in &e.variants {
                if let Some(v) = val {
                    lines.push(format!("    {} = {}", var, v));
                } else {
                    lines.push(format!("    {}", var));
                }
            }
            let body = lines.join(",\n");
            if e.is_typedef {
                out.push_str(&format!("typedef enum {{\n{}\n}} {};\n\n", body, e.name));
            } else {
                out.push_str(&format!("enum {} {{\n{}\n}};\n\n", e.name, body));
            }
        }

        // 4. Structured Memory Models
        for s in &self.structs {
            let mut lines = Vec::new();
            for (ty, name) in &s.fields {
                match ty {
                    CType::Array(inner, size) => lines.push(format!("    {} {}[{}];", inner, name, size)),
                    _ => lines.push(format!("    {} {};", ty, name)),
                }
            }
            let body = lines.join("\n");
            if s.is_typedef {
                out.push_str(&format!("typedef struct {{\n{}\n}} {};\n\n", body, s.name));
            } else {
                out.push_str(&format!("struct {} {{\n{}\n}};\n\n", s.name, body));
            }
        }

        // 5. Function Blocks Generation Loop
        for func in &self.functions {
            let args_str = func.args
                .iter()
                .map(|(t, n)| match t {
                    CType::Array(inner, size) => format!("{} {}[{}]", inner, n, size),
                    _ => format!("{} {}", t, n)
                })
                .collect::<Vec<_>>()
                .join(", ");

            out.push_str(&format!("{} {}({}) {{\n", func.return_type, func.name, args_str));
            for line in &func.root_block.lines {
                // Formatting optimization handles dangling label layouts safely
                if line.starts_with("case ") || line.starts_with("default:") {
                    out.push_str(&format!("{}\n", line));
                } else {
                    out.push_str(&format!("    {}\n", line));
                }
            }
            out.push_str("}\n\n");
        }

        out
    }

    /// Directly pipes generated source code to TCC via stdin to create an executable binary.
    pub fn compile_with_tcc(&self, output_binary_path: &str) -> std::io::Result<Output> {
        let source_code = self.generate();

        let mut child = Command::new("tcc")
            .arg("-x").arg("c")
            .arg("-")
            .arg("-o").arg(output_binary_path)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(source_code.as_bytes())?;
        }

        child.wait_with_output()
    }
}