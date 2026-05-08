# Plan de Desarrollo: Core Escolar "Full Rust" (Inspiración iSAMS)

Este documento detalla la hoja de ruta técnica y funcional para la implementación de un sistema de gestión escolar (ERP) de alto rendimiento, diseñado específicamente para el mercado chileno utilizando un stack **Full Rust**.

---

## 🏗️ Arquitectura del Sistema
El proyecto se organiza como un **Cargo Workspace** para facilitar la compartición de código entre microservicios y el frontend.

### Stack Tecnológico
- **Backend:** Axum (Framework Web), SQLx (Persistencia SQL), PostgreSQL (DB).
- **Frontend:** Leptos / Dioxus (WebAssembly).
- **Comunicación:** gRPC para inter-servicios y REST/GraphQL para el cliente.
- **Seguridad:** Argon2 para hashing y JWT/Paseto para sesiones.

---

## 🗓️ Fases de Implementación

### Etapa 1: Cimientos e Identidad (Mes 1-2)
Foco en la infraestructura base y la seguridad de los datos.
- **Identity Service**: Gestión de usuarios, autenticación y RBAC (Roles: Sostenedor, Director, UTP, Profesor, Alumno, Apoderado).
- **SIS Service (Student Information System)**: El "Core" de datos. Manejo de fichas de alumnos, validación de RUT y gestión de parentescos.
- **Frontend Shell**: Creación del layout principal estilo iSAMS con barra lateral y búsqueda global.

### Etapa 2: Motor Académico y Asistencia (Mes 3-4)
Implementación de la lógica de negocio crítica para el funcionamiento del colegio.
- **Academic Service**: Configuración de periodos académicos, asignaturas y lógica del **Decreto 67** (ponderaciones y evaluaciones).
- **Attendance Service**: Microservicio optimizado para la toma de asistencia diaria y por bloque, cumpliendo con estándares de la Supereduc para subvenciones.
- **Dashboard de Mosaicos**: Primera versión del inicio reactivo con widgets informativos.

### Etapa 3: Comunicación y Finanzas (Mes 5-6)
Extensión del sistema hacia la comunidad escolar y la administración financiera.
- **Communication Engine**: Sistema de mensajería interna, notificaciones push y bitácora de entrevistas.
- **Finance Service**: Control de pagos, becas, mensualidades e integración con pasarelas de pago.
- **Vista 360° del Alumno**: Perfil consolidado que une datos académicos, conductuales y financieros.

### Etapa 4: Reportabilidad y Cumplimiento (Mes 7+)
Automatización de trámites legales y salida de datos.
- **Reporting Service**: Generación de certificados de alumno regular, concentraciones de notas y actas finales.
- **Módulo SIGE**: Herramientas de exportación de datos listas para la plataforma del MINEDUC.

---

## 🖥️ Guía de Diseño de Interfaz (UI/UX)
Para lograr la estética de **iSAMS**, el frontend debe seguir estas reglas:

1. **Navegación Lateral**: Menú colapsable con jerarquía clara.
2. **Búsqueda Global (Quick Search)**: Acceso rápido a perfiles de alumnos por nombre o RUT (Cmd+K).
3. **Mosaicos Inteligentes (Tiles)**:
   - **Mosaico Asistencia**: Gráfico en tiempo real del % de asistencia del día.
   - **Mosaico Alertas**: Notificaciones de alumnos en riesgo académico o inasistencia crítica.
   - **Mosaico Agenda**: Próximos hitos del calendario escolar.
