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

- [x] ~~Motor de asistencia laboral DT (ingesta API/CSV, validación jornada, geocerca)~~
- [x] ~~Gestión de vacaciones y ausencias con flujo de aprobación~~

#### Detalle de lo implementado (Mes 2):

##### Motor de Asistencia DT
- [x] Tabla `employee_attendance_logs` con marcaciones (timestamp, entry_type, device_id, location_hash, source)
- [x] Tabla `employee_attendance_modifications` (bitácora DT: valor original, nuevo, motivo, usuario)
- [x] Tipos `AttendanceLog`, `AttendanceModification`, `AttendanceLogPayload`, `AttendanceModificationPayload` en common crate
- [x] `POST /api/hr/attendance/sync` — ingesta de marcaciones vía API
- [x] `GET /api/hr/employees/:id/attendance` — listar marcaciones (con filtro por fecha)
- [x] `GET /api/hr/employees/:id/attendance/summary` — resumen mensual
- [x] `PUT /api/hr/attendance/:att_id/modify` — modificar marcación con bitácora de auditoría DT

##### Vacaciones y Ausencias
- [x] Tabla `leave_requests` con tipos (Vacaciones, Licencia Médica, Permiso Personal, Capacitación, Otro)
- [x] Tipos `LeaveRequest`, `CreateLeavePayload`, `LeaveApprovalPayload` en common crate
- [x] `POST /api/hr/employees/:id/leave-requests` — crear solicitud
- [x] `GET /api/hr/employees/:id/leave-requests` — listar solicitudes de un empleado
- [x] `GET /api/hr/leave-requests` — listar todas (con filtros)
- [x] `PUT /api/hr/leave-requests/:id/approve` — aprobar/rechazar con descuento automático de días de vacaciones

##### Frontend
- [x] Página `/hr/{employee_id}` con ficha del empleado
- [x] Tabs: Contratos, Asistencia, Vacaciones y Permisos
- [x] Tabla de contratos con histórico
- [x] Registro manual de marcaciones
- [x] Tabla de asistencia en tiempo real
- [x] Solicitud de vacaciones/permisos con formulario
- [x] Tabla de solicitudes con estado (Pendiente/Aprobado/Rechazado)

### Pendiente — Mes 3 (próxima sesión)

- [x] ~~Módulo de Remuneraciones (cálculo líquido, AFP, ISAPRE, LRE)~~
- [x] ~~Exportador Previred~~

#### Detalle de lo implementado (Mes 3):

##### Módulo de Remuneraciones
- [x] Tabla `payrolls` con todos los campos de liquidación (salary_base, gratificación, descuentos, líquido)
- [x] Tabla `employee_pension_funds` para AFP y sistema de salud por empleado
- [x] Tipos `Payroll`, `PayrollCalculation`, `PayrollLineItem`, `EmployeePensionFund`, `PensionFund` enum (7 AFP con comisiones reales)
- [x] Función `calculate_payroll()` con lógica chilena: gratificación 25% (tope $500K), AFP 10% + comisión variable, Salud 7%, Seguro Cesantía 0.6%
- [x] `POST /api/hr/payroll/calculate` — vista previa del cálculo
- [x] `POST /api/hr/payroll` — generar y persistir liquidación
- [x] `GET /api/hr/payroll` — listar liquidaciones por mes/año con datos del empleado
- [x] `GET /api/hr/payroll/:id` — detalle de liquidación
- [x] `GET /api/hr/employees/:id/payroll` — historial de un empleado
- [x] `POST /api/hr/employees/:id/pension-fund` — configurar AFP/salud del empleado
- [x] `GET /api/hr/employees/:id/pension-fund` — obtener datos AFP/salud

##### Exportador LRE (Libro de Remuneraciones Electrónico)
- [x] `GET /api/hr/payroll/export/lre` — CSV con formato DT: RUT, Nombre, Sueldo Base, Gratificación, Imponible, AFP, Salud, Seguro Cesantía, Líquido
- [x] Marca `lre_exported = true` al exportar

##### Exportador Previred
- [x] `GET /api/hr/payroll/export/previred` — CSV formato Previred
- [x] Marca `previred_exported = true` al exportar

