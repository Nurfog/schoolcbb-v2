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
