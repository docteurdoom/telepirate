#!/bin/bash

eerror() {
	echo -e " \e[1;31m*\e[0m ${@}\e[0m" >&2
}

die() {
	eerror "${@}"
	exit 1
}

einfo() {
	echo -e " \e[1;32m*\e[0m ${@}\e[0m" >&2
}

checks () {
	if [ "$(id -u)" -eq 0 ]
		then
		einfo "Running as root ..."
		else
		die "Please run this script as root user. Exiting."
		exit
	fi
}

prepare() {
	einfo "Removing old yt-dlp repository ..."
	add-apt-repository -r ppa:tomtomtom/yt-dlp -y || die "Removing old repository failed."

	einfo "Adding yt-dlp repository ..."
	add-apt-repository ppa:tomtomtom/yt-dlp -y || die "Adding repository failed."

	einfo "Syncing repositories ..."
	apt update || die "Sync failed."

	einfo "Updating the system ..."
	apt upgrade -y || die "System update failed."

	einfo "Installing dependencies ..."
	apt install -y git cargo yt-dlp ffmpeg gcc pkg-config openssl libssl-dev || die "Failed to install dependencies."
}

NAME="telepirate"
WD="/opt/${NAME}"

download() {
	einfo "Downloading the source code ..."
	rm -vrf ${WD}
	git clone https://github.com/docteurdoom/telepirate.git ${WD} || die "Failed to git clone the source code."

	read -p "Paste your Ngrok token: " NGROK || die "Failed to receive Ngrok token."
	echo "NGROK_AUTHTOKEN=\"${NGROK}\"" > ${WD}/env
	read -p "Paste your Telegram HTTP API token: " TG || die "Failed to receive Telegram API token."
	echo "TELOXIDE_TOKEN=\"${TG}\"" >> ${WD}/env
}

compile() {
	einfo "Compiling the bot ..."
	cd ${WD} && cargo build --release --verbose || die "Failed to compile the bot."
	mv -v "target/release/${NAME}" "${WD}"
}

launch() {
	einfo "Adding systemd services ..."
	cp -v "${WD}/systemd-services/${NAME}.service" "/etc/systemd/system"

	einfo "Enabling ${NAME} to start on boot ..."
	systemctl enable "${NAME}" || die "Failed to make service launch on boot."

	einfo "Starting ${NAME} ..."
	systemctl start "${NAME}" || die "Failed to start a service."

	einfo "Done. The bot is now running."
}

checks && prepare && download && compile && launch