##### Frontend
- [x] Nueva ruta `/payroll` con página completa de remuneraciones
- [x] Selector de mes/año con filtro
- [x] Vista previa del cálculo con detalle de conceptos
- [x] Generación de liquidación
- [x] Tabla de liquidaciones del periodo
- [x] Botones de exportación LRE y Previred

### Pendiente — Mes 4 (próxima sesión)

##### Vinculación Empleado ↔ Usuario
- [x] Campo `user_id` en tabla `employees` (FK → users) para vincular cuenta de usuario con perfil laboral
- [x] Campo `supervisor_id` en tabla `employees` (FK → employees) para cadena de aprobación
- [x] `POST /api/hr/employees/:id/link-user` — vincular empleado con usuario del sistema
- [x] Actualizadas todas las queries de empleados para incluir `user_id` y `supervisor_id`

##### Portal de Auto-consulta (/my-portal)
- [x] `GET /api/hr/me` — perfil del empleado (datos + contrato activo + supervisor)
- [x] `GET /api/hr/me/payroll` — liquidaciones del empleado (últimos 12 meses)
- [x] `GET /api/hr/me/attendance` — asistencia últimos 30 días
- [x] `GET /api/hr/me/leave-requests` — mis solicitudes de vacaciones/permisos
- [x] `POST /api/hr/me/leave-requests` — crear solicitud desde el portal
- [x] Ruta frontend `/my-portal` con tabs: Mi Perfil, Liquidaciones, Asistencia, Vacaciones

##### Flujos de Aprobación con Notificaciones
- [x] `POST /api/hr/leave-requests/:id/notify` — notificar al empleado resultado de su solicitud
- [x] Al crear solicitud desde auto-consulta: notificación automática al supervisor vía mensajería interna
- [x] Notificaciones integradas con tabla `messages` (WebSocket en tiempo real)

---

## 🆕 Tercera Sesión — Bulk Import, Inline Editing y Búsqueda Global

### Bulk Import CSV
- [x] `POST /api/students/import/csv` — importación masiva de alumnos con validación de RUT, detección de duplicados y reporte detallado
- [x] `POST /api/hr/employees/import/csv` — importación masiva de empleados
- [x] Soporte de headers en español/inglés (nombres/nombre, apellidos/apellido, etc.)
- [x] Rutas frontend `/import/students` y `/import/employees`
- [x] Componente `CsvImportPage` con área de CSV, preview de template, resultado con errores

### Inline Editing
- [x] Componente `InlineEdit` reutilizable (doble clic → input/select → guardado automático)
- [x] Integrado en columna `grade_level` y `section` de tabla de alumnos
- [x] Soporte para input text y select con opciones

### Búsqueda Global Cmd+K
- [x] `GET /api/search?q=...` — endpoint unificado que busca alumnos y empleados simultáneamente
- [x] Atajo global Cmd+K / Ctrl+K registrado en topbar
- [x] QuickSearch modal mejorado con resultados mixtos (students + employees)
- [x] Navegación por teclado (flechas + Enter) y clic
- [x] Badge visual Student/Employee por resultado
- [x] Ruteo inteligente: `/students/:id` o `/hr/:id`

### Pendientes para próxima sesión
- [ ] ~~**Ley Karin** — canal de denuncias anónimas con registro de medidas de resguardo~~ ✅
- [ ] ~~**Geocerca + Reporte PDF Asistencia** — marcaje con ubicación y exportación HTML para fiscalización DT~~ ✅
- [ ] ~~**CSV Mapper** — interfaz de mapeo de columnas (cubierto por Bulk Import CSV)~~ ✅
- [ ] ~~**Impuesto segunda categoría** — retención de renta en liquidaciones~~ ✅
- [ ] ~~**Días progresivos vacaciones** — cálculo según antigüedad~~ ✅

### Pendientes para próxima sesión
- [ ] ~~**Carga certificados + descarga liquidaciones** — self-service empleado~~ ✅
- [ ] ~~**SIGE Module** — exportación de datos a MINEDUC~~ ✅
- [ ] ~~**Kanban View** — arrastrar postulantes entre etapas de admisión~~ ✅
- [ ] ~~**Mosaicos Dashboard** — tiles de asistencia, alertas y agenda en tiempo real~~ ✅

---

## 🎯 Todos los pendientes completados

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
