# Estado del Proyecto — SchoolCBB v2

## Último commit: `a40f16d` — Etapa 6 completa (Configuración Académica + CRM Admisión)
## Sesión actual: Etapa 1-6 completadas

---

## ✅ Etapa 1-4 — Completadas (ver secciones anteriores)

### Identity Service — Auth completo (JWT + Argon2 + RBAC)
### SIS Service — CRUD Alumnos + CSV import + Parentescos + Cursos + Matrículas
### Academic Service — Decreto 67, ponderaciones, calificaciones, reportes
### Attendance Service — Asistencia diaria/mensual, alertas Supereduc, exportación SIGE
### Notifications Service — Mensajería WebSocket + bitácora entrevistas
### Finance Service — Cuotas, pagos, becas
### Reporting Service — Certificados, concentración, acta final, SIGE

---

## ✅ Etapa 5 — Module Manager (Completada)

- [x] Tiles con íconos SVG por módulo
- [x] Búsqueda y filtro por nombre
- [x] Favoritos con estrella (persistencia vía API)
- [x] Sidebar con secciones y favoritos dinámicos

---

## ✅ Etapa 6 — Configuración Académica y Admisión (Completada)

### Módulo Académico — Mejoras

- [x] `academic_years` — tabla + CRUD + frontend (`/academic-years`)
- [x] Clonación anual — `POST /api/academic-years/clone` (copia course_subjects)
- [x] `grade_levels` — tabla + seed con 22 niveles del CN + CRUD + frontend (`/grade-levels`)
- [x] `classrooms` — tabla + CRUD salas con aforo + frontend (`/classrooms`)
- [x] Audit Trail — tabla `audit_log`, helper en common, hooks en subjects CRUD
- [x] Frontend auditoría (`/audit`) con listado de cambios
- [x] Inline Editing — doble clic en código/nombre de asignatura
- [x] Bulk Import CSV — `POST /api/grades/subjects/import` + frontend paste area
- [x] Planes HC/TP/Artístico — columna `plan` en courses + seed de asignaturas por plan
- [x] Seed automático de ~28 asignaturas con horas semanales según CN

### Módulo Admisión (CRM)

- [x] `pipeline_stages` — etapas configurables con seed (6 etapas: Primer Contacto → Matriculado)
- [x] `prospects` — postulantes con RUT, email, teléfono, origen, notas
- [x] CRUD prospects + filtros (por etapa, búsqueda, asignado a)
- [x] Cambio de etapa — `PUT /api/admission/prospects/{id}/stage`
- [x] `prospect_activities` — timeline de actividades (llamada, email, tarea, nota)
- [x] `prospect_documents` — carga de documentos multipart a filesystem
- [x] Frontend Kanban — columnas por etapa, cards, modal de detalle con timeline
- [x] Vacancy Check — `GET /api/admission/vacancy-check` + frontend
- [x] Workflow Engine — al pasar a "Matriculado" crea alumno en SIS automáticamente
- [x] RBAC — rol `Admision` agregado a endpoints y UserRole enum

### Infraestructura

- [x] CI/CD — GitHub Actions (build backend + frontend, test, clippy)
- [x] SQL migrations — `sqlx::migrate!` con archivos versionados
- [x] Dockerfile optimizado — multi-stage con `cargo fetch` para cache de dependencias
- [x] `.dockerignore` — excluye .git, target, .agents, imágenes
- [x] GraphQL — URLs de servicios configurable via env vars
- [x] docker-compose — corregido: ATTENDANCE_URL, agregados FINANCE_URL, REPORTING_URL
- [x] Tests — 33 unit tests en common (attendance, student, finance, grades, rut)

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
| Finance | 3006 |
| Reporting | 3007 |
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
