#!/bin/bash
set -e

# ─── prd-reviewer installer ───────────────────────────────────────────────

REPO_URL="https://github.com/arimunandar/prd-reviewer-cli.git"
CONFIG_FILE="$HOME/.prd-reviewer.yaml"

# Colours
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# Cleanup on early exit
CLEANUP_DIR=""
cleanup() {
    local exit_code=$?
    if [ -n "$CLEANUP_DIR" ] && [ -d "$CLEANUP_DIR" ]; then
        rm -rf "$CLEANUP_DIR"
    fi
    if [ $exit_code -ne 0 ]; then
        echo ""
        echo -e "${RED}Installation aborted.${NC}"
    fi
}
trap cleanup EXIT INT TERM

# ─── Helpers ──────────────────────────────────────────────────────────────

banner() {
    echo ""
    echo -e "${BOLD}===================================${NC}"
    echo -e "${BOLD}  $1${NC}"
    echo -e "${BOLD}===================================${NC}"
    echo ""
}

ok()   { echo -e "  ${GREEN}✓${NC} $1"; }
info() { echo -e "  ${BLUE}›${NC} $1"; }
warn() { echo -e "  ${YELLOW}!${NC} $1"; }
err()  { echo -e "  ${RED}✗${NC} $1"; }

# Prompt for a required, non-empty value.
# Usage: prompt_required "label" "hint" VAR_NAME
prompt_required() {
    local label="$1"
    local hint="$2"
    local __var="$3"
    local value=""
    while [ -z "$value" ]; do
        if [ -n "$hint" ]; then
            echo -e "  ${DIM}$hint${NC}"
        fi
        echo -n "  $label: "
        read -r value
        if [ -z "$value" ]; then
            err "Required — cannot be empty."
        fi
    done
    eval "$__var=\"\$value\""
}

