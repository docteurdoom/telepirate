# TelePirate Telegram Bot
### Download music and videos from anywhere via Telegram

Manual installation steps for Ubuntu:
	
    sudo su root
    add-apt-repository ppa:tomtomtom/yt-dlp
	apt install -y git cargo yt-dlp
    git clone https://github.com/docteurdoom/telepirate.git /opt/telepirate
    cd /opt/telepirate && cargo build --release
    mv -v target/release/telepirate ${PWD}
    mv -v systemd-services/telepirate.service /etc/systemd/system

Edit `/opt/telepirate/env` file. Replace placeholders with real values. 
The file should contain only the 2 following lines:

    NGROK_AUTHTOKEN="your_token_from_ngrok.com"
    TELOXIDE_TOKEN="your_bot's_telegram_api_token"

Run a `systemd` service:

	systemctl enable telepirate
    systemctl start telepirate

### Notes
The bot uses `yt-dlp` to download files.
At the moment of writing this readme, `yt-dlp` supports more than 1800 resources to download from.
