# Estado del Proyecto — SchoolCBB v2

---

## ✅ Etapa 1-6 — Completadas (sesiones anteriores)

Identity, SIS, Academic, Attendance, Notifications, Finance, Reporting, Module Manager, Configuración Académica, CRM Admisión.

---

## 🆕 Sesión Actual — Mejoras de Arquitectura y Nuevos Módulos

### Workflow Engine (Event-Driven)

- [x] `CrmEvent` enum (StageChanged, DocumentUploaded, DocumentVerified)
- [x] `WorkflowEngine` con reglas configurables y procesamiento asíncrono via `tokio::spawn`
- [x] Reglas: "Aceptado" → notifica finanzas, "Matriculado" → crea alumno en SIS
- [x] Subida de documento → crea tarea de verificación automática
- [x] Tabla `event_log` para auditoría de eventos

### Dashboard de Métricas de Admisión

- [x] `GET /api/admission/metrics` — KPIs: total postulantes, matriculados, tasa de conversión, actividades (7d)
- [x] Distribución por etapa (barras) y por origen
- [x] Widget `AdmissionMetricsWidget` integrado en página de admisiones

### Business Process Flow

- [x] Componente `BusinessProcessFlow` — barra de progreso visual de etapas (completada/actual/pendiente)
- [x] Integrado en modal de detalle de postulante

### gRPC Inter-Service Communication

- [x] Proto `workflow.proto` con servicio `WorkflowEvents` (RPC `NotifyEvent`)
- [x] Servidor gRPC en Finance (puerto 4006) para recibir eventos
- [x] Cliente gRPC en SIS Workflow Engine para notificar a Finance
- [x] Configurable via `FINANCE_GRPC_URL`

### Pasarela de Pago (Webpay/Mock)

- [x] `PaymentGateway` trait con implementaciones `MockGateway` y `WebpayGateway`
- [x] Endpoints: `GET /api/finance/payment/init/{fee_id}`, `GET /api/finance/payment/return`
- [x] Tabla `payment_transactions` para tracking de transacciones
- [x] Botón "Pagar Online" en frontend de cuotas

### Campos Personalizados en CRM

- [x] Tablas `custom_field_definitions` y `custom_field_values`
- [x] CRUD de definiciones y valores por entidad (prospect)
- [x] Soporta tipos text y select con opciones configurables
- [x] Sección "Campos Personalizados" en modal de postulante

### Event Bus (Pub/Sub)

- [x] `event_bus` module en common crate con `BroadcastBus` (tokio::sync::broadcast)
- [x] `EventBus` trait intercambiable por NATS/Redis en producción
- [x] Integrado en Workflow Engine — publica eventos al bus
- [x] Suscriptor en main.rs que persiste eventos en `event_log`

### Roles y Permisos (Granular RBAC)

- [x] Tablas: `roles`, `permission_definitions`, `role_permissions`, `user_roles`
- [x] `Module` enum con 10 módulos y ~35 recursos
- [x] Seed automático de 8 roles del sistema y todas las definiciones de permisos
- [x] CRUD roles + asignación de permisos CRUD por recurso
- [x] Asignación de roles a usuarios desde página de usuarios
- [x] Tree UI con checkboxes (Leer/Crear/Actualizar/Eliminar) y guardado masivo
- [x] "Crear" auto-marca "Leer"
- [x] `GET /api/auth/my-permissions` — permisos del usuario actual
- [x] Control de acceso: solo usuarios con permiso de escritura en módulo Users pueden asignar roles
- [x] Endpoints de roles/permissions en identity service
- [x] Módulo "Roles y Permisos" en Module Manager (categoría Sistema)
- [x] Permisos vía API para Sostenedor (full access)

### Correcciones de Infraestructura

- [x] Migración de rutas `{param}` → `:param` en todos los servicios (axum 0.7.9)
- [x] Rutas base agregadas en gateway (sin `/*path`) para evitar 404
- [x] `ALTER TABLE` para columnas faltantes (`diseases`, `allergies`, `emergency_contact_*`, `classroom_id`, `plan`)
- [x] Fix de compilación en `business_process_flow.rs` (move after borrow)

### Corporaciones y Multi-colegio

- [x] Tablas: `corporations`, `schools`
- [x] `school_id` agregado a: `users`, `students`, `courses`, `enrollments`, `fees`, `scholarships`
- [x] Tipos `Corporation`, `School` en common crate
- [x] CRUD corporaciones y colegios en identity service
- [x] `corporation_id`/`school_id` en JWT Claims
- [x] Seed de corporación/colegio default
- [x] Filtrado por `school_id` en SIS, Academic, Attendance, Finance, Reporting
- [x] Página `/corporations` con tabla expandible y formularios
- [x] Columna "Colegio" en tabla de usuarios
- [x] Selector de colegio en panel de usuario
- [x] Módulo "Corporaciones y Colegios" en Module Manager

---

## 🆕 Segunda Sesión — RRHH y Correcciones

### Correcciones de Modelo

- [x] Cursos ya no tienen `subject` (asignatura principal) — ahora son conjuntos de asignaturas vía `course_subjects`
- [x] `ALTER TABLE courses ALTER COLUMN subject DROP NOT NULL`
- [x] Course subjects: endpoints `GET/POST /api/grades/course-subjects`, `DELETE /api/grades/course-subjects/:id`
- [x] Frontend: columna asignatura eliminada, modal para gestionar asignaturas por curso

### Módulo RRHH — Mes 1: Ficha del Empleado y Contratos

- [x] Tablas: `employees` (con `category` para clasificar trabajadores), `employee_contracts`, `employee_documents`
- [x] Tipos `Employee`, `EmployeeContract`, `EmployeeDocument` en common crate
- [x] Backend (SIS service): CRUD empleados + contratos con validación Ley 40 Horas (≤40h semanales)
- [x] Categorías: Docente, Directivo, Administrativo, Asistente, Enfermería, Psicología, Psicopedagogía, Auxiliar, Otro
- [x] Gateway: rutas `/api/hr` proxy al SIS
- [x] Frontend: página `/hr` con formulario de creación, tabla, búsqueda, selector de categoría
- [x] Módulo "Recursos Humanos" en Module Manager (categoría Administración)

### Pendiente — Mes 2 (próxima sesión)

- [ ] Motor de asistencia laboral DT (ingesta API/CSV, validación jornada, geocerca)
- [ ] Gestión de vacaciones y ausencias con flujo de aprobación

### Pendiente — Mes 3 (próxima sesión)

- [ ] Módulo de Remuneraciones (cálculo líquido, AFP, ISAPRE, LRE)
- [ ] Exportador Previred

### Pendiente — Mes 4 (próxima sesión)

- [ ] Portal de auto-consulta del empleado (liquidaciones, certificados)
- [ ] Flujos de aprobación (vacaciones, licencias)

---

## 📐 Puertos

| Servicio | Puerto Interno | Puerto gRPC |
|---|---|---|
| Gateway | 3000 | — |
| Identity | 3001 | — |
| SIS | 3002 | — |
| Academic | 3003 | — |
| Attendance | 3004 | — |
| Notifications | 3005 | — |
| Finance | 3006 | 4006 |
| Reporting | 3007 | — |
| Frontend (nginx) | 8080 | — |
| PostgreSQL | 5432 | — |

---

## 🚀 Comandos rápidos

```bash
# Iniciar todo
docker compose up -d

# Reconstruir un servicio específico
docker compose build --no-cache identity

# Logs
docker compose logs -f gateway identity sis

# Probar login
curl -X POST http://localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@colegio.cl","password":"admin123"}'
```
