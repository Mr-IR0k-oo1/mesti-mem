#!/usr/bin/env bash
# install.sh — build and install matis-mem to ~/.local/bin
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INSTALL_DIR="${HOME}/.local/bin"

BOLD="\033[1m"; GREEN="\033[32m"; CYAN="\033[36m"; YELLOW="\033[33m"; RESET="\033[0m"

echo -e "${BOLD}Building matis-mem (release)...${RESET}"
cd "$SCRIPT_DIR"
cargo build --release

strip target/release/matis-mem 2>/dev/null || true
mkdir -p "$INSTALL_DIR"
cp target/release/matis-mem "$INSTALL_DIR/matis-mem"
chmod +x "$INSTALL_DIR/matis-mem"

echo -e "${GREEN}✓${RESET} Installed → ${CYAN}$INSTALL_DIR/matis-mem${RESET}"

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    echo -e "\n  ${YELLOW}Add to PATH:${RESET}"
    echo -e "  ${CYAN}echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc${RESET}"
fi

echo -e "\n  ${BOLD}Usage:${RESET}  matis-mem"
echo -e "  ${BOLD}Data:${RESET}   ~/.matis-mem/"
echo -e "\n  ${BOLD}Models you can use:${RESET}"
echo -e "    ollama  →  ${CYAN}ollama pull llama3${RESET}"
echo -e "    gemini  →  ${CYAN}npm install -g @google/gemini-cli && gemini auth${RESET}"
