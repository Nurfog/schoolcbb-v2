# Plan de Desarrollo — SchoolCBB v2

> Plataforma escolar integral multi-tenant construida en Rust (axum + Dioxus WASM).

---

## Etapas 7-12 — Modelo de Licencias, Root Dashboard, Portal Público y Cliente

---

### Etapa 7 — Arquitectura del Portal y Root Service

**Objetivo:** Establecer la base para el modelo de licencias y la separación entre portal público (server-side) y app Dioxus.

**Archivos nuevos:**
- `packages/common/src/licensing.rs` — structs de datos (LicensePlan, PlanModule, CorporationLicense, LicensePayment, LicenseExtension, LicenseSummary)
- `packages/services/portal/` — nuevo servicio axum + minijinja para portal público
- `packages/services/portal/templates/` — templates HTML server-side
- `packages/services/portal/src/main.rs`
- `packages/services/portal/src/config.rs`
- `packages/services/portal/src/routes.rs`
- `packages/services/portal/src/models.rs`

**Archivos a modificar:**
- `packages/common/src/lib.rs` — exportar módulo `licensing`
- `packages/common/src/db_schema.rs` — agregar tablas de licencias
- `packages/common/src/user.rs` — agregar rol `Root`
- `docker-compose.yml` — agregar service `portal`
- `nginx.conf` — ruteo condicional según dominio

**DDL:**

