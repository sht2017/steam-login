use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::{fs, time::timeout};
use tokio_tungstenite::connect_async;

pub async fn evaluate(
    port: u16,
    js: Option<&str>,
    username: &str,
    password: &str,
    captcha: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let js = match js {
        Some(path) => fs::read_to_string(path).await?,
        None => include_str!("../javascript/steam-login.js").to_string(),
    }
    .replace("{%username%}", username)
    .replace("{%password%}", password)
    .replace("{%captcha%}", captcha);

    let url = format!("http://127.0.0.1:{port}/json");
    println!("[DEBUG] Waiting for Steam debugger at: {}", url);

    let ws_url: String = timeout(Duration::from_secs(180), async {
        loop {
            if let Ok(resp) = reqwest::get(&url).await {
                if let Ok(val) = resp.json::<Value>().await {
                    if let Some(arr) = val.as_array() {
                        if let Some(item) = arr.iter().find(|item| {
                            item.get("url")
                                .and_then(|x| x.as_str())
                                .map(|u| u.starts_with("about:blank"))
                                .unwrap_or(false)
                        }) {
                            if let Some(ws) =
                                item.get("webSocketDebuggerUrl").and_then(|x| x.as_str())
                            {
                                println!("[DEBUG] Found WebSocket URL: {}", ws);
                                return Ok::<String, Box<dyn std::error::Error>>(ws.to_string());
                            }
                        }
                    }
                }
            }

            print!(".");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    })
    .await??;
    println!();

    println!("[DEBUG] Connecting to WebSocket...");
    let (ws, _) = timeout(Duration::from_secs(60), async {
        let res = connect_async(&ws_url).await?;
        Ok::<_, Box<dyn std::error::Error>>(res)
    })
    .await??;
    println!("[DEBUG] WebSocket connected successfully");

    let (mut write, mut read) = ws.split();
    let mut id: u64 = 0;
    println!("[DEBUG] Initializing Chrome DevTools Protocol...");

    let mut send = |method: &str, params: Option<Value>| {
        id += 1;
        let mut msg = json!({"id": id, "method": method});
        if let Some(p) = params {
            msg["params"] = p;
        }
        (id, msg.to_string())
    };

    let (_enable_id, msg) = send("Runtime.enable", None);
    write.send(msg.into()).await?;
    println!("[DEBUG] Sent Runtime.enable");

    let (eval_id, msg) = send(
        "Runtime.evaluate",
        Some(json!({
            "expression": js,
            "returnByValue": true,
            "awaitPromise": true
        })),
    );
    write.send(msg.into()).await?;
    println!("[DEBUG] Sent JavaScript evaluation request, waiting for response...");

    let resp = timeout(Duration::from_secs(60), async {
        while let Some(msg) = read.next().await {
            let msg = msg?;
            if !msg.is_text() {
                continue;
            }
            let v: Value = serde_json::from_str(msg.to_text()?)?;

            if v.get("id").and_then(|x| x.as_u64()) == Some(eval_id) {
                println!("[DEBUG] Received evaluation response");
                return Ok::<_, Box<dyn std::error::Error>>(v);
            }
        }
        Err::<Value, Box<dyn std::error::Error>>("WebSocket closed before eval response".into())
    })
    .await??;

    println!("[DEBUG] Evaluation completed successfully");
    return Ok(resp);
}
