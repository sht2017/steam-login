mod debugger;
mod steam;

use clap::{ArgGroup, Parser};

use crate::steam::get_otp;

/// Steam Login Automation Tool
///
/// Automate Steam login by launching Steam and injecting a JS script via remote debugging.
#[derive(Parser, Debug)]
#[command(
    group(
        ArgGroup::new("guard")
            .required(true)
            .multiple(false)
            .args(["captcha", "shared_secret"])
    )
)]
struct Args {
    /// Path to the Steam executable (Steam.exe / steam)
    #[arg(long)]
    steam: String,

    /// Path to a javascript file
    #[arg(long)]
    js: String,

    /// Steam account name
    #[arg(long)]
    username: String,

    /// Steam account password
    #[arg(long)]
    password: String,

    /// Steam Guard code (email / manual)
    #[arg(long)]
    captcha: Option<String>,

    /// Steam shared secret (base64), used to generate OTP
    #[arg(long, value_name = "BASE64")]
    shared_secret: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let port = steam::start(&args.steam).await?;

    let guard_code = match (&args.captcha, &args.shared_secret) {
        (Some(code), None) => code.clone(),
        (None, Some(secret)) => get_otp(secret)?,
        _ => unreachable!("clap already guarantees exactly one"),
    };

    let result =
        debugger::evaluate(port, &args.js, &args.username, &args.password, &guard_code).await?;
    println!("result: {:?}", result);
    Ok(())
}
