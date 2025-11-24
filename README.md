# steam-login

A command-line utility for automating Steam client authentication.

## SYNOPSIS

    steam-login --steam <PATH> [--js <PATH>] --username <USER> --password <PASS> (--captcha <CODE> | --shared_secret <SECRET>)

## DESCRIPTION

`steam-login` automates the Steam login process by launching the client with the Chromium Embedded Framework (CEF) remote debugging port enabled. It connects to the debugging interface via WebSocket and injects a JavaScript payload to populate credentials and handle Two-Factor Authentication (2FA).

This tool is designed for headless environments or automated gaming setups where manual entry of credentials is impractical.

## OPTIONS

*   `--steam <PATH>`
    Absolute path to the Steam executable (e.g., `C:\Program Files (x86)\Steam\steam.exe` or `/usr/bin/steam`).

*   `--js <PATH>` (optional)
    Path to a custom automation script. If not provided, uses the embedded default script.

*   `--username <STRING>`
    Steam account username.

*   `--password <STRING>`
    Steam account password.

*   `--captcha <STRING>`
    Manual Steam Guard code (email or mobile). Mutually exclusive with `--shared_secret`.

*   `--shared_secret <BASE64>`
    The shared secret from your Steam mobile authenticator (Base64 encoded). If provided, `steam-login` will generate the current TOTP code automatically.

## EXAMPLES

**Login with a manually provided Steam Guard code:**

    $ steam-login \
        --steam "C:\Program Files (x86)\Steam\steam.exe" \
        --username myuser \
        --password mypass \
        --captcha AB123

**Login with automatic TOTP generation:**

    $ steam-login \
        --steam /usr/bin/steam \
        --username myuser \
        --password mypass \
        --shared_secret "U29tZVJhbmRvbVNlY3JldA=="

**Login with a custom JavaScript automation script:**

    $ steam-login \
        --steam /usr/bin/steam \
        --js custom-script.js \
        --username myuser \
        --password mypass \
        --captcha AB123

## BUILDING

Ensure you have the Rust toolchain installed.

    $ cargo build --release

The resulting binary will be found in `target/release/`.

## LICENSE

This project is dedicated to the public domain under the CC0 1.0 Universal license.
