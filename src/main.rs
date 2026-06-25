use std::fmt::{Display, Formatter, Result as FmtResult};
use std::io::Write as IoWrite;
use std::process::{Command, Output};

/// Represents standard and custom C data types.
#[derive(Clone, Debug)]
pub enum CType {
    Int,
    UnsignedInt,
    Char,
    Float,
    Double,
    Void,
    Ptr(Box<CType>),
    Const(Box<CType>),
    Custom(String),
}

impl Display for CType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            CType::Int => write!(f, "int"),
            CType::UnsignedInt => write!(f, "unsigned int"),
            CType::Char => write!(f, "char"),
            CType::Float => write!(f, "float"),
            CType::Double => write!(f, "double"),
            CType::Void => write!(f, "void"),
            CType::Ptr(ty) => write!(f, "{}*", ty),
            CType::Const(ty) => write!(f, "const {}", ty),
            CType::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Builder for constructing individual C functions.
#[derive(Clone, Debug)]
pub struct CFunction {
    name: String,
    return_type: CType,
    args: Vec<(CType, String)>,
    body: Vec<String>,
}

impl CFunction {
    pub fn new(name: &str, return_type: CType) -> Self {
        Self {
            name: name.to_string(),
            return_type,
            args: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Adds a parameter to the function signature.
    pub fn arg(mut self, ctype: CType, name: &str) -> Self {
        self.args.push((ctype, name.to_string()));
        self
    }

    /// Declares a variable inside the function scope.
    pub fn declare(&mut self, ctype: CType, name: &str, init: &str) -> &mut Self {
        self.body.push(format!("{} {} = {};", ctype, name, init));
        self
    }

    /// Performs an assignment to an existing variable.
    pub fn assign(&mut self, name: &str, expr: &str) -> &mut Self {
        self.body.push(format!("{} = {};", name, expr));
        self
    }

    /// Injects a function call statement.
    pub fn call(&mut self, func: &str, args: &[&str]) -> &mut Self {
        self.body.push(format!("{}({});", func, args.join(", ")));
        self
    }

    /// Appends a return statement.
    pub fn ret(&mut self, expr: &str) -> &mut Self {
        self.body.push(format!("return {};", expr));
        self
    }

    /// Injects a raw string line directly into the function body (ideal for loops/ifs).
    pub fn raw(&mut self, raw_code: &str) -> &mut Self {
        self.body.push(raw_code.to_string());
        self
    }
}

/// The top-level compilation unit container.
#[derive(Default, Clone, Debug)]
pub struct CModule {
    sys_includes: Vec<String>,
    local_includes: Vec<String>,
    typedefs_and_structs: Vec<String>,
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

    pub fn add_raw_definition(&mut self, structure: &str) -> &mut Self {
        self.typedefs_and_structs.push(structure.to_string());
        self
    }

    pub fn add_function(&mut self, func: CFunction) -> &mut Self {
        self.functions.push(func);
        self
    }

    /// Serializes the entire AST into a valid, formatted C source code string.
    pub fn generate(&self) -> String {
        let mut out = String::new();

        // 1. Headers
        for inc in &self.sys_includes {
            out.push_str(&format!("#include <{}>\n", inc));
        }
        for inc in &self.local_includes {
            out.push_str(&format!("#include \"{}\"\n", inc));
        }
        if !self.sys_includes.is_empty() || !self.local_includes.is_empty() {
            out.push('\n');
        }

        // 2. Types/Structs
        for item in &self.typedefs_and_structs {
            out.push_str(&format!("{}\n\n", item));
        }

        // 3. Functions
        for func in &self.functions {
            let args_str = func.args
                .iter()
                .map(|(t, n)| format!("{} {}", t, n))
                .collect::<Vec<_>>()
                .join(", ");
            
            out.push_str(&format!("{} {}({}) {{\n", func.return_type, func.name, args_str));
            for line in &func.body {
                out.push_str(&format!("    {}\n", line));
            }
            out.push_str("}\n\n");
        }

        out
    }

    /// Pipes the generated code directly to TCC via stdin to compile an executable binary.
    pub fn compile_with_tcc(&self, output_binary_path: &str) -> std::io::Result<Output> {
        let source_code = self.generate();

        // "-x c -" tells TCC to treat standard input data stream as a C source file
        let mut child = Command::new("tcc")
            .arg("-x")
            .arg("c")
            .arg("-")
            .arg("-o")
            .arg(output_binary_path)
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

fn main() {}