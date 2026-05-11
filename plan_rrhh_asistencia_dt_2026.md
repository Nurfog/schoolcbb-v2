# Plan de Desarrollo: RRHH + Hub de Asistencia DT-Compliant (Chile 2026)

Este anexo detalla la implementación técnica para la ingesta de datos de asistencia desde software de terceros, cumpliendo con la normativa de la Dirección del Trabajo.

---

## 1. Métodos de Ingesta (Interface)

### A. API REST para Terceros (Push)
- **Endpoint:** `POST /api/hr/attendance/sync`
- **Seguridad:** API Keys por proveedor + Firma HMAC para asegurar que los datos no fueron alterados en el tránsito (Integridad exigida por DT).
- **Formato:** JSON estandarizado.

### B. Carga Masiva CSV (Manual)
- **Mapper Inteligente:** Interfaz en **Leptos** que permite al usuario mapear columnas del CSV de cualquier reloj biométrico a los campos internos del sistema.

---

## 2. Lógica de Validación (Rust Logic)

El sistema no solo guarda el dato, lo audita:

```rust
pub struct AttendanceLog {
    pub rut: String,
    pub timestamp: DateTime<Utc>,
    pub entry_type: EntryType, // "Entrada", "Salida Colación", "Retorno", "Salida"
    pub device_id: String,
    pub location_hash: Option<String>, // Para cumplimiento de geocerca DT
}

impl AttendanceValidator {
    pub fn validate_compliance(&self, log: &AttendanceLog, contract: &Contract) -> Result<(), ComplianceError> {
        // 1. Verificar exceso de jornada (> 10 hrs diarias o > 40 semanales)
        // 2. Verificar descanso mínimo de 12 horas entre jornadas
        // 3. Detectar inconsistencias (ej: dos entradas seguidas sin salida)
        Ok(())
    }
}
```

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
