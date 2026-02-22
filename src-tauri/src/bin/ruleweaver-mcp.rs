fn main() {
    let mut port: u16 = 8080;
    let args: Vec<String> = std::env::args().collect();

    let mut i = 0usize;
    while i < args.len() {
        if args[i] == "--port" {
            if let Some(value) = args.get(i + 1) {
                if let Ok(parsed) = value.parse::<u16>() {
                    port = parsed;
                }
            }
            i += 1;
        }
        i += 1;
    }

    if let Err(e) = ruleweaver_lib::run_mcp_cli(port) {
        eprintln!("ruleweaver-mcp error: {}", e);
        std::process::exit(1);
    }
}
