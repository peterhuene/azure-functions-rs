use colored::Colorize;

pub fn print_running(message: &str) {
    print!("{} {}", "️🚀".cyan(), message);
}

pub fn print_success() {
    println!(" {}", "✓".green());
}

pub fn print_failure() {
    println!(" {}", "✗".red());
}