```sql
CREATE TABLE license_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    price_monthly DECIMAL(10,2) NOT NULL DEFAULT 0,
    price_yearly DECIMAL(10,2) NOT NULL DEFAULT 0,
    featured BOOLEAN NOT NULL DEFAULT false,
    sort_order INT NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE plan_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES license_plans(id) ON DELETE CASCADE,
    module_key VARCHAR(50) NOT NULL,
    module_name VARCHAR(100) NOT NULL,
    included BOOLEAN NOT NULL DEFAULT true
);

CREATE TABLE corporation_licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corporation_id UUID NOT NULL REFERENCES corporations(id) ON DELETE CASCADE,
    plan_id UUID NOT NULL REFERENCES license_plans(id),
    start_date DATE NOT NULL,
    end_date DATE,
    auto_renew BOOLEAN NOT NULL DEFAULT false,
    grace_period_days INT NOT NULL DEFAULT 30,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE corporation_module_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corporation_id UUID NOT NULL REFERENCES corporations(id) ON DELETE CASCADE,
    module_key VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE license_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corporation_license_id UUID NOT NULL REFERENCES corporation_licenses(id),
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'CLP',
    payment_method VARCHAR(30) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    transaction_id VARCHAR(100),
    paid_at TIMESTAMPTZ,
    period_start DATE,
    period_end DATE,
    receipt_url TEXT,
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE license_extensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corporation_license_id UUID NOT NULL REFERENCES corporation_licenses(id) ON DELETE CASCADE,
    days_extended INT NOT NULL,
    reason VARCHAR(255) NOT NULL,
    approved_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

**Seed inicial de planes:**

| Plan | Módulos incluidos |
|---|---|
| Básico | Dashboard, Alumnos, Cursos, Asistencia, Notas/Académico |
| Profesional | Básico + RRHH, Finanzas, Admisión CRM, Reportes |
| Corporativo | Profesional + SIGE, Multi-colegio, API, Módulo LRE, Ley Karin |

---

### Etapa 8 — Root Dashboard (Super Admin)

**Objetivo:** Panel administrativo global para gestionar corporaciones, licencias, pagos y monitorear la plataforma.

**Archivos nuevos:**
- `packages/frontend/src/components/pages/root_page.rs` — página principal del dashboard root
- `packages/frontend/src/components/pages/root_corporations.rs` — gestión de corporaciones
- `packages/frontend/src/components/pages/root_plans.rs` — CRUD de planes/licencias
- `packages/frontend/src/components/pages/root_payments.rs` — gestión de pagos
- `packages/frontend/src/components/pages/root_activity.rs` — activity log
- `packages/frontend/src/components/widgets/kpi_card.rs` — caluga reutilizable
- `packages/frontend/src/components/widgets/simple_chart.rs` — gráficos SVG básicos
- `packages/services/identity/src/admin.rs` — endpoints del root dashboard

**Rol `Root`:**
- Nuevo variant en `UserRole` enum
- Sin `corporation_id` asociado (acceso global)
- Se crea desde `ROOT_EMAIL`/`ROOT_PASSWORD` en `.env`
- Endpoint `GET /api/auth/me` devuelve `is_root: true`

**Endpoints del admin (identity service):**

| Método | Ruta | Propósito |
|---|---|---|
| `GET` | `/api/admin/stats/summary` | KPIs globales |
| `GET` | `/api/admin/stats/monthly` | Evolución mensual |
| `GET` | `/api/admin/stats/license-distribution` | Distribución de planes |
| `GET` | `/api/admin/corporations` | Listar corporaciones con métricas |
| `POST` | `/api/admin/corporations` | Crear corporación + licencia |
| `PUT` | `/api/admin/corporations/{id}/toggle` | Activar/desactivar |
| `GET` `/POST` `/PUT` `/DELETE` | `/api/admin/license-plans[/{id}]` | CRUD planes |
| `POST` | `/api/admin/license-plans/{id}/modules` | Set módulos del plan |
| `GET` `/POST` | `/api/admin/licenses[/{id}]` | Asignar/listar licencias |
| `PUT` | `/api/admin/licenses/{id}/extend` | Prorrogar (sumar días) |
| `PUT` | `/api/admin/licenses/{id}/change-plan` | Cambiar plan |
| `PUT` | `/api/admin/licenses/{id}/status` | Cambiar estado |
| `GET` `/POST` | `/api/admin/payments` | Historial / registrar pago |
| `GET` | `/api/admin/activity-log` | Log de acciones |
| `GET` | `/api/admin/system/health` | Health check servicios |

**Widgets del dashboard root:**

```
┌─────────────────────────────────────────────────────┐
│  [Corporaciones]  [Colegios]  [Alumnos]  [Empleados] │  ← calugas KPI
│     12 activas       34         8,450       1,230    │
├─────────────────────────────────────────────────────┤
│  [Licencias Activas]  [Por Vencer]  [Ingresos Mes]  │
│        9                  3            $2,450,000    │
├──────────────────┬──────────────────────────────────┤
│  Ingresos 12m    │  Distribución de Planes           │
│  ████████░░      │  ██████░░ Básico                  │
│  ███████░░░      │  ███░░░░░ Profesional             │
│  ███████░░░      │  ██░░░░░░ Corporativo             │
├──────────────────┴──────────────────────────────────┤
│  Corporaciones              │ Acciones                │
│  Colegio Los Alpes   Básico │ [Prorrogar] [Editar]   │
│  Colegio Santa María  Pro   │ [Prorrogar] [Editar]   │
└─────────────────────────────────────────────────────┘
```

**Gráficos:** SVG nativo sin dependencias externas:
- `<KpiCard>` — caluga con icono, valor, label, color
- `<BarChart>` — barras verticales/horizontales
- `<DoughnutChart>` — donut con SVG `stroke-dasharray`
- `<LineChart>` — línea poligonal con SVG `<polyline>`

---

### Etapa 9 — Portal Público Dinámico

**Objetivo:** Landing page con server-side rendering, SEO, y tabla comparativa de planes generada desde la base de datos.

**Stack:** Nuevo servicio `packages/services/portal` con axum + minijinja.

**Rutas del portal:**

| Ruta | Template | Descripción |
|---|---|---|
| `/` | `index.html` | Hero + features summary + CTA |
| `/features` | `features.html` | Grid detallado de características |
| `/pricing` | `pricing.html` | Tabla comparativa dinámica desde DB |
| `/about` | `about.html` | Sobre nosotros |
| `/contact` | `contact.html` | Formulario de contacto |
| `/blog` | `blog/index.html` | Listado de posts (opcional) |
| `/blog/{slug}` | `blog/post.html` | Post individual (opcional) |

**Estructura de templates:**
```
packages/services/portal/templates/
  base.html              — layout base con nav y footer
  index.html
  features.html
  pricing.html
  about.html
  contact.html
  blog/
    index.html
    post.html
packages/services/portal/static/
  css/style.css
  js/main.js
  images/
