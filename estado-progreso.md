# Estado del Proyecto — SchoolCBB v2

## Último commit: `2f94834` — Identity Service con JWT + Argon2
## Sesión actual: Etapa 2 completa (Academic + Attendance + Frontend)

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

## ✅ Identity — Completado
- [x] Endpoint register (`POST /api/auth/register`) — solo Administrador/Sostenedor
- [x] RBAC middleware (`require_any_role`) con 7 roles del plan de desarrollo
- [x] Refresh tokens con tabla `refresh_tokens`, expiración 7d, rotación segura
- [x] Claims actualizados: `sub`, `role`, `name`, `email`, `exp`, `iat`
- [x] Roles implementados: Sostenedor, Director, UTP, Administrador, Profesor, Apoderado, Alumno

## ✅ SIS Service — CRUD Alumnos completado
- [x] Validación RUT con dígito verificador (`POST /api/students` lo valida antes de insertar)
- [x] CRUD completo: `GET /api/students` (con filtros grade_level/section/search), `GET /api/students/{id}`, `POST /api/students`, `PUT /api/students/{id}`, `DELETE /api/students/{id}` (desactivación soft)
- [x] Datos médicos: enfermedades, alergias, contacto emergencia (3 campos)
- [x] Parentescos: tabla `guardian_relationships` + endpoints CRUD vinculación apoderado-alumno
- [x] Autenticación JWT en SIS + RBAC por endpoint
- [x] Columnas médicas en tabla `students`: diseases, allergies, emergency_contact_*

## ✅ SIS — Carga masiva CSV/SIGE
- [x] `POST /api/students/import` — acepta CSV en body JSON + mapeo de columnas opcional
- [x] Validación por fila: RUT, campos obligatorios, duplicados
- [x] Procesamiento transaccional: commit si todo ok, errores por fila sin rollback total
- [x] Reporte detallado: total, importados, omitidos (duplicados), errores con fila y causa
- [x] Columnas configurables vía `columnas` en payload (mapeo nombre_columna_csv → nombre_campo)
- [x] Soporta formato SIGE y columnas personalizadas

## 📋 .env actualizado
- [x] `.env.example` completo con TODAS las variables de todos los servicios (incluye JWT_SECRET, URLs entre servicios, puertos por servicio)
- [x] `docker-compose.yml` actualizado con `JWT_SECRET` en SIS service

## ✅ Frontend Shell (estilo iSAMS)

### Sidebar colapsable (navegación lateral)
- [x] 10 íconos SVG (Lucide-style) para cada sección
- [x] Secciones con labels: Principal, Gestión, Sistema
- [x] Colapso animado con toggle button (flechas SVG)
- [x] Enlaces a rutas (Dashboard → `/`, Alumnos → `#`, etc.)

### Búsqueda global Cmd+K
- [x] Atajo de teclado (Ctrl+K) — input con hint visual
- [x] Modal overlay con backdrop semi-transparente
- [x] Búsqueda async contra `GET /api/students?search=...`
- [x] Navegación con flechas (ArrowUp/Down) + Enter
- [x] Cerrar con ESC o click fuera
- [x] Resultados con avatar inicial + nombre + curso + RUT

### Mosaicos inteligentes (Dashboard tiles)
- [x] Mosaico Asistencia: KPI grid (total, presentes, ausentes, atrasos) + barra porcentual
- [x] Mosaico Alertas: lista de alumnos en riesgo con badge de severidad
- [x] Mosaico Agenda: próximos eventos con badge de fecha y tipo

