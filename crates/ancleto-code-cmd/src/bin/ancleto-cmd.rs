fn main() {
    let options = parse_options();
    if let Err(error) = ancleto_code_cmd::run(options) {
        eprintln!("acmd: {error}");
        std::process::exit(1);
    }
}

fn parse_options() -> ancleto_code_cmd::CmdOptions {
    let mut options = ancleto_code_cmd::CmdOptions::default();
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project" => options.project = args.next(),
            "--shell" => options.shell = args.next(),
            "--theme" => options.theme = args.next(),
            "--kill-on-close" => options.kill_on_close = true,
            "--version" | "-V" => {
                println!("acmd {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!("acmd — AncletoCMD");
                println!("Uso: acmd [--project <id|ruta>] [--shell <id>] [--theme <id>] [--kill-on-close]");
                std::process::exit(0);
            }
            other => {
                eprintln!("acmd: opción desconocida: {other}");
                std::process::exit(2);
            }
        }
    }
    options
}
