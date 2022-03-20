use argh::FromArgs;
use asmintr::Interpreter;

/// Run assembly code
#[derive(FromArgs)]
struct Cli {
    /// '.asm' file path
    #[argh(positional)]
    file_name: String,

    /// debug interpreter registers, stack, flags and output
    #[argh(switch, short = 'd')]
    debug: bool,

    /// print parsed instructions
    #[argh(switch, short = 'i')]
    inst: bool,
}
fn main() {
    let cli: Cli = argh::from_env();

    let content = std::fs::read_to_string(cli.file_name).unwrap();
    let (interpreter, actual_output) = Interpreter::interpret(content.as_str());

    if cli.inst {
        println!("Instructions: {:?}", interpreter.program.instructions);
    }

    if cli.debug {
        println!("{}\nActual Output is : {:?}", interpreter, actual_output);
    } else {
        println!("{:?}", actual_output);
    }
}