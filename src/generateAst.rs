use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;



fn define_ast(output_dir: &str, base_name: &str, types: Vec<&str>) -> io::Result<()> {
    let path = Path::new(output_dir).join(format!("{}.rs", base_name.to_lowercase()));
    let mut file = File::create(&path)?;

    // Package equivalent comment
    writeln!(file, "// Generated Rust AST for {}", base_name)?;
    writeln!(file)?;

    // Define visitor trait
    define_visitor(&mut file, base_name, &types)?;

    // Define the base trait
    writeln!(file, "pub trait {} {{", base_name)?;
    writeln!(file, "    fn accept<T>(&self, visitor: &dyn {}Visitor<T>) -> T;", base_name)?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    // Define each AST node
    for t in &types {
        let parts: Vec<&str> = t.split(':').map(|s| s.trim()).collect();
        let class_name = parts[0];
        let field_list = parts[1];

        define_type(&mut file, base_name, class_name, field_list)?;
    }

    println!("Generated {} in {:?}", base_name, path);
    Ok(())
}

fn define_type(file: &mut File, base_name: &str, class_name: &str, field_list: &str) -> io::Result<()> {
    writeln!(file, "pub struct {} {{", class_name)?;

    // Process each field
    let fields: Vec<&str> = field_list.split(", ").collect();
    for field in &fields {
        let field_parts: Vec<&str> = field.split_whitespace().collect();
        if field_parts.len() == 2 {
            let field_type = map_type(field_parts[0]);
            let field_name = field_parts[1];
            writeln!(file, "    pub {}: {},", field_name, field_type)?;
        }
    }
    writeln!(file, "}}")?;

    // Implement constructor-like behavior with an `impl` block
    writeln!(file, "impl {} {{", class_name)?;
    writeln!(file, "    pub fn new({}) -> Self {{", 
        fields.iter().map(|f| {
            let parts: Vec<&str> = f.split_whitespace().collect();
            format!("{}: {}", parts[1], map_type(parts[0]))
        }).collect::<Vec<String>>().join(", ")
    )?;

    writeln!(file, "        {} {{", class_name)?;
    for field in &fields {
        let field_parts: Vec<&str> = field.split_whitespace().collect();
        if field_parts.len() == 2 {
            let field_name = field_parts[1];
            writeln!(file, "            {}: {},", field_name, field_name)?;
        }
    }
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    // Implementing the base trait
    writeln!(file, "impl {} for {} {{", base_name, class_name)?;
    writeln!(file, "    fn accept<T>(&self, visitor: &dyn {}Visitor<T>) -> T {{", base_name)?;
    writeln!(file, "        visitor.visit_{}(self)", class_name.to_lowercase())?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    Ok(())
}

fn define_visitor(file: &mut File, base_name: &str, types: &[&str]) -> io::Result<()> {
    writeln!(file, "pub trait {}Visitor<T> {{", base_name)?;
    for t in types {
        let class_name = t.split(':').map(|s| s.trim()).collect::<Vec<&str>>()[0];
        writeln!(file, "    fn visit_{}(&self, {}: &{}) -> T;", class_name.to_lowercase(), class_name.to_lowercase(), class_name)?;
    }
    writeln!(file, "}}")?;
    writeln!(file)?;
    Ok(())
}

fn map_type(java_type: &str) -> &str {
    match java_type {
        "Expr" => "Box<dyn Expr>",
        "Token" => "Token",
        "Object" => "Box<dyn std::any::Any>",
        _ => "UNKNOWN_TYPE",
    }
}
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: generate_ast <output directory>");
        std::process::exit(64);
    }

    let output_dir = &args[1];

    define_ast(
        output_dir,
        "Expr",
        vec![
            "Binary   : Expr left, Token operator, Expr right",
            "Grouping : Expr expression",
            "Literal  : Object value",
            "Unary    : Token operator, Expr right",
        ],
    )
    .expect("Failed to generate AST");

    
}
