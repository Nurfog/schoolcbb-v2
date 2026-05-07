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