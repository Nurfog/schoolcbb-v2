#!/usr/bin/env bash
set -euo pipefail

# ─────────────────────────────────────────────────────────────
# SchoolCBB v2 — Setup Inicial
# ─────────────────────────────────────────────────────────────
# Uso: ./setup.sh [--dev]
#   --dev   Modo desarrollo (puertos locales, sin docker)
# ─────────────────────────────────────────────────────────────

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

MODE="${1:-docker}"
ENV_FILE=".env"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[INFO]${NC} $1"; }
ok()    { echo -e "${GREEN}[OK]${NC}   $1"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $1"; }
err()   { echo -e "${RED}[ERR]${NC}  $1"; }

# ─── Banner ──────────────────────────────────────────────────
echo ""
echo "╔═══════════════════════════════════════════════╗"
echo "║        SchoolCBB v2 — Setup Inicial           ║"
echo "║   Plataforma Escolar Multi-Tenant en Rust     ║"
echo "╚═══════════════════════════════════════════════╝"
echo ""

# ─── 1. Verificar dependencias ──────────────────────────────
info "Verificando dependencias..."

if ! command -v docker &>/dev/null; then
    err "Docker no encontrado. Instalalo en https://docs.docker.com/engine/install/"
    exit 1
fi
ok "Docker $(docker --version | cut -d' ' -f3 | cut -d',' -f1)"

if ! docker compose version &>/dev/null; then
    err "Docker Compose no encontrado (docker compose plugin)."
    exit 1
fi
ok "Docker Compose $(docker compose version | cut -d' ' -f4)"

# ─── 2. Configuración interactiva ───────────────────────────
echo ""
info "Configuración de la plataforma"
echo ""

# -- Nombre de la empresa
read -r -p "$(echo -e "${CYAN}Nombre de la empresa${NC} [SchoolCBB]: ")" COMPANY_NAME
COMPANY_NAME="${COMPANY_NAME:-SchoolCBB}"
ok "Nombre: $COMPANY_NAME"

# -- Dominio principal
read -r -p "$(echo -e "${CYAN}Dominio principal${NC} (ej: schoolcbb.cl) [localhost]: ")" DOMAIN
DOMAIN="${DOMAIN:-localhost}"
ok "Dominio: $DOMAIN"

# -- Subdominio de la app
if [ "$DOMAIN" != "localhost" ]; then
    read -r -p "$(echo -e "${CYAN}Subdominio para la app${NC} (ej: app) [app]: ")" APP_SUBDOMAIN
    APP_SUBDOMAIN="${APP_SUBDOMAIN:-app}"
    APP_URL="https://${APP_SUBDOMAIN}.${DOMAIN}"
    PORTAL_URL="https://${DOMAIN}"
else
    APP_URL="http://localhost:8080"
    PORTAL_URL="http://localhost:3010"
fi
ok "App URL: $APP_URL"
ok "Portal URL: $PORTAL_URL"

# -- Root admin
echo ""
info "Cuenta de administrador global (Root)"
echo "Esta cuenta tiene acceso completo a todas las corporaciones."
read -r -p "$(echo -e "${CYAN}Email del Root${NC} [admin@${COMPANY_NAME,,}.cl]: ")" ROOT_EMAIL
ROOT_EMAIL="${ROOT_EMAIL:-admin@${COMPANY_NAME,,}.cl}"

read -r -s -p "$(echo -e "${CYAN}Contraseña del Root${NC} [admin123]: ")" ROOT_PASSWORD
ROOT_PASSWORD="${ROOT_PASSWORD:-admin123}"
echo ""
ok "Root: $ROOT_EMAIL"

# -- Puerto de la app
read -r -p "$(echo -e "${CYAN}Puerto de la app (frontend)${NC} [8080]: ")" APP_PORT
APP_PORT="${APP_PORT:-8080}"
ok "Puerto: $APP_PORT"

# -- Base de datos
echo ""
info "Configuración de base de datos"
read -r -p "$(echo -e "${CYAN}Host DB${NC} [localhost]: ")" DB_HOST
DB_HOST="${DB_HOST:-localhost}"

read -r -p "$(echo -e "${CYAN}Puerto DB${NC} [5432]: ")" DB_PORT
DB_PORT="${DB_PORT:-5432}"

read -r -p "$(echo -e "${CYAN}Usuario DB${NC} [schoolcbb]: ")" DB_USER
DB_USER="${DB_USER:-schoolcbb}"

read -r -s -p "$(echo -e "${CYAN}Contraseña DB${NC} [schoolcbb]: ")" DB_PASS
DB_PASS="${DB_PASS:-schoolcbb}"
echo ""

read -r -p "$(echo -e "${CYAN}Nombre DB${NC} [schoolcbb]: ")" DB_NAME
DB_NAME="${DB_NAME:-schoolcbb}"

if [ "$DOMAIN" != "localhost" ]; then
    DB_URL="postgres://${DB_USER}:${DB_PASS}@db:5432/${DB_NAME}"
else
    DB_URL="postgres://${DB_USER}:${DB_PASS}@${DB_HOST}:${DB_PORT}/${DB_NAME}"
fi

# -- JWT Secret
JWT_SECRET=$(openssl rand -hex 32 2>/dev/null || echo "cambio-en-produccion-por-favor")

# ─── 3. Generar .env ────────────────────────────────────────
echo ""
info "Generando $ENV_FILE..."

cat > "$ENV_FILE" <<ENVEOF
# ─── SchoolCBB v2 — Configuración General ─────────────────
# Generado por setup.sh — $(date +"%Y-%m-%d %H:%M:%S")
COMPANY_NAME="${COMPANY_NAME}"
DOMAIN="${DOMAIN}"
APP_URL="${APP_URL}"
PORTAL_URL="${PORTAL_URL}"

# ─── Root Admin ───────────────────────────────────────────
ROOT_EMAIL="${ROOT_EMAIL}"
ROOT_PASSWORD="${ROOT_PASSWORD}"

# ─── Base de Datos ────────────────────────────────────────
DATABASE_URL=${DB_URL}

# ─── JWT ──────────────────────────────────────────────────
JWT_SECRET=${JWT_SECRET}

# ─── Gateway ──────────────────────────────────────────────
GATEWAY_HOST=0.0.0.0
GATEWAY_PORT=3000
FRONTEND_URL=${APP_URL}
IDENTITY_URL=http://identity:3001
SIS_URL=http://sis:3002
ACADEMIC_URL=http://academic:3003
ATTENDANCE_URL=http://attendance:3004
NOTIFICATIONS_URL=http://notifications:3005
FINANCE_URL=http://finance:3006
REPORTING_URL=http://reporting:3007

# ─── Servicios ────────────────────────────────────────────
IDENTITY_HOST=0.0.0.0
IDENTITY_PORT=3001

SIS_HOST=0.0.0.0
SIS_PORT=3002

ACADEMIC_HOST=0.0.0.0
ACADEMIC_PORT=3003

ATTENDANCE_HOST=0.0.0.0
ATTENDANCE_PORT=3004

NOTIFICATIONS_HOST=0.0.0.0
NOTIFICATIONS_PORT=3005

FINANCE_HOST=0.0.0.0
FINANCE_PORT=3006
FINANCE_GRPC_URL=http://finance:4006

REPORTING_HOST=0.0.0.0
REPORTING_PORT=3007

PORTAL_HOST=0.0.0.0
PORTAL_PORT=3010

# ─── Logging ──────────────────────────────────────────────
RUST_LOG=info,schoolcbb=debug
ENVEOF

ok "$ENV_FILE generado"

# ─── 4. Migraciones SQL ─────────────────────────────────────
echo ""
info "Migraciones de base de datos"

if [ "$DOMAIN" != "localhost" ] || [ "$DB_HOST" != "localhost" ]; then
    warn "Base de datos remota — asegúrate de ejecutar las migraciones manualmente."
    warn "Las tablas se crean automáticamente al iniciar los servicios."
else
    ok "Las tablas se crearán automáticamente al iniciar los servicios (db_schema.rs)."
fi

# ─── 5. Docker Compose ──────────────────────────────────────
echo ""
info "Servicios Docker disponibles en docker-compose.yml:"
echo "  db, gateway, identity, sis, academic, attendance,"
echo "  notifications, finance, reporting, frontend"

if [ "$DOMAIN" == "localhost" ]; then
    echo ""
    info "Iniciando servicios..."
    docker compose up -d
    echo ""
    ok "Servicios iniciados"
    echo ""
    echo "  Frontend:  http://localhost:${APP_PORT}"
    echo "  API:       http://localhost:3000"
    echo "  Root login: ${ROOT_EMAIL} / ${ROOT_PASSWORD}"
else
    echo ""
    info "Configuración para producción detectada."
    echo ""
    echo "Pasos siguientes:"
    echo "  1. Revisa docker-compose.yml y ajusta dominios/SSL"
    echo "  2. Configura nginx con HTTPS (Certbot/Let's Encrypt)"
    echo "  3. Inicia con: docker compose up -d"
    echo ""
    warn "No olvides cambiar JWT_SECRET por un valor seguro en producción."
fi

# ─── 6. Resumen ─────────────────────────────────────────────
echo ""
echo "╔═══════════════════════════════════════════════╗"
echo "║           Setup completado                     ║"
echo "╠═══════════════════════════════════════════════╣"
printf "║ %-20s %-25s ║\n" "Empresa" "$COMPANY_NAME"
printf "║ %-20s %-25s ║\n" "Root email" "$ROOT_EMAIL"
printf "║ %-20s %-25s ║\n" "App URL" "$APP_URL"
printf "║ %-20s %-25s ║\n" "Portal URL" "$PORTAL_URL"
printf "║ %-20s %-25s ║\n" "JWT Secret" "${JWT_SECRET:0:16}..."
echo "╚═══════════════════════════════════════════════╝"
echo ""
