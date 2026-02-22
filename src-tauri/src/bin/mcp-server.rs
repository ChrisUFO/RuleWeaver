fn main() {
    env_logger::init();
    log::info!("MCP Server starting up");
    
    let mut port: u16 = 8080;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--port" {
            if let Some(value) = args.next() {
                if let Ok(parsed) = value.parse::<u16>() {
                    port = parsed;
                } else {
                    log::error!("Invalid --port value: {}", value);
                    eprintln!("Invalid --port value: {}", value);
                    std::process::exit(2);
                }
            } else {
                log::error!("Missing value for --port");
                eprintln!("Missing value for --port");
                std::process::exit(2);
            }
        }
    }

    log::info!("Starting MCP server on port {}", port);
    if let Err(e) = ruleweaver_lib::run_mcp_cli(port) {
        log::error!("MCP server error: {}", e);
        eprintln!("mcp-server error: {}", e);
        std::process::exit(1);
    }
}
