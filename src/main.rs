mod client;
mod convert;
mod error;
mod models;

use clap::Parser;

#[derive(Parser)]
#[command(version)]
struct Cli {
    #[arg(long)]
    base_url: String,
    #[arg(long)]
    token: Option<String>,
}

fn validate_base_url(raw: &str) -> Result<String, String> {
    let parsed =
        url::Url::parse(raw).map_err(|e| format!("invalid URL: {e}"))?;

    match parsed.scheme() {
        "http" | "https" => {}
        s => return Err(format!("unsupported scheme '{s}', expected http or https")),
    }

    if parsed.path() != "/" && !parsed.path().is_empty() {
        return Err(format!(
            "base-url must be an origin (no path), got '{}'",
            parsed.path()
        ));
    }
    if parsed.query().is_some() {
        return Err("base-url must not contain a query string".into());
    }
    if parsed.fragment().is_some() {
        return Err("base-url must not contain a fragment".into());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("base-url must not contain credentials".into());
    }

    let s = parsed.as_str().trim_end_matches('/');
    Ok(s.to_owned())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let base_url = match validate_base_url(&cli.base_url) {
        Ok(u) => u,
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    };

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("base_url: {base_url}");
    if cli.token.is_some() {
        tracing::info!("token: configured");
    } else {
        tracing::info!("token: not configured");
    }

    // TODO: build OjClient, OjServer, start transport

    Ok(())
}