# Prompt for a required URL (must start with http:// or https://).
prompt_url() {
    local label="$1"
    local hint="$2"
    local __var="$3"
    local value=""
    while true; do
        if [ -n "$hint" ]; then
            echo -e "  ${DIM}$hint${NC}"
        fi
        echo -n "  $label: "
        read -r value
        if [ -z "$value" ]; then
            err "Required — cannot be empty."
            continue
        fi
        if [[ "$value" != http://* && "$value" != https://* ]]; then
            err "Must start with http:// or https://"
            continue
        fi
        break
    done
    eval "$__var=\"\$value\""
}

# Prompt for a required secret (hidden input).
prompt_secret() {
    local label="$1"
    local hint="$2"
    local __var="$3"
    local value=""
    while [ -z "$value" ]; do
        if [ -n "$hint" ]; then
            echo -e "  ${DIM}$hint${NC}"
        fi
        echo -n "  $label: "
        read -rs value
        echo ""
        if [ -z "$value" ]; then
            err "Required — cannot be empty."
        fi
    done
    eval "$__var=\"\$value\""
}

# Prompt for an optional secret (hidden, blank allowed).
prompt_secret_optional() {
    local label="$1"
    local hint="$2"
    local __var="$3"
    if [ -n "$hint" ]; then
        echo -e "  ${DIM}$hint${NC}"
    fi
    echo -en "  $label ${DIM}(optional, press Enter to skip)${NC}: "
    local value=""
    read -rs value
    echo ""
    eval "$__var=\"\$value\""
}

show_next_steps() {
    banner "Next Steps"
    echo "  1. Navigate to any project:"
    echo "       cd /path/to/your-project"
    echo ""
    echo "  2. Initialize prd-reviewer (installs skill + agent + CLAUDE.md):"
    echo "       prd-reviewer init"
    echo ""
    echo "  3. Start using:"
    echo "       prd-reviewer prd fetch <PAGE_ID> --raw   # fetch a PRD"
    echo "       prd-reviewer prd rules                   # 11-section rules (markdown)"
    echo "       prd-reviewer prd rules --json            # rules as JSON (for AI)"
    echo "       prd-reviewer prd workflow                # review workflow steps"
    echo "       prd-reviewer prd template                # PRD template (11 sections)"
    echo "       prd-reviewer jira --help                 # Jira & Wiki tools"
    echo "       prd-reviewer figma --help                # Figma tools"
    echo ""
    echo "  4. Inside Claude Code:"
    echo "       /prd-reviewer <page_id | URL | feature brief>"
    echo "       @prd-reviewer  (autonomous end-to-end)"
    echo ""
    echo "  Update later:  prd-reviewer update"
    echo ""
}

# ─── Start ────────────────────────────────────────────────────────────────

banner "prd-reviewer installer"

# ─── Step 1: Rust toolchain ───────────────────────────────────────────────

echo -e "${BOLD}Step 1/4 — Rust toolchain${NC}"
echo ""

if ! command -v cargo &> /dev/null; then
    warn "Rust (cargo) not found. Installing via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    # shellcheck disable=SC1091
    source "$HOME/.cargo/env"
    ok "Rust installed."
else
    ok "Rust found: $(cargo --version)"
fi
echo ""

# ─── Step 2: Build & install binary ───────────────────────────────────────

echo -e "${BOLD}Step 2/4 — Build & install the binary${NC}"
echo ""

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
if [ -f "$SCRIPT_DIR/Cargo.toml" ] && grep -q 'name = "prd-reviewer"' "$SCRIPT_DIR/Cargo.toml" 2>/dev/null; then
    BUILD_DIR="$SCRIPT_DIR"
    info "Installing from local repo: $BUILD_DIR"
else
    BUILD_DIR="/tmp/prd-reviewer-install-$$"
    CLEANUP_DIR="$BUILD_DIR"
    info "Cloning from $REPO_URL"
    rm -rf "$BUILD_DIR"
    git clone --depth 1 "$REPO_URL" "$BUILD_DIR" > /dev/null 2>&1 || {
        err "git clone failed."
        exit 1
    }
    ok "Cloned to $BUILD_DIR"
fi
echo ""

info "Running: cargo install --path . --force"
(cd "$BUILD_DIR" && cargo install --path . --force) || {
    err "cargo install failed."
    echo ""
    echo "  Try manually:"
    echo "    cd $BUILD_DIR && cargo install --path ."
    exit 1
}

if ! command -v prd-reviewer &> /dev/null; then
    err "Binary not found on PATH after install."
    echo "  Check that ~/.cargo/bin is in your PATH:"
    echo "    echo 'export PATH=\"\$HOME/.cargo/bin:\$PATH\"' >> ~/.zshrc"
    exit 1
fi

ok "Installed: $(prd-reviewer version)"
echo ""

# ─── Step 3: Credentials ──────────────────────────────────────────────────

echo -e "${BOLD}Step 3/4 — Configure credentials${NC}"
echo ""
info "Config will be written to: $CONFIG_FILE"
echo ""

if [ -f "$CONFIG_FILE" ]; then
    warn "Config already exists at $CONFIG_FILE"
    echo -n "  Overwrite? (y/N): "
    read -r OVERWRITE
    if [ "$OVERWRITE" != "y" ] && [ "$OVERWRITE" != "Y" ]; then
        ok "Keeping existing config."
        show_next_steps
        exit 0
    fi
    echo ""
fi

# ─── Jira ────────────────────────────────────────────────────────────────

echo -e "${BOLD}Jira${NC} ${DIM}(required — used for ticket lookups)${NC}"
echo ""

prompt_url "Base URL" \
    "e.g. https://your-jira.example.com/rest/api/2" \
    JIRA_URL

prompt_secret "Access Token (Bearer)" \
    "Create at: <Jira> → Profile → Personal Access Tokens" \
    JIRA_TOKEN

ok "Jira configured."
echo ""

# ─── Wiki (Confluence) ───────────────────────────────────────────────────

echo -e "${BOLD}Confluence Wiki${NC} ${DIM}(required — used to fetch and post PRDs)${NC}"
echo ""

prompt_url "Base URL" \
    "e.g. https://your-wiki.example.com/rest/api/content" \
    WIKI_URL

prompt_secret "Access Token (Bearer)" \
    "Create at: <Confluence> → Profile → Personal Access Tokens" \
    WIKI_TOKEN

ok "Wiki configured."
echo ""

# ─── Figma ────────────────────────────────────────────────────────────────

echo -e "${BOLD}Figma${NC} ${DIM}(optional — used to inspect PRD design references)${NC}"
echo ""

prompt_secret_optional "Personal Access Token" \
    "Create at: Figma → Settings → Security → Personal access tokens" \
    FIGMA_TOKEN

if [ -n "$FIGMA_TOKEN" ]; then
    ok "Figma configured."
else
    warn "Figma skipped — \`prd-reviewer figma\` commands will be disabled until set."
fi
echo ""

# ─── Step 4: Write config ─────────────────────────────────────────────────

echo -e "${BOLD}Step 4/4 — Write config${NC}"
echo ""

umask 077
cat > "$CONFIG_FILE" << CONFIGEOF
jira:
  access_token: "$JIRA_TOKEN"
  base_url: "$JIRA_URL"
wiki:
  access_token: "$WIKI_TOKEN"
  base_url: "$WIKI_URL"
figma:
  personal_token: "$FIGMA_TOKEN"
  default_team_id: ""
  export_dir: "./"
CONFIGEOF

chmod 600 "$CONFIG_FILE"
ok "Config saved to $CONFIG_FILE (permissions 600)"
echo ""

show_next_steps

echo -e "${GREEN}✓ Installation complete.${NC}"
echo ""