```

**Pricing dinámico:**
1. Endpoint público `GET /api/public/plans` devuelve planes activos + módulos
2. Template itera planes y genera tabla comparativa tipo:

```
                    ┌──────────┬────────────┬──────────────┐
                    │  Básico  │ Profesional │ Corporativo  │
├───────────────────┼──────────┼────────────┼──────────────┤
│ Dashboard         │    ✅    │     ✅     │      ✅      │
│ Alumnos / Cursos  │    ✅    │     ✅     │      ✅      │
│ Asistencia        │    ✅    │     ✅     │      ✅      │
│ Notas / Académico │    ✅    │     ✅     │      ✅      │
│ RRHH              │    —     │     ✅     │      ✅      │
│ Finanzas          │    —     │     ✅     │      ✅      │
│ Admisión CRM      │    —     │     ✅     │      ✅      │
│ SIGE / MINEDUC    │    —     │     —      │      ✅      │
│ Multi-colegio     │    —     │     —      │      ✅      │
├───────────────────┼──────────┼────────────┼──────────────┤
│ Precio mensual    │   $X     │    $Y      │     $Z       │
│ Precio anual      │   $X*12  │   $Y*12    │    $Z*12     │
└───────────────────┴──────────┴────────────┴──────────────┘
```

**Endpoints públicos (gateway, sin auth):**

| Método | Ruta | Descripción |
|---|---|---|
| `GET` | `/api/public/plans` | Planes activos con módulos |
| `GET` | `/api/public/features` | Características disponibles |
| `POST` | `/api/public/contact` | Formulario de contacto |

**Hosting:**
- `schoolccb.cl` → portal (nginx → portal:3010)
- `schoolccb.cl/app/` → app Dioxus (nginx → gateway:3000)
- O: `schoolccb.cl` + `app.schoolccb.cl`

---

### Etapa 10 — Portal Cliente (Corporación)

**Objetivo:** Autogestión de licencia para dueños de colegio (Sostenedor).

**Ruta:** `/portal` dentro de la app Dioxus o subdominio separado.

**Funcionalidades:**
- Dashboard de licencia: plan actual, fecha vencimiento, días restantes
- Módulos activos (checklist visual)
- Historial de pagos + descarga de comprobantes
- Botón "Solicitar upgrade de plan" → notifica al root
- Datos de facturación (RUT, dirección, giro, email)

**Endpoints nuevos (identity service):**

| Método | Ruta | Descripción |
|---|---|---|
| `GET` | `/api/client/license` | Licencia de mi corporación |
| `GET` | `/api/client/payments` | Mis pagos |
| `GET` `/PUT` | `/api/client/billing-info` | Datos de facturación |

**Componentes frontend:**
- `client_portal_page.rs` — layout del portal cliente
- `client_license_card.rs` — card de estado de licencia
- `client_payment_history.rs` — historial de pagos
- `client_billing_form.rs` — formulario de datos de facturación

---

### Etapa 11 — Prórroga, Pagos y Automatizaciones

**Objetivo:** Ciclo de vida completo de licencias con prórrogas, pagos y tareas programadas.

#### 11.1 Prórroga de Licencias

- Modal en root dashboard: input días + motivo + confirmación
- `PUT /api/admin/licenses/{id}/extend`:
  - Inserta en `license_extensions`
  - Actualiza `end_date` sumando días
  - Log en `admin_activity_log`
- Tarea programada (tokio cron):
  - Diario: detectar licencias próximas a vencer (30/15/7/1 días)
  - Notificar al root (in-app)
  - Al expirar: cambiar status a `expired`

#### 11.2 Administración de Pagos

- Modal "Registrar pago": monto, método (transferencia/webpay/efectivo), período, archivo comprobante
- `POST /api/admin/payments` persiste en `license_payments`
- Genera ID de transacción único legible (`PAY-{año}-{correlativo}`)
- Opción: enviar comprobante por email a la corporación
- Botón "Marcar como pagado" desde listado de licencias

#### 11.3 Activity Log (Auditoría)

- Tabla `admin_activity_log`:
  ```sql
  CREATE TABLE admin_activity_log (
      id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
      admin_id UUID NOT NULL REFERENCES users(id),
      action VARCHAR(50) NOT NULL,
      entity_type VARCHAR(50),
      entity_id UUID,
      details JSONB,
      created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
  );
  ```
- Toda acción root (crear, asignar, prorrogar, toggle, etc.) se loguea automáticamente
- Vista en root dashboard con filtros por acción, fecha, admin

---

### Etapa 12 — Sistema de Notificaciones Multi-canal

- **Email service**: notificar vencimientos, pagos recibidos, bienvenida
- **In-app**: usar WebSocket existente para alertas del sistema
- **Templates de email**: bienvenida, renovación, pago recibido, suspensión, upgrade
- **Scheduler**: tarea tokio diaria que verifica licencias y dispara notificaciones

---

## Estructura de Archivos Nueva (Resumen)

```
packages/
  common/src/
    licensing.rs              ← nuevo: structs de licencias
  services/
    portal/                   ← nuevo: servicio portal público
      src/main.rs
      src/config.rs
      src/routes.rs
      src/models.rs
      templates/              ← server-side HTML
      static/                 ← CSS, JS, imágenes
  frontend/src/
    components/
      pages/
        root_page.rs          ← nuevo: dashboard root
        root_corporations.rs  ← nuevo
        root_plans.rs         ← nuevo
        root_payments.rs      ← nuevo
        root_activity.rs      ← nuevo
        client_portal_page.rs ← nuevo
      widgets/
        kpi_card.rs           ← nuevo
        simple_chart.rs       ← nuevo
  identity/src/
    admin.rs                  ← nuevo: endpoints root
    admin_models.rs           ← nuevo: queries admin
