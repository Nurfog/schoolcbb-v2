# Estado del Proyecto — SchoolCBB v2

## Último commit: `2f94834` — Identity Service con JWT + Argon2

---

## ✅ Completado

### Arquitectura — Microservicios Full Rust
- [x] Workspace Cargo con members: `common`, `gateway`, `services/*`, `frontend`
- [x] `packages/common` — tipos compartidos (RUT, Student, Grades, Attendance, User)
- [x] `packages/gateway` — API Gateway (Axum reverse proxy, sin DB)
- [x] `packages/services/identity` — Auth service
- [x] `packages/services/sis` — Student Information System (esqueleto)
- [x] `packages/services/academic` — Notas y calificaciones (esqueleto)
- [x] `packages/services/attendance` — Asistencia (esqueleto)
- [x] `packages/services/notifications` — WebSocket hub
- [x] `packages/frontend` — Dioxus WASM

### Identity Service (Auth real)
- [x] Argon2 para hash de contraseñas
- [x] jsonwebtoken para sesiones (12h expiración)
- [x] `POST /api/auth/login` — login contra DB
- [x] `POST /api/auth/me` — perfil vía JWT
- [x] Seed automático: `admin@colegio.cl` / `admin123`
- [x] Extractor `Claims` via `FromRequestParts`

### Docker
- [x] `Dockerfile` único para todos los servicios backend
- [x] `Dockerfile.frontend` con build Dioxus 0.6
- [x] `docker-compose.yml` con 8 servicios (db, gateway, 5 microservicios, frontend)
- [x] `nginx.conf` con reverse proxy a gateway

### Fixes
- [x] Feature-gate sqlx en common para compilación wasm
- [x] Ruta de output correcta de Dioxus 0.6 (`target/dx/.../public`)
- [x] Mount point `#main` en index.html
- [x] Fix `/*path` en lugar de `/{*path}` en Axum
- [x] Copia de assets (style.css) al output
- [x] Suppress dead_code warnings

---

## 🔄 Siguiente (Etapa 1 del Plan)

### Identity — Próximos pasos
- [ ] Endpoint register para crear usuarios
- [ ] RBAC middleware (solo Admin puede crear usuarios)
- [ ] Refresh tokens

### SIS Service — CRUD Alumnos
- [ ] Validación de RUT con dígito verificador
- [ ] CRUD completo (crear, leer, actualizar, desactivar alumno)
- [ ] Datos médicos (enfermedades, alergias, contactos emergencia)
- [ ] Parentescos (apoderados, familiares)
- [ ] Carga masiva desde CSV/SIGE

### Frontend Shell (estilo iSAMS)
- [ ] Sidebar colapsable con jerarquía (navegación lateral)
- [ ] Búsqueda global Cmd+K (Quick Search por nombre/RUT)
- [ ] Mosaicos inteligentes (Dashboard tiles)
  - [ ] Mosaico Asistencia (% en tiempo real)
  - [ ] Mosaico Alertas (alumnos en riesgo)
  - [ ] Mosaico Agenda (próximos hitos)
- [ ] Paleta de colores iSAMS
  - Primario: Azul Medianoche (#1A2B3C)
  - Fondo: Gris Neutro (#F8F9FA)
  - Acentos: Verde Éxito / Ámbar Alerta

---

## 📐 Puertos

| Servicio | Puerto Interno |
|---|---|
| Gateway | 3000 |
| Identity | 3001 |
| SIS | 3002 |
| Academic | 3003 |
| Attendance | 3004 |
| Notifications | 3005 |
| Frontend (nginx) | 8080 |
| PostgreSQL | 5432 |

---

## 🚀 Comandos rápidos

```bash
# Iniciar todo
docker compose up -d

# Reconstruir un servicio específico
docker compose build --no-cache identity

# Logs
docker compose logs -f gateway identity

# Probar login
curl -X POST http://localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@colegio.cl","password":"admin123"}'
```
