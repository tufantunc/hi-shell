use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "hi-shell")]
#[command(version = "0.1.0")]
#[command(
    about = "An intelligent terminal assistant that translates your natural language descriptions into executable bash commands."
)]
struct Args {
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    input: Vec<String>,
}
fn main() {
    let args = Args::parse();

    if args.input.is_empty() {
        println!("Hi! This is Interactive mode. What do you want to do? (exit to quit)");
        // we will add REPL loop here
    } else {
        let user_input = args.input.join(" ");
        println!("Input: '{}'", user_input);
        // We will call llm here
        println!("(Command will be generated here)");
    }
}
