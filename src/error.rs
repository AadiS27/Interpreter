pub fn error(line: usize, message: &str, context: &str) {
    eprintln!(
        "[line {}] Error: {}\n{}\n{}^",
        line,
        message,
        context,
        " ".repeat(context.len())
    );
}
