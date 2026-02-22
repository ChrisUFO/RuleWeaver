use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// API token for authentication (optional, generated if not provided)
    #[arg(short, long)]
    token: Option<String>,
}

fn main() {
    env_logger::init();
    log::info!("MCP Server starting up");

    let args = Args::parse();
    let port = args.port;

    log::info!("Starting MCP server on port {}", port);
    if let Err(e) = ruleweaver_lib::run_mcp_cli(port, args.token) {
        log::error!("MCP server error: {}", e);
        eprintln!("ruleweaver-mcp error: {}", e);
        std::process::exit(1);
    }
}
