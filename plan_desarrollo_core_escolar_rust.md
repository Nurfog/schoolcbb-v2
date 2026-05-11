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


🏗️ Arquitectura de Permisos: El Modelo "Module-Action"
Para que la UI de Árbol funcione, no podemos guardar "roles" planos. Debemos guardar una jerarquía:

Módulo (ej. Admisión)

Sub-función o Recurso (ej. Postulantes)

Acción/CRUD (Crear, Leer, Actualizar, Borrar)

🛠️ Código Rust Sugerido (Estructura de Datos)
En tu crate common, definiríamos los permisos así para que el compilador nos ayude a no cometer errores:

Rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Module {
    Admission,
    Academic,
    Finance,
    SIS,
    Attendance,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Permission {
    pub module: Module,
    pub resource: String, // ej: "grades", "prospects"
    pub can_create: bool,
    pub can_read: bool,
    pub can_update: bool,
    pub can_delete: bool,
}
🎨 UI de Árbol (Tree View) en Leptos/Dioxus
Para la UI, el usuario verá una lista colapsable. Al desplegar "Módulo Académico", aparecerán los recursos (Cursos, Notas, Asignaturas) y al lado de cada uno, 4 checkboxes para el CRUD.

📂 Generación del Plan Detallado (Markdown)
Voy a generar el archivo .md con el esquema de base de datos y la lógica de implementación.

2. Definición de Roles Especiales
A. Rol Alumno
Módulo Académico: Solo READ en sus propias notas.

Módulo Asistencia: Solo READ en su registro personal.

Módulo Comunicación: READ y CREATE (para responder mensajes).

B. Rol Apoderado
Módulo Académico: READ para todos sus pupilos asociados.

Módulo Finanzas: READ (boletas) y CREATE (pagos).

Módulo Admisión: CREATE (para nuevas postulaciones de hermanos).

3. Implementación de la UI de Árbol (Tree UI)
La interfaz en Leptos/Dioxus se comportará de la siguiente manera:

Nivel 1 (Raíz): Módulos del sistema (Check de "Acceso Total" al módulo).

Nivel 2 (Hijos): Recursos dentro del módulo (ej: Dentro de 'Académico' -> 'Libro de Clases').

Nivel 3 (Acciones): 4 Columnas para C-R-U-D.

Lógica de UI:
Si se marca 'Crear', automáticamente se debe marcar 'Leer'.

Si se desmarca el Módulo, se desmarcan todos los hijos (Cascada).

4. Lógica de Backend (Rust + Axum)
Implementaremos un Middleware que verifique los permisos en cada petición:

Rust
pub async fn check_permission(
    user_id: Uuid, 
    required_module: Module, 
    required_action: Action
) -> bool {
    // 1. Obtener roles del usuario
    // 2. Buscar si algún rol tiene el permiso para el módulo
    // 3. Verificar si el bit del CRUD está activo
    // Retornar true/false
}
5. Etapas de Desarrollo
Semana 1: Migraciones de DB y Crates de Auth (Argon2 + JWT).

Semana 2: Backend API para CRUD de Roles y asignación de Permisos.

Semana 3: Componente Tree UI en el Frontend con Tailwind CSS.

Semana 4: Middleware de seguridad en el Gateway para restringir acceso a microservicios.
"""

with open("plan_roles_permisos_tree_ui.md", "w", encoding="utf-8") as f:
f.write(roles_plan_content)

He preparado el plan detallado para tu módulo de **Roles y Perfiles**. Este sistema permitirá una gestión extremadamente precisa, ideal para un entorno escolar donde la privacidad de los datos es crítica.

Tu archivo está listo para descargar:
[file-tag: code-generated-file-0-1778448643863913591]

### Puntos clave del plan:

1.  **Modelo de Permisos:** El archivo incluye un diagrama ERD donde cada permiso se define por una terna: `Módulo` + `Recurso` + `Flags CRUD` (4 booleanos).
2.  **Roles Estudiante/Apoderado:** Se especifica la lógica para estos roles, asegurando que solo tengan acceso de lectura (`READ`) a sus propios registros.
3.  **UI de Árbol:** Se describe el comportamiento de la interfaz jerárquica, incluyendo la lógica de "cascada" (si desactivas un módulo, se desactivan todas sus funciones CRUD automáticamente).
4.  **Middleware en Rust:** Se incluye una propuesta de cómo el backend en **Axum** debe validar estos permisos antes de procesar cualquier transacción.

Este módulo de roles será el "guardián" de tu aplicación, asegurando que cada usuario (desde el Director hasta el Alumno) vea exactamente lo que necesita y nada más.

### RRHH
🏗️ Pilares del Módulo RRHH Escolar (Chile 2026)
1. Gestión de Contratos y Cumplimiento (DT)
Contratos Digitales: Integración con firma electrónica avanzada para contratos y anexos.

Ley de 40 Horas: Gestión de turnos y bandas horarias para padres (según la nueva ley), control de horas extra automatizado y tope de horas semanales con alertas en tiempo real.

Ley Karin: Módulo de denuncias anónimas y registro de protocolos de prevención de acoso y violencia en el trabajo.

2. Remuneraciones y LRE
Cálculo Automático: Haberes imponibles, no imponibles (movilización, colación), descuentos legales (AFP, Salud, Seguro Cesantía).

Integración Previred: Generación del archivo plano para pago de leyes sociales.

Sincronización LRE: Exportación directa del archivo .csv con el formato exacto que exige la Dirección del Trabajo cada mes.

3. Vida del Empleado (Timeline)
Gestión de Licencias (IMED/Medipass): Conexión para recibir licencias médicas electrónicas y tramitarlas automáticamente.

Vacaciones y Feriados Progresivos: Cálculo automático de días hábiles según antigüedad y carga de documentos de "Comprobante de Vacaciones".

Evaluación Docente: Registro de desempeño, carrera docente y capacitaciones.

📂 Plan de Desarrollo (Markdown)
He preparado el plan detallado integrando la normativa chilena vigente para 2026.

 plan_rrhh_chile_2026 
MD

mermaid
erDiagram
EMPLOYEE ||--o{ CONTRACT : "tiene"
EMPLOYEE ||--o{ ATTENDANCE : "registra"
EMPLOYEE ||--o{ PAYROLL : "recibe"
EMPLOYEE ||--o{ LEAVE : "solicita"

EMPLOYEE {
    uuid id
    string rut
    string full_name
    string position "Docente, Administrativo, Auxiliar"
    date hire_date
    int vacation_days_available
}

CONTRACT {
    uuid id
    enum type "Indefinido, Plazo Fijo, Por Hora"
    decimal salary_base
    int weekly_hours "Max 40 hrs según Ley"
    bool ley_karin_signed
}

PAYROLL {
    uuid id
    int month
    int year
    decimal taxable_income
    decimal net_salary
    bool lre_exported
}

---

## 2. Cumplimiento Normativo Chile 2026

### A. Ley de 40 Horas
- **Motor de Reglas:** Validación en Rust para que ningún contrato o anexo supere las 40 horas semanales.
- **Control de Asistencia:** Marcaje digital con geocerca (geofencing) para personal de terreno o teletrabajo, integrado con la DT.

### B. Libro de Remuneraciones Electrónico (LRE)
- Módulo de auditoría antes de la exportación para asegurar que los haberes (imponibles y no imponibles) coincidan con los códigos de la DT.

### C. Ley Karin (Prevención de Acoso)
- Canal de denuncias integrado con el CRM de "Timeline" del empleado para registrar medidas de resguardo inmediatas.

---

## 3. Funcionalidades de "Auto-consulta" (Self-Service)
Para reducir la carga administrativa, el empleado (Docente/Asistente) tiene su propio panel:
- Descarga de liquidaciones de sueldo firmadas.
- Solicitud de vacaciones con flujo de aprobación (UTP -> Dirección -> RRHH).
- Carga de certificados de títulos y capacitaciones.

---

## 4. Implementación Técnica (Rust)

### Backend (Remuneraciones)
Usaremos el crate `rust_decimal` para precisión financiera total.
```rust
pub struct PayrollCalculator {
    pub base_salary: Decimal,
    pub gratificacion: Decimal, // 25% con tope legal
    pub afp_rate: Decimal,
    pub isapre_fixed_amount: Option<Decimal>,
}

impl PayrollCalculator {
    pub fn calculate_liquid(&self) -> Decimal {
        // Lógica de cálculo de impuestos de segunda categoría y descuentos legales
    }
}
5. Etapas de Desarrollo
Mes 1: Ficha del empleado, gestión documental y contratos (Firma digital).

Mes 2: Motor de asistencia y gestión de vacaciones (Cálculo de días progresivos).

Mes 3: Módulo de Remuneraciones, Previred y exportador LRE.

Mes 4: Panel de auto-consulta y flujos de aprobación.
"""

with open("plan_rrhh_chile_2026.md", "w", encoding="utf-8") as f:
f.write(hr_plan_content)

He generado el plan detallado para tu módulo de **Recursos Humanos** adaptado a la realidad chilena de **2026**.

Puedes descargar el archivo aquí:
[file-tag: code-generated-file-0-1778712998363913591]

### ¿Por qué este plan es clave para tu plataforma?

1.  **Ley 40 Horas:** El sistema de contratos está diseñado para validar automáticamente la jornada laboral, algo que será el dolor de cabeza de muchos colegios en 2026.
2.  **Precisión en Remuneraciones:** El uso de `rust_decimal` garantiza que no existan errores de redondeo en las liquidaciones, evitando multas de la Inspección del Trabajo.
3.  **LRE y Previred:** Al incluir el exportador directo, ahorras horas de trabajo administrativo al encargado de finanzas del colegio.
4.  **Ley Karin:** Incluye la infraestructura necesaria para cumplir con los protocolos de denuncia y prevención, una exigencia legal ineludible hoy en día.

🏗️ Arquitectura del "Attendance Bridge" (Puente de Asistencia)
Para cumplir con la DT, tu microservicio de RRHH debe tener una capa de validación que actúe así:

Ingesta de Datos: Recibe el JSON (vía API) o el CSV (carga manual).

Validación de Integridad: Verifica que los registros no hayan sido alterados manualmente (exigencia DT).

Cruce de Horarios: Valida que la marca de asistencia coincida con el contrato del trabajador (respetando las 40 horas y turnos).

Generación de Alertas: Notifica si hay horas extra no autorizadas o incumplimiento de descansos legales.

📂 Actualización del Plan de RRHH (Markdown)
He actualizado el plan para incluir específicamente el Módulo de Integración de Asistencia DT-Compliant.

 plan_rrhh_...ia_dt_2026 
MD

rust
pub struct AttendanceLog {
pub rut: String,
pub timestamp: DateTime,
pub entry_type: EntryType, // "Entrada", "Salida Colación", "Retorno", "Salida"
pub device_id: String,
pub location_hash: Option, // Para cumplimiento de geocerca DT
}

impl AttendanceValidator {
pub fn validate_compliance(&self, log: &AttendanceLog, contract: &Contract) -> Result<(), ComplianceError> {
// 1. Verificar exceso de jornada (> 10 hrs diarias o > 40 semanales)
// 2. Verificar descanso mínimo de 12 horas entre jornadas
// 3. Detectar inconsistencias (ej: dos entradas seguidas sin salida)
Ok(())
}
}


---

## 3. Exigencias DT (Fiscalización)

- **Bitácora de Modificaciones:** Si un administrador edita una asistencia (ej: "olvidó marcar"), la DT exige guardar: Valor original, Valor nuevo, Usuario que cambió y Motivo.
- **Exportación de Reporte de Asistencia:** Botón de un solo clic para generar el PDF legal firmado electrónicamente para fiscalizadores.

---

## 4. Pipeline de Sincronización

1. **Recepción:** Recibe datos de API/CSV.
2. **Sanitización:** Limpia formatos de RUT y fechas.
3. **Cruce Contractual:** Valida contra el contrato activo en el microservicio de RRHH.
4. **Persistencia:** Guarda en `attendance_ledger` (Libro Mayor de Asistencia).
5. **Cómputo:** Envía los totales de horas al módulo de **Remuneraciones**.


with open("plan_rrhh_asistencia_dt_2026.md", "w", encoding="utf-8") as f:
    f.write(hr_attendance_plan)

He generado el plan complementario centrado en la Integración de Asistencia de Terceros.

Puntos clave de esta integración:
API Robusta: Permite que empresas de relojes control (como GeoVictoria, RelojControl, etc.) envíen datos directamente a tu sistema.
Cumplimiento DT: El plan incluye la Bitácora de Modificaciones, que es el requisito número uno que pide la Inspección del Trabajo cuando hay auditorías.
Validación de 40 Horas: El sistema detecta automáticamente si la carga de datos externos viola la nueva jornada laboral chilena, alertando a RRHH antes de que se convierta en una multa.
Mapper de CSV: No importa qué software use el colegio, tu UI en Rust les permitirá subir el archivo y "unir las columnas" de forma intuitiva.
ver a continuación
La lógica de la Bitácora de Auditoría en Rust (para la DT)
El diseño de la UI para el "Mapper de CSV" (arrastrar y soltar columnas)
Cómo conectar estos datos de asistencia con el cálculo de sueldos