4. **Paleta de Colores**:
   - `Primario`: Azul Medianoche (#1A2B3C)
   - `Fondo`: Gris Neutro Muy Claro (#F8F9FA)
   - `Acentos`: Verde Éxito y Ámbar Alerta.

---

## 📋 Matriz de Microservicios

| Servicio | Responsabilidad Crítica | Lógica Chilena Específica |
| :--- | :--- | :--- |
| **Identity** | Seguridad y Acceso | Privacidad de datos sensibles (Ley 19.628). |
| **SIS** | Datos Maestros | Validación RUT con dígito verificador. |
| **Academic** | Notas y Promedios | Cálculo automático Decreto 67. |
| **Attendance** | Registro de Presencia | Formato exportable para Subvenciones. |
| **Comm** | Avisos y Alertas | Bitácora oficial para la Supereduc. |

---

> **Nota Técnica**: Al ser un desarrollo **Full Rust**, se recomienda crear un crate `common` dentro del workspace para definir las estructuras (`structs`) de Alumno, Curso y Nota que serán serializadas por `serde` tanto en el servidor como en el cliente Wasm.

### ✅ Etapa 5: Module Manager (Completada)
### 🔄 Etapa 6: Configuración Académica y Admisión (En Progreso)

Foco en la parametrización del motor académico y el pipeline de matrícula.

#### 1. Módulo de Configuración Académica (Asignaturas y Cursos)
Estructura jerárquica para cumplir con el acta de notas final:
- **Niveles**: Pre-kinder a 4° Medio
- **Cursos**: Relacionados a un Nivel y un Profesor Jefe
- **Asignaturas**: Horas semanales, criterio de evaluación (Decreto 67), Código SIGE
- **Clonación**: Endpoint para clonar estructura académica del año anterior al actual

**Backend**: Axum + SQLx. CRUD de niveles, cursos, asignaturas + clonación anual.

#### 2. Módulo de Admisión (Pipeline CRM)
Pipeline de estados del postulante: `Interesado → Evaluación Pendiente → Aceptado → Matriculado`
- Formulario dinámico con carga de documentos (S3 compatible)
- Validación automática de vacantes por nivel
- El postulante no es alumno hasta que se matricula

**Backend**: Axum + PostgreSQL + almacenamiento S3.
**Frontend**: Pipeline visual en Dioxus con kanban-style.

#### 3. Estructura de Base de Datos
| Tabla | Función |
|-------|---------|
| `academic_years` | Año escolar activo (ej. 2026) |
| `subjects` | Catálogo maestro de asignaturas |
| `course_definitions` | Curso + profesor + año académico |
| `admissions_portal` | Postulantes con documentos y estado |

#### 4. Toque iSAMS en UI
- **Bulk Import**: Carga masiva vía CSV con validación en Wasm
- **Inline Editing**: Editar en fila (doble clic) sin abrir modal
- **Audit Trail**: Registro de quién y cuándo modificó cada asignatura

   ## usar la foto ManagerModulos.png como referencia para la UI

    "Actúa como un Desarrollador Senior de Frontend en Rust utilizando Dioxus y Tailwind CSS. Necesito crear el componente 'Module Manager' que aparece tras el inicio de sesión, inspirado en la estética de iSAMS.
    1. Estructura del Layout (Grid System):
        Sidebar Izquierdo: Un panel fijo de 280px con fondo oscuro. Debe incluir:
            Un 'Quick Search' superior para filtrar módulos.
            Sección de 'Favoritos' (que se actualiza dinámicamente).
            Lista de categorías (Administración, Académico, Comunicaciones, etc.) con recuento de módulos.
        Main Content: Una cuadrícula (CSS Grid) que muestre 'Tiles' (mosaicos) cuadrados.
    2. Diseño del 'Module Tile' (Mosaico):
        Cada mosaico debe tener un borde suave, fondo claro y un icono central representativo.
        Interactividad de Favoritos: Incluye un icono de estrella en la esquina superior derecha de cada mosaico. Al hacer clic (toggle), el módulo debe aparecer/desaparecer de la sección de 'Favoritos' de la Sidebar instantáneamente.
        Usa Signals (Leptos) o State (Dioxus) para manejar esta reactividad sin recargar la página.
    3. Módulos Específicos (Contexto Chile):
        Genera mosaicos para: 'Libro de Clases Digital', 'Gestión de Matrícula (SIS)', 'Configuración SIGE', 'Evaluación Decreto 67', 'Asistencia Real-time', y 'Centro de Mensajería'.
    4. Lógica de Backend (Rust/Axum):
        Define una estructura Module en Rust con campos: id, name, icon, category y is_favorite: bool.
        Explica cómo el frontend debe enviar una petición asíncrona al backend en Axum para persistir la preferencia del usuario cuando marque una estrella.
    5. Estética Visual:
        Usa una paleta de colores profesional: Azules profundos para la sidebar (#1a2b3c), blancos limpios para el área de trabajo y un dorado brillante para las estrellas de favoritos. La fuente debe ser 'Inter' o 'Urbanist'."


   ## Consideraciones Técnicas para tu Implementación

      Para que esta UI funcione realmente como iSAMS, debes considerar estos tres puntos clave en tu código Rust:
      1. El Crate common (Estructuras Compartidas)
      En tu workspace de Rust, crea un archivo donde definas el módulo. Esto asegura que el Backend y el Frontend hablen el mismo idioma:
      Rust
      #[derive(Serialize, Deserialize, Clone, Debug)]
      pub struct Module {
         pub id: String,
         pub name: String,
         pub icon: String,
         pub category: String,
         pub is_favorite: bool,
      }

      2. Persistencia de Configuración por Usuario

      En tu base de datos PostgreSQL, no guardes los favoritos en la tabla de módulos. Crea una tabla relacional:
         user_favorites: user_id | module_id
         Cuando el usuario presione la estrella, el frontend llama a un endpoint de Axum: POST /api/user/favorites/{module_id}.
         Crear la lógica del buscador global que filtra los módulos mientras escribes


#### etapa 6 CRM admision

🏗️ Arquitectura de Microservicios: CRM Educativo

Para mantener la independencia y escalabilidad, dividiremos el CRM en 3 servicios específicos que se conectarán a tu API Gateway.
1. admission-crm-service (El Motor)

    Función: Gestionar el ciclo de vida del postulante.

    Entidades: Lead (Interesado), Opportunity (Postulante con proceso iniciado), Pipeline (Etapas de admisión).

    Tecnología: Axum + SQLx (PostgreSQL).

2. interaction-service (Línea de Tiempo)

    Función: Registrar cada "Toque". Si un apoderado llama o envía un email, este servicio lo guarda.

    Entidades: Activity (Llamada, Email, Tarea, Nota).

    Sincronización: Usa NATS o Redis Pub/Sub para que, cuando el comm-engine envíe un correo, este servicio lo registre automáticamente.

3. workflow-engine (Automatización)

    Función: El "cerebro". Ejemplo: "Si el postulante pasa a estado 'Aceptado', enviar automáticamente el contrato y el cupón de pago".

    Lógica: Basado en eventos de Rust.

📅 Plan de Implementación por Etapas
Etapa 1: Definición de Entidades y Pipeline (Mes 1)

Inspirado en Dynamics, crearemos una base de datos relacional flexible.

    Pipeline de Admisión: Configurar etapas: Primer Contacto → Tour Escolar → Evaluación → Documentación → Matriculado.

    Campos Personalizados: Permitir que el colegio defina si necesita saber "Religión", "Colegio de procedencia" o "Alergias" desde la admisión.

    UI (Leptos/Dioxus): Vista de Kanban para arrastrar postulantes entre etapas.

Etapa 2: La Ficha 360° del Postulante (Mes 2)

Esta es la joya de la corona de Dynamics CRM.

    Panel Central: Datos maestros del alumno y apoderados.

    Línea de Tiempo (Timeline): Panel central que muestra en orden cronológico inverso todo lo que ha pasado con esa familia.

    Sub-grillas: Tablas integradas de hermanos (relaciones), documentos cargados y pagos pendientes.

Etapa 3: Gestión de Documentos y Flujos (Mes 3)

    Document Service: Integración con S3 para almacenar Certificados de Nacimiento, IPE/IPA, e informes de personalidad.

    Workflows: Creación de disparadores (Triggers). Si un postulante está "Estancado" en una etapa por más de 5 días, enviar alerta al equipo de Admisión.

🎨 UI/UX: Estilo Dynamics CRM en Rust

Para que se vea como Microsoft Dynamics, el frontend debe ser denso en información pero limpio.
Elementos Clave en el Frontend (Leptos/Dioxus):

    Business Process Flow: Una barra superior que indica en qué etapa del proceso de admisión está el alumno y qué pasos faltan para completarla.

    Tabbed Interface: Dentro de la ficha, usar pestañas para separar "Datos Personales", "Documentos", "Entrevistas" y "Finanzas".

    Side Panel de Resumen: Un panel derecho persistente que muestra los KPIs del postulante (ej: Probabilidad de cierre, Puntaje en examen de admisión).

🛠️ Prompt para la Implementación Técnica (Copia y Pega)

    "Actúa como un Arquitecto de Software experto en Microsoft Dynamics CRM y Full Rust. Diseña el microservicio de CRM de Admisión para un colegio chileno.

    Requerimientos Técnicos:

        Backend (Axum): Define un modelo de datos Prospect que herede campos de la entidad Alumno del Core, pero con estados de CRM (Lead, Opportunity, Account).

        Timeline Engine: Implementa un sistema de eventos donde cada microservicio (Comunicaciones, Finanzas, Académico) pueda enviar un 'ActivityLog' al CRM para mostrarlo en una línea de tiempo unificada.

        Frontend (Leptos/Dioxus): Diseña un componente de 'Business Process Flow' (barra de progreso de etapas) que sea reactivo y permita cambiar el estado del postulante mediante 'Signals'.

        Lógica de Negocio: Asegura que al marcar un postulante como 'Matriculado', se dispare un evento gRPC que cree automáticamente la ficha oficial en el módulo SIS (Student Information System) y el Libro de Clases, evitando la doble digitación.

    Estética: Utiliza el sistema de diseño de Microsoft Power Apps (Fluent UI) adaptado a Tailwind CSS, priorizando la densidad de datos y la navegación por pestañas."

🚀 Integración con el Core

El secreto de este CRM es que no es una isla.

    Admisión → SIS: Cuando el CRM cierra la "venta", el alumno nace en el Core.

    Admisión → Finanzas: El CRM genera el compromiso de pago de la matrícula.

    Admisión → Académico: El CRM asigna el cupo en el curso correspondiente.

Implementación del Workflow Engine en Rust

El motor de flujos se basa en un patrón de Máquina de Estados y Event-Driven Architecture. Cuando un Prospect cambia de estado, se dispara un evento que el motor procesa para ejecutar acciones automáticas.
Ejemplo de Estructura en Rust (Axum + Logic)
Rust

// common/crm_events.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CrmEvent {
    StageChanged {
        prospect_id: uuid::Uuid,
        from_stage: String,
        to_stage: String,
    },
    DocumentUploaded {
        prospect_id: uuid::Uuid,
        doc_type: String,
    },
}

// services/workflow_engine/mod.rs
pub async fn process_workflow(event: CrmEvent) {
    match event {
        CrmEvent::StageChanged { prospect_id, to_stage, .. } => {
            if to_stage == "ACEPTADO" {
                // Lógica: Notificar a Finanzas para generar cupón de pago
                send_notification_to_finance(prospect_id).await;
            }
            if to_stage == "MATRICULADO" {
                // Lógica: Promocionar Prospecto a Alumno oficial en el SIS
                promote_to_official_student(prospect_id).await;
            }
        }
        CrmEvent::DocumentUploaded { prospect_id, doc_type } => {
            // Lógica: Marcar tarea de verificación para el equipo de admisión
            create_verification_task(prospect_id, doc_type).await;
        }
    }
}

4. Etapas del Plan de Implementación
Fase 1: Core del CRM (Semana 1-4)

    Implementación del CRUD de Prospects.

    Creación de la tabla de Activities para la línea de tiempo (Timeline).

    API en Axum para gestionar el cambio de estados en el Pipeline.

Fase 2: Gestión Documental e Integración (Semana 5-8)

    Integración con S3/Minio para carga de certificados.

    Desarrollo del Workflow Engine inicial (Eventos internos en Rust).

    Sincronización vía gRPC entre CRM y el módulo de Finanzas.

Fase 3: UI Avanzada (Semana 9-12)

    Vista de Kanban en Leptos/Dioxus.

    Dashboard de métricas de admisión (Tasa de conversión, tiempo promedio en etapa).

    Buscador global optimizado para postulantes.

5. Consideraciones de Integración con el Core

    Single Source of Truth: Una vez que el estado es 'MATRICULADO', el registro maestro se mueve al servicio SIS-Service.

    RBAC: El equipo de admisión tiene permisos limitados al CRM, mientras que el equipo UTP solo ve alumnos matriculados.
    """

Plan Técnico: CRM de Admisión Escolar (Estilo Microsoft Dynamics)

Este documento detalla la arquitectura, el modelo de datos y el motor de flujos para el microservicio de CRM integrado al Core de Gestión Escolar.
1. Visión del Producto

El objetivo es transformar el proceso de admisión en un embudo de conversión (Pipeline) donde cada postulante es una "Oportunidad". Se busca centralizar la comunicación, documentos y estados de matrícula en una sola "Ficha 360°".
2. Diagrama de Entidad-Relación (ERD) - Mermaid
Fragmento de código

erDiagram
    PROSPECT ||--o{ ACTIVITY : "tiene"
    PROSPECT ||--o{ DOCUMENT : "sube"
    PROSPECT }|--|| PIPELINE_STAGE : "está en"
    PROSPECT ||--o{ FAMILY_MEMBER : "relacionado con"
    
    PROSPECT {
        uuid id
        string first_name
        string last_name
        string rut
        string email
        enum current_status
        uuid assigned_user_id
        timestamp created_at
    }

    PIPELINE_STAGE {
        uuid id
        string name
        int order
        bool is_final
    }

    ACTIVITY {
        uuid id
        enum type
        string subject
        text description
        timestamp scheduled_at
        bool is_completed
    }

    DOCUMENT {
        uuid id
        string file_name
        string s3_url
        enum doc_type
        bool is_verified
    }

3. Implementación del Workflow Engine en Rust

El motor de flujos se basa en un patrón de Máquina de Estados y Event-Driven Architecture. Cuando un Prospect cambia de estado, se dispara un evento que el motor procesa para ejecutar acciones automáticas.
Ejemplo de Estructura en Rust (Axum + Logic)
Rust

// common/crm_events.rs
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CrmEvent {
    StageChanged {
        prospect_id: uuid::Uuid,
        from_stage: String,
        to_stage: String,
    },
    DocumentUploaded {
        prospect_id: uuid::Uuid,
        doc_type: String,
    },
}

// services/workflow_engine/mod.rs
pub async fn process_workflow(event: CrmEvent) {
    match event {
        CrmEvent::StageChanged { prospect_id, to_stage, .. } => {
            if to_stage == "ACEPTADO" {
                // Lógica: Notificar a Finanzas para generar cupón de pago
                send_notification_to_finance(prospect_id).await;
            }
            if to_stage == "MATRICULADO" {
                // Lógica: Promocionar Prospecto a Alumno oficial en el SIS
                promote_to_official_student(prospect_id).await;
            }
        }
        CrmEvent::DocumentUploaded { prospect_id, doc_type } => {
            // Lógica: Marcar tarea de verificación para el equipo de admisión
            create_verification_task(prospect_id, doc_type).await;
        }
    }
}

4. Etapas del Plan de Implementación
Fase 1: Core del CRM (Semana 1-4)

    Implementación del CRUD de Prospects.

    Creación de la tabla de Activities para la línea de tiempo (Timeline).

    API en Axum para gestionar el cambio de estados en el Pipeline.

Fase 2: Gestión Documental e Integración (Semana 5-8)

    Integración con S3/Minio para carga de certificados.

    Desarrollo del Workflow Engine inicial (Eventos internos en Rust).

    Sincronización vía gRPC entre CRM y el módulo de Finanzas.

Fase 3: UI Avanzada (Semana 9-12)

    Vista de Kanban en Leptos/Dioxus.

    Dashboard de métricas de admisión (Tasa de conversión, tiempo promedio en etapa).

    Buscador global optimizado para postulantes.

5. Consideraciones de Integración con el Core

    Single Source of Truth: Una vez que el estado es 'MATRICULADO', el registro maestro se mueve al servicio SIS-Service.

    RBAC: El equipo de admisión tiene permisos limitados al CRM, mientras que el equipo UTP solo ve alumnos matriculados.

¿Cómo avanzar con esto?

Como mencionamos antes, este plan te da la base para que el CRM no sea solo una lista de nombres, sino un sistema vivo. Si ya tienes esto claro, desarrollemos el código para la vista Kanban en el frontend y el microservicio de gestión de archivos en S3 disponible opcionalmente y configurado por variables de entorno





# ─── Base de datos ────────────────────────────────────────────
DATABASE_URL=postgres://schoolcbb:schoolcbb@localhost:5432/schoolcbb

# ─── JWT ─────────────────────────────────────────────────────
JWT_SECRET=cambio-en-produccion

# ─── Gateway ─────────────────────────────────────────────────
GATEWAY_HOST=0.0.0.0
GATEWAY_PORT=3000
FRONTEND_URL=http://localhost:8080
IDENTITY_URL=http://localhost:3001
SIS_URL=http://localhost:3002
ACADEMIC_URL=http://localhost:3003
ATTENDANCE_URL=http://localhost:3004
NOTIFICATIONS_URL=http://localhost:3005

# ─── Identity Service ────────────────────────────────────────
IDENTITY_HOST=0.0.0.0
IDENTITY_PORT=3001

# ─── SIS Service ─────────────────────────────────────────────
SIS_HOST=0.0.0.0
SIS_PORT=3002

# ─── Academic Service ────────────────────────────────────────
ACADEMIC_HOST=0.0.0.0
ACADEMIC_PORT=3003

# ─── Attendance Service ──────────────────────────────────────
ATTENDANCE_HOST=0.0.0.0
ATTENDANCE_PORT=3004

# ─── Notifications Service ───────────────────────────────────
NOTIFICATIONS_HOST=0.0.0.0
NOTIFICATIONS_PORT=3005

# ─── Logging ─────────────────────────────────────────────────
RUST_LOG=info,schoolcbb=debug
