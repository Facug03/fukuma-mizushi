# Deploying fukuma-mizushi to Lichess

## 1. Build release binary

```bash
cargo build --release -p fukuma-mizushi
# Binary: target/release/fukuma-mizushi
```

## 2. Smoke test (UCI handshake)

Verify the binary speaks UCI before deploying:

```bash
echo -e "uci\nisready\nquit" | ./target/release/fukuma-mizushi
# Expected output:
# id name fukuma-mizushi
# id author Kiro [claude-sonnet-4-5] (Amazon)
# uciok
# readyok
```

## 3. Set up lichess-bot

[lichess-bot](https://github.com/lichess-bot-devs/lichess-bot) is the official Python client
that bridges a UCI engine to the Lichess API.

```bash
git clone https://github.com/lichess-bot-devs/lichess-bot
cd lichess-bot
pip install -r requirements.txt
cp config.yml.default config.yml
```

Edit `config.yml`:

```yaml
token: "${LICHESS_TOKEN}"   # never hardcode — use env variable
engine:
  dir: /path/to/fukuma-mizushi   # directory containing the binary
  name: fukuma-mizushi
  protocol: uci
  ponder: false
```

**Security**: The Lichess API token is a secret.
- Store it in an environment variable: `export LICHESS_TOKEN=lip_xxxx`
- Add `config.yml` to `.gitignore` if it contains the token directly
- Never commit a token to git

## 4. Run the bot

```bash
LICHESS_TOKEN=lip_xxxx python lichess-bot.py
```

## 5. Server deployment (example: systemd)

```ini
# /etc/systemd/system/fukuma-mizushi.service
[Unit]
Description=fukuma-mizushi Lichess bot

[Service]
WorkingDirectory=/opt/lichess-bot
ExecStart=/usr/bin/python lichess-bot.py
EnvironmentFile=/etc/fukuma-mizushi/env   # contains LICHESS_TOKEN=...
Restart=on-failure

[Install]
WantedBy=multi-user.target
```

```bash
sudo systemctl enable --now fukuma-mizushi
```
