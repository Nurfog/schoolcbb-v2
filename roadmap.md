# Roadmap — SchoolCBB v2

---

## ✅ Etapa 1-6 — Completadas

Identity, SIS, Academic, Attendance, Notifications, Finance, Reporting, Module Manager, Configuración Académica, CRM Admisión, Workflow Engine, Dashboard Métricas, gRPC, Pasarela de Pago, Campos Personalizados, Event Bus, RBAC, Corporaciones/Multi-colegio, RRHH Completo (Fichas, Asistencia DT, Vacaciones, Remuneraciones, LRE, Previred, Portal Auto-consulta), Bulk Import CSV, Búsqueda Global, SIGE, Ley Karin, Geocerca, Kanban Admisión, Mosaicos Dashboard.

---

## 🎯 Próximas Etapas — Plan de Implementación

---

### Etapa 7 — Arquitectura del Portal y Root Service

Crear nuevo servicio `packages/services/portal` (axum + server-side templates con `minijinja` o `tera`) para el portal público. Alternativamente usar Astro/Next.js como subdirectorio si se prefiere separar tecnología. La app Dioxus existente pasa a `app.schoolcbb.cl` y el portal vive en la raíz del dominio.

Rutas del portal:
| Ruta | Descripción |
|---|---|
| `/` | Hero / Index |
| `/features` | Características del software |
| `/pricing` | Planes y precios (dinámico desde DB) |
| `/about` | Sobre nosotros |
| `/contact` | Contacto |
| `/blog` | Blog / anuncios |

Nuevo Docker Compose service `portal` en puerto 3010, con nginx ruteando `/` al portal y `/app/` a la app Dioxus.

---

### Etapa 7.1 — Data Model: Licencias y Planes

```sql
CREATE TABLE license_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,           -- "Básico", "Profesional", "Corporativo"
    description TEXT,
    price_monthly DECIMAL(10,2) NOT NULL DEFAULT 0,
    price_yearly DECIMAL(10,2) NOT NULL DEFAULT 0,
    featured BOOLEAN NOT NULL DEFAULT false,   -- destacar en pricing table
    sort_order INT NOT NULL DEFAULT 0,
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE plan_modules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plan_id UUID NOT NULL REFERENCES license_plans(id) ON DELETE CASCADE,
    module_key VARCHAR(50) NOT NULL,      -- "academic", "attendance", "finance", "hr", "admission", "reports", "sige", "communications"
    module_name VARCHAR(100) NOT NULL,    -- nombre visible
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
    status VARCHAR(20) NOT NULL DEFAULT 'active',  -- active, expired, cancelled, suspended, trial
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE corporation_module_overrides (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corporation_id UUID NOT NULL REFERENCES corporations(id) ON DELETE CASCADE,
    module_key VARCHAR(50) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    reason TEXT,                            -- "extra_purchase", "promotion", "courtesy"
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE license_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    corporation_license_id UUID NOT NULL REFERENCES corporation_licenses(id),
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'CLP',
    payment_method VARCHAR(30) NOT NULL,    -- transfer, webpay, cash, invoice
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, completed, failed, refunded
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

Seed de planes iniciales: **Básico** (esencial: dashboard, alumnos, cursos, asistencia, notas), **Profesional** (+ RRHH, finanzas, admisión, reportes), **Corporativo** (todo + multi-colegio, SIGE, API).

---

### Etapa 7.2 — Structs en Common Crate

`packages/common/src/licensing.rs`:

```rust
pub struct LicensePlan {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_monthly: f64,
    pub price_yearly: f64,
    pub featured: bool,
    pub sort_order: i32,
    pub active: bool,
    pub created_at: DateTime<Utc>,
}

pub struct PlanModule {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub module_key: String,
    pub module_name: String,
    pub included: bool,
}