### Paleta de colores iSAMS
- [x] Primario: #1A2B3C (Azul Medianoche) — sidebar y acentos
- [x] Fondo: #F8F9FA (Gris Neutro) — fondo general
- [x] Acentos: Verde Éxito (#16a34a) y Ámbar Alerta (#f59e0b)
- [x] Sidebar activo: #243B4F, texto: #94A3B8

## ✅ Academic Service — Completado
- [x] Configuración de periodos académicos y asignaturas (CRUD subjects + academic_periods)
- [x] Lógica de ponderaciones y evaluación (Decreto 67) con grade_categories + weighted averages
- [x] CRUD calificaciones + bulk entry + weighted averages por categoría
- [x] Cálculo de promedios y promoción (semestral, anual, Decreto 67 rules)
- [x] JWT auth + RBAC en todos los endpoints
- [x] Reportes: estudiante (semestral/anual), curso (performance), promoción

## ✅ Attendance Service — Completado
- [x] CRUD asistencias + bulk entry con upsert (ON CONFLICT DO UPDATE)
- [x] Toma de asistencia por curso y fecha (formato Supereduc)
- [x] Reportes mensuales: global, por curso, exportación Supereduc
- [x] Resumen anual de asistencia por estudiante (mes a mes)
- [x] Alertas con umbrales: 85% general, 75% NEE (por severidad Alto/Medio/Bajo)
- [x] JWT auth + RBAC en todos los endpoints

## ✅ Frontend Dashboard — Completado
- [x] Rutas reales: Dashboard `/`, Alumnos `/students`, Asistencia `/attendance`, Calificaciones `/grades`
- [x] Sidebar con hrefs reales y detección de ruta activa
- [x] Página Alumnos: listado con búsqueda, RUT, curso, sección, estado
- [x] Página Asistencia: reporte mensual con filtros, summary cards, tabla con porcentajes
- [x] Página Calificaciones: selector de asignatura con datos del backend
- [x] Widget Rendimiento Académico en Dashboard (KPI grid + barra de aprobación)
- [x] JWT auth en API client (login, token en localStorage, headers automáticos)
- [x] Nuevos estilos CSS: data-table, summary-cards, page-header, toolbar, filters, badges
- [x] Vista 360° Alumno: perfil consolidado (info personal, notas S1/S2, promoción, entrevistas, finanzas)

## ✅ Etapa 3 — Completada

### Communication Engine
- [x] Notifications Service con JWT auth + DB (mensajes, bitácora entrevistas)
- [x] REST endpoints: mensajes (CRUD + mark read + contador), entrevistas (CRUD + por estudiante)
- [x] WebSocket hub con broadcast de mensajes en tiempo real
- [x] Gateway: ruta `/api/communications/*` + `/api/finance/*`
- [x] Frontend: página Notificaciones con bandeja de mensajes + badge en vivo en topbar

### Finance Service
- [x] Nuevo microservicio `schoolcbb-finance` (puerto 3006)
- [x] Schema: fees, payments, scholarships
- [x] CRUD cuotas (fees) + pagos con actualización automática de estado
- [x] CRUD becas con aprobación por usuario
- [x] JWT auth + RBAC en todos los endpoints
- [x] Integración en Dockerfile + docker-compose + gateway

### Vista 360° del Alumno
- [x] Página de detalle con información consolidada
- [x] Datos personales + estado de promoción
- [x] Calificaciones Semestre 1 y Semestre 2
- [x] Bitácora de entrevistas
- [x] Situación financiera (cuotas + deuda)
- [x] Navegación desde lista de alumnos (Link por alumno)

## ✅ Etapa 4 — Completada

### Reporting Service (nuevo, puerto 3007)
- [x] Certificado de Alumno Regular — datos personales, curso, año, validez, emisor
- [x] Concentración de Notas — historial completo por semestre (asignatura, notas, promedios, min/max)
- [x] Acta Final — acta de curso con todos los alumnos, notas por asignatura S1/S2, promedio final, promoción
- [x] Módulo SIGE — exportación de alumnos (RUT, nombres, curso, códigos MINEDUC)
- [x] Módulo SIGE — exportación de asistencia mensual (formato subvenciones)
- [x] JWT auth + RBAC en todos los endpoints
- [x] Gateway route `/api/reports/*` + docker-compose + Dockerfile

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
