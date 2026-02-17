#!/usr/bin/env bash
set -euo pipefail

SERVICE_NAME=shutters-service
UNIT_SRC=deploy/${SERVICE_NAME}.service
UNIT_DST=/etc/systemd/system/${SERVICE_NAME}.service
INSTALL_DIR=/opt/${SERVICE_NAME}
EXECUTABLE=${INSTALL_DIR}/${SERVICE_NAME}

usage(){
  cat <<EOF
Usage: $0 {install|start|stop|restart|status|uninstall}
  install   Build, copy binary and unit, enable and start service
  start     Start the service
  stop      Stop the service
  restart   Restart the service
  status    Show systemd status for the service
  uninstall Disable service, remove unit and installed files
EOF
}

case "${1:-}" in
  install)
    echo "Building release..."
    cargo build --release

    echo "Creating system user 'shutters' (if missing)..."
    if ! id -u shutters >/dev/null 2>&1; then
      sudo useradd -r -s /usr/sbin/nologin shutters || true
    fi

    echo "Installing binary to ${INSTALL_DIR}..."
    sudo mkdir -p "${INSTALL_DIR}"
    sudo cp target/release/${SERVICE_NAME} "${EXECUTABLE}"
    sudo chown -R shutters:shutters "${INSTALL_DIR}"

    echo "Installing systemd unit..."
    sudo cp "${UNIT_SRC}" "${UNIT_DST}"
    sudo systemctl daemon-reload
    sudo systemctl enable --now "${SERVICE_NAME}"
    echo "Installed and started ${SERVICE_NAME}."
    ;;
  start)
    sudo systemctl start "${SERVICE_NAME}"
    ;;
  stop)
    sudo systemctl stop "${SERVICE_NAME}"
    ;;
  restart)
    sudo systemctl restart "${SERVICE_NAME}"
    ;;
  status)
    sudo systemctl status "${SERVICE_NAME}"
    ;;
  uninstall)
    sudo systemctl disable --now "${SERVICE_NAME}" || true
    sudo rm -f "${UNIT_DST}"
    sudo systemctl daemon-reload
    sudo rm -rf "${INSTALL_DIR}"
    sudo userdel shutters || true
    echo "Uninstalled ${SERVICE_NAME}."
    ;;
  *)
    usage
    exit 2
    ;;
esac