```

## Puertos

| Servicio | Puerto |
|---|---|
| Gateway | 3000 |
| Identity | 3001 |
| SIS | 3002 |
| Academic | 3003 |
| Attendance | 3004 |
| Notifications | 3005 |
| Finance | 3006 |
| Reporting | 3007 |
| **Portal (nuevo)** | **3010** |
| Frontend (nginx) | 8080 |
| PostgreSQL | 5432 |

---

## ✅ Historial — Etapas 1-6 (Completadas)

### Sesión 1 — Arquitectura Inicial y Módulos Core

Identity, SIS, Academic, Attendance, Notifications, Finance, Reporting, Module Manager, Configuración Académica, CRM Admisión.

#### Workflow Engine (Event-Driven)
- [x] `CrmEvent` enum (StageChanged, DocumentUploaded, DocumentVerified)
- [x] `WorkflowEngine` con reglas configurables y procesamiento asíncrono via `tokio::spawn`
- [x] Reglas: "Aceptado" → notifica finanzas, "Matriculado" → crea alumno en SIS
- [x] Subida de documento → crea tarea de verificación automática
- [x] Tabla `event_log` para auditoría de eventos

#### Dashboard de Métricas de Admisión
- [x] `GET /api/admission/metrics` — KPIs: total postulantes, matriculados, tasa de conversión, actividades (7d)
- [x] Distribución por etapa (barras) y por origen
- [x] Widget `AdmissionMetricsWidget` integrado en página de admisiones

#### Business Process Flow
- [x] Componente `BusinessProcessFlow` — barra de progreso visual de etapas
- [x] Integrado en modal de detalle de postulante

#### gRPC Inter-Service Communication
- [x] Proto `workflow.proto` con servicio `WorkflowEvents` (RPC `NotifyEvent`)
- [x] Servidor gRPC en Finance (puerto 4006)
- [x] Cliente gRPC en SIS Workflow Engine
- [x] Configurable via `FINANCE_GRPC_URL`

#### Pasarela de Pago (Webpay/Mock)
- [x] `PaymentGateway` trait con `MockGateway` y `WebpayGateway`
- [x] Endpoints: `GET /api/finance/payment/init/{fee_id}`, `GET /api/finance/payment/return`
- [x] Tabla `payment_transactions`
- [x] Botón "Pagar Online" en frontend de cuotas

#### Campos Personalizados en CRM
- [x] Tablas `custom_field_definitions` y `custom_field_values`
- [x] CRUD de definiciones y valores por entidad (prospect)
- [x] Soporta tipos text y select con opciones

#### Event Bus (Pub/Sub)
- [x] `event_bus` module en common crate con `BroadcastBus`
- [x] `EventBus` trait intercambiable por NATS/Redis
- [x] Integrado en Workflow Engine + suscriptor que persiste en `event_log`

#### Roles y Permisos (Granular RBAC)
- [x] Tablas: `roles`, `permission_definitions`, `role_permissions`, `user_roles`
- [x] `Module` enum con 10 módulos y ~35 recursos
- [x] Seed automático de 8 roles del sistema
- [x] CRUD roles + asignación de permisos CRUD por recurso
- [x] Asignación de roles a usuarios
- [x] Tree UI con checkboxes (Leer/Crear/Actualizar/Eliminar)
- [x] Control de acceso por permiso de escritura

#### Corporaciones y Multi-colegio
- [x] Tablas: `corporations`, `schools`
- [x] `school_id` agregado a: `users`, `students`, `courses`, `enrollments`, `fees`, `scholarships`
- [x] Tipos `Corporation`, `School` en common crate
- [x] CRUD corporaciones y colegios en identity service
- [x] `corporation_id`/`school_id` en JWT Claims
- [x] Página `/corporations` con tabla expandible y formularios
- [x] Módulo "Corporaciones y Colegios" en Module Manager

### Sesión 2 — RRHH y Correcciones

#### Correcciones de Modelo
- [x] Cursos ya no tienen `subject` — ahora son conjuntos de asignaturas via `course_subjects`
- [x] `ALTER TABLE courses ALTER COLUMN subject DROP NOT NULL`
- [x] Course subjects: endpoints + frontend

#### Módulo RRHH — Mes 1: Ficha del Empleado y Contratos
- [x] Tablas: `employees`, `employee_contracts`, `employee_documents`
- [x] CRUD empleados + contratos con validación Ley 40 Horas
- [x] Categorías: Docente, Directivo, Administrativo, Asistente, Enfermería, Psicología, Psicopedagogía, Auxiliar, Otro
- [x] Gateway: rutas `/api/hr` proxy al SIS
- [x] Frontend: página `/hr` con formulario de creación, tabla, búsqueda
- [x] Módulo "Recursos Humanos" en Module Manager

#### Mes 2: Motor de Asistencia DT
- [x] Tabla `employee_attendance_logs` con marcaciones
- [x] Tabla `employee_attendance_modifications` (bitácora DT)
- [x] `POST /api/hr/attendance/sync` — ingesta de marcaciones
- [x] `GET /api/hr/employees/:id/attendance` — listar marcaciones
- [x] `GET /api/hr/employees/:id/attendance/summary` — resumen mensual
- [x] `PUT /api/hr/attendance/:att_id/modify` — modificar con bitácora

#### Mes 2: Vacaciones y Ausencias
- [x] Tabla `leave_requests` con tipos
- [x] `POST /api/hr/employees/:id/leave-requests` — crear solicitud
- [x] `GET /api/hr/employees/:id/leave-requests` — listar
- [x] `PUT /api/hr/leave-requests/:id/approve` — aprobar/rechazar con descuento de días

#### Mes 3: Módulo de Remuneraciones
- [x] Tabla `payrolls` con todos los campos de liquidación
- [x] Tabla `employee_pension_funds` para AFP y salud
- [x] Función `calculate_payroll()` con lógica chilena completa
- [x] Exportador LRE y Previred en CSV

#### Mes 4: Vinculación, Portal y Flujos
- [x] Campo `user_id` en employees
- [x] Campo `supervisor_id` en employees
- [x] `POST /api/hr/employees/:id/link-user`
- [x] Portal auto-consulta `/my-portal`
- [x] Flujos de aprobación con notificaciones
- [x] Frontend RRHH completo con tabs

### Sesión 3 — Bulk Import, Inline Editing y Búsqueda Global

- [x] Bulk Import CSV alumnos y empleados
- [x] Inline Editing reutilizable
- [x] Búsqueda global Cmd+K con QuickSearch
- [x] Ley Karin — canal de denuncias
- [x] Geocerca + validación de ubicación
- [x] SIGE Module — exportación MINEDUC
- [x] Kanban View en admisión
- [x] Mosaicos Dashboard
- [x] Correcciones de infraestructura y migración axum 0.8