pub struct CorporationLicense {
    pub id: Uuid,
    pub corporation_id: Uuid,
    pub plan_id: Uuid,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub auto_renew: bool,
    pub grace_period_days: i32,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct LicensePayment { ... }
pub struct LicenseExtension { ... }
pub struct LicenseSummary {
    pub corporation_name: String,
    pub plan_name: String,
    pub status: String,
    pub days_remaining: i64,
    pub total_schools: i64,
    pub total_students: i64,
    pub total_employees: i64,
}
```

---

### Etapa 8 — Root Dashboard (Rol Super Admin)

#### 8.1 Nuevo Rol `Root`

Agregar `Root` al enum `UserRole` en `common/src/user.rs`:
```rust
pub enum UserRole {
    Root,          // ← nuevo: super admin global
    Sostenedor,
    Director,
    UTP,
    Administrador,
    Profesor,
    Apoderado,
    Alumno,
    Admision,
}
```

Seeder: primer `Root` se crea desde variable de entorno `ROOT_EMAIL`/`ROOT_PASSWORD` en identity. Login normal, pero sin `corporation_id` asociado.

Endpoint `GET /api/auth/me` devuelve `is_root: true` para que el frontend muestre el panel root en lugar del dashboard normal.

Middleware `require_root()` que verifica `claims.role == "Root"`.

#### 8.2 Endpoints Root (identity service)

| Método | Ruta | Descripción |
|---|---|---|
| `GET` | `/api/admin/stats/summary` | KPIs globales (totales) |
| `GET` | `/api/admin/stats/monthly` | Evolución mensual (registros, ingresos) |
| `GET` | `/api/admin/stats/license-distribution` | Distribución de planes |
| `GET` | `/api/admin/corporations` | Listar todas con licencia y métricas |
| `POST` | `/api/admin/corporations` | Crear corporación + asignar licencia |
| `PUT` | `/api/admin/corporations/{id}/toggle` | Activar/desactivar |
| `GET` | `/api/admin/license-plans` | Listar planes |
| `POST` | `/api/admin/license-plans` | Crear plan |
| `PUT` | `/api/admin/license-plans/{id}` | Editar plan |
| `DELETE` | `/api/admin/license-plans/{id}` | Eliminar plan |
| `POST` | `/api/admin/license-plans/{id}/modules` | Set módulos del plan |
| `GET` | `/api/admin/licenses` | Todas las licencias con filtros |
| `POST` | `/api/admin/licenses` | Asignar licencia a corporación |
| `PUT` | `/api/admin/licenses/{id}/extend` | Prorrogar licencia (sumar días) |
| `PUT` | `/api/admin/licenses/{id}/change-plan` | Cambiar plan |
| `PUT` | `/api/admin/licenses/{id}/status` | Cambiar estado |
| `GET` | `/api/admin/payments` | Historial de pagos |
| `POST` | `/api/admin/payments` | Registrar pago manual |
| `GET` | `/api/admin/activity-log` | Log de acciones de admin |
| `GET` | `/api/admin/system/health` | Health check de todos los servicios |

#### 8.3 Root Dashboard Frontend

Nueva ruta `/root` en Dioxus con layout propio (sidebar root distinta). Componentes/widgets:

**Calugas (KPIs):**
- Total corporaciones (activas/inactivas)
- Total colegios
- Total alumnos (suma cruzando todas las corporaciones)
- Total empleados
- Licencias activas / próximas a vencer (&lt;30 días)
- Ingresos del mes / total histórico

**Gráficos (SVG puro o Chart.js via WASM interop):**
- Ingresos mensuales (barras, últimos 12 meses)
- Distribución de planes (donut/pie)
- Crecimiento de corporaciones (línea, últimos 12 meses)
- Top 10 corporaciones por cantidad de alumnos (barras horizontales)

**Tablas:**
- Corporaciones con licencia, plan, fecha vencimiento, acciones
- Pagos recientes
- Activity log

**Acciones:**
- Crear corporación + licencia en wizard
- Prorrogar licencia (modal: días, motivo)
- Cambiar plan de corporación
- Registrar pago manual
- Editar planes (modal CRUD con editor de módulos checkboxes)

#### 8.4 Estrategia de Gráficos en Dioxus

Opción recomendada: **Chart.js** via `wasm-bindgen` interop (inyectar `<canvas>` y llamar JS). Ya existe `js_sys` como dependencia. Alternativa: implementar SVG `<svg>` nativo con datos vinculados — más simple y sin dependencias externas.

Para el roadmap inicial, SVG directo:
- `<BarChart>` — barras responsivas con ejes
- `<DoughnutChart>` — donut con leyenda
- `<LineChart>` — línea de evolución

---

### Etapa 9 — Portal Público Dinámico

#### 9.1 Stack Recomendado

Nuevo servicio `packages/services/portal` usando **axum + minijinja** (templates server-side). Ya existe handlebars como dependencia; minijinja es más liviano y familiar (sintaxis Jinja2/Django).

Alternativa: **Astro** (JS) si se prefiere mejor DX para landing pages. Pero mantener Rust unifica el stack.

Hosting:
- `schoolcbb.cl` → portal (nginx → portal:3010)
- `schoolcbb.cl/app/` → app Dioxus (nginx → gateway:3000 → frontend assets)

O dominio separado: `schoolcbb.cl` (portal), `app.schoolcbb.cl` (app).

#### 9.2 Templates del Portal

```
packages/services/portal/templates/
  base.html              — layout base (nav, footer)
  index.html             — hero + features summary + CTA
  features.html          — grid de características
  pricing.html           — tabla comparativa dinámica
  about.html             — sobre nosotros
  contact.html           — formulario de contacto
  blog/
    index.html           — listado de posts
    post.html            — post individual
```

#### 9.3 Pricing Dinámico

`/pricing` renderiza desde DB:
1. `GET /api/public/plans` — lista planes activos con sus módulos
2. Template itera planes y genera tabla comparativa:

| Característica | Básico | Profesional | Corporativo |
|---|---|---|---|
| Dashboard | ✅ | ✅ | ✅ |
| Alumnos / Cursos | ✅ | ✅ | ✅ |
| Asistencia | ✅ | ✅ | ✅ |
| Notas / Académico | ✅ | ✅ | ✅ |
| RRHH | — | ✅ | ✅ |
| Finanzas | — | ✅ | ✅ |
| Admisión CRM | — | ✅ | ✅ |
| SIGE / MINEDUC | — | — | ✅ |
| Multi-colegio | — | — | ✅ |
| **Precio mensual** | $X | $Y | $Z |
| **Precio anual** | $X*12 | $Y*12 | $Z*12 |

Cada fila de característica se genera dinámicamente iterando `plan_modules`. Si un módulo no está en ningún plan, no aparece. Si un plan no tiene un módulo, muestra "—".

#### 9.4 Endpoints Públicos

Agregados al gateway (sin auth):
| Método | Ruta | Descripción |
|---|---|---|
| `GET` | `/api/public/plans` | Planes activos con módulos |
| `GET` | `/api/public/features` | Lista de características disponibles |
| `POST` | `/api/public/contact` | Formulario de contacto (→ email interno) |

---

### Etapa 10 — Portal Cliente (Corporación Login)

Ruta `/portal` dentro de la app Dioxus existente, o subdominio separado.

Funcionalidades:
- Dashboard de licencia: plan actual, fecha vencimiento, días restantes
- Ver módulos activos
- Historial de pagos / descargar comprobantes
- Botón "Solicitar upgrade" (notifica a root)
- Datos de facturación (RUT, dirección, giro)

Endpoint nuevo:
| Método | Ruta | Descripción |
|---|---|---|
| `GET` | `/api/client/license` | Licencia de mi corporación |
| `GET` | `/api/client/payments` | Mis pagos |
| `GET` | `/api/client/billing-info` | Datos de facturación |
| `PUT` | `/api/client/billing-info` | Actualizar datos |

---

### Etapa 11 — Prórroga, Pagos y Automatizaciones

#### 11.1 Prórroga de Licencias

Desde root dashboard:
- Modal "Prorrogar licencia": input de días + motivo
- Backend: `PUT /api/admin/licenses/{id}/extend`
  - Inserta en `license_extensions`
  - Actualiza `end_date` sumando días
  - Log en activity log

Programar tarea periódica (cron/tokio):
- Diario: detectar licencias próximas a vencer (30/15/7 días)
- Enviar notificación al root y a la corporación
- Al vencer: cambiar status a "expired", suspender acceso a módulos no-básicos

#### 11.2 Administración de Pagos

Registro manual desde root:
- Modal "Registrar pago": monto, método, período, subir comprobante
- Backend persiste en `license_payments`
- Genera ID de transacción único
- Opción de enviar comprobante por email a la corporación

Sugerencia futura: integrar con Webpay existente para pagos automáticos.

#### 11.3 Activity Log

Tabla `admin_activity_log`:
```
id, admin_id, action, entity_type, entity_id, details, created_at
```

Cada acción root (crear corporación, asignar licencia, prorrogar, etc.) se loguea. Visible en el root dashboard con filtros.

---

### Etapa 12 — Sistema de Notificaciones (Cross-Cutting)

- [ ] Email service: notificar a corporaciones sobre vencimiento de licencia
- [ ] Notificación in-app (WebSocket existente) para alertas del sistema
- [ ] Template de emails (bienvenida, renovación, pago recibido, suspensión)

---

## 📐 Puertos (Actualizado)

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
| Portal (nuevo) | 3010 |
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
docker compose logs -f gateway identity sis

# Probar login
curl -X POST http://localhost:3000/api/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@colegio.cl","password":"admin123"}'
```
