fn main() {
    let options = parse_options();
    if let Err(error) = ancleto_code_cmd::run(options) {
        eprintln!("acmd: {error}");
        std::process::exit(1);
    }
}

fn parse_options() -> ancleto_code_cmd::CmdOptions {
    let mut options = ancleto_code_cmd::CmdOptions::default();
    let mut install_hooks: Option<String> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--project" => options.project = args.next(),
            "--shell" => options.shell = args.next(),
            "--theme" => options.theme = args.next(),
            "--kill-on-close" => options.kill_on_close = true,
            "--install-hooks" => install_hooks = args.next(),
            "--version" | "-V" => {
                println!("acmd {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--help" | "-h" => {
                println!("acmd — AncletoCMD");
                println!("Uso: acmd [--project <id|ruta>] [--shell <id>] [--theme <id>] [--kill-on-close]");
                println!("     acmd --install-hooks <bash|zsh|fish|powershell> [--shell <id>]");
                std::process::exit(0);
            }
            other => {
                eprintln!("acmd: opción desconocida: {other}");
                std::process::exit(2);
            }
        }
    }
    if let Some(shell) = install_hooks {
        match ancleto_code_cmd::profiles::install_hooks(&shell) {
            Ok(path) => {
                println!("acmd: hooks OSC 133 instalados en {}", path.display());
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("acmd: no se pudieron instalar los hooks: {error}");
                std::process::exit(1);
            }
        }
    }
    options
}
