use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use hmac::{Hmac, Mac};
use sha1::Sha1;

use tokio::{net::TcpListener, process::Command};

async fn get_free_port() -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    Ok(listener.local_addr()?.port())
}

pub async fn start(steam: &str) -> Result<u16, Box<dyn std::error::Error>> {
    let steam_path = Path::new(&steam);
    if !steam_path.exists() || !steam_path.is_file() {
        return Err(format!("Steam executable not found or not a file at '{}'", steam).into());
    }

    let mut cmd = Command::new(steam);
    let port = get_free_port().await?;
    cmd.args(["-cef-enable-debugging", "-devtools-port", &port.to_string()]);

    #[cfg(windows)]
    {
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }

    #[cfg(unix)]
    {
        use std::process::Stdio;

        cmd.stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        unsafe {
            cmd.pre_exec(|| {
                if libc::setsid() == -1 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
    }

    cmd.spawn()?;

    Ok(port)
}

pub fn get_otp(shared_secret: &str) -> Result<String, Box<dyn std::error::Error>> {
    let key = STANDARD
        .decode(shared_secret.trim())
        .map_err(|_| "bad/empty shared_secret")?;

    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let time_slice = now / 30;

    let msg = time_slice.to_be_bytes();

    let mut mac = Hmac::<Sha1>::new_from_slice(&key)?;
    mac.update(&msg);
    let hmac = mac.finalize().into_bytes();

    let offset = (hmac[19] & 0x0f) as usize;
    let part = &hmac[offset..offset + 4];
    let mut code_int = u32::from_be_bytes([part[0], part[1], part[2], part[3]]) & 0x7fffffff;

    const STEAM_CHARS: &[u8; 26] = b"23456789BCDFGHJKMNPQRTVWXY";

    let mut otp = [0u8; 5];
    for i in 0..5 {
        otp[i] = STEAM_CHARS[(code_int % 26) as usize];
        code_int /= 26;
    }

    Ok(std::str::from_utf8(&otp)?.to_string())
}
