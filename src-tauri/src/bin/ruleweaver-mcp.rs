fn main() {
    let mut port: u16 = 8080;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--port" {
            if let Some(value) = args.next() {
                if let Ok(parsed) = value.parse::<u16>() {
                    port = parsed;
                } else {
                    eprintln!("Invalid --port value: {}", value);
                    std::process::exit(2);
                }
            } else {
                eprintln!("Missing value for --port");
                std::process::exit(2);
            }
        }
    }

    if let Err(e) = ruleweaver_lib::run_mcp_cli(port) {
        eprintln!("ruleweaver-mcp error: {}", e);
        std::process::exit(1);
    }
}
