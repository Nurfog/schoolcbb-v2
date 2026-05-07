---
name: curriculo-chile
description: >
  Asistente experto en el Currículum Nacional de Chile, normativa educacional MINEDUC,
  evaluación y promoción escolar (Decreto 67), inclusión y NEE (Decreto 83, Ley TEA),
  estándares docentes (MBE, CPEIP), SIMCE, OIC, y protección de datos (Ley 21.719).
  Use esta skill cuando: (1) se necesite consultar las Bases Curriculares por nivel y asignatura,
  (2) se requiera información sobre normativa educativa chilena vigente, (3) se necesiten
  orientaciones sobre evaluación, calificación y promoción, (4) se requiera apoyo sobre
  adecuación curricular e inclusión, (5) se consulten estándares de calidad y desempeño,
  (6) se necesite validar datos según formato MINEDUC/SIGE.
license: MIT
compatibility: Rust+Actix-web backend, React frontend, PostgreSQL
metadata:
  author: SchoolCCB
  version: "1.0"
  source: MINEDUC, BCN Ley Chile, CPEIP, Agencia de Calidad, Supereduc
allowed-tools: Bash(cargo:*) Bash(pdftotext:*) Read Write Edit Glob Grep WebFetch
---

# Currículum Nacional de Chile — Skill del Agente Curricular

Agente especializado en el sistema educativo chileno. Responde consultas sobre currículum, normativa, evaluación, inclusión y estándares de calidad.

## Skills del Currículum Nacional (cn/)

Skills detalladas por nivel, asignatura y tipo documental dentro de `.agents/skills/cn/`:

### Transversales
- [Normativa](cn/normativa/SKILL.md): decretos, leyes y resoluciones
- [Bases Curriculares](cn/bases_curriculares/SKILL.md): todos los niveles
- [Planes de Estudio](cn/planes_estudio/SKILL.md): distribución horaria
- [Programas de Estudio](cn/programas_estudio/SKILL.md): por asignatura

### Por curso y asignatura
Se generan automáticamente con `docs/cn/process_rag.py`. Cada skill contiene:
- `SKILL.md` con descripción y consultas típicas
- `kb.json` con chunks + embeddings para RAG semántico

### Procesamiento local
```bash
cd docs/cn
python3 process_rag.py          # Extrae PDFs → chunks → embeddings → skills
```

### Backend RAG (KB consolidada)
- **KB chunks:** `rust/data/kb-chunks.json` + `docs/cn/kb-cn-chunks.json`
- **Endpoint:** `POST /api/curriculum/search` con soporte embeddings

## Referencias adicionales

- [Estructura del Sistema Educativo](references/01-estructura.md)
- [Bases Curriculares por Nivel](references/02-bases-curriculares.md)
- [Marco Legal](references/03-marco-legal.md)
- [Estándares y Calidad](references/04-estandares.md)
- [Validaciones SIGE](references/05-sige.md)
- [Plazos Legales](references/06-plazos.md)

## Arquitectura del Agente

### Backend
- **Handler:** `rust/src/handlers/curriculum_agent.rs`
- **Endpoint:** `POST /api/curriculum/search` — recibe `{ q: string, limit?: number }`, devuelve chunks rankeados con fuentes
- **KB:** `rust/data/kb-chunks.json` — 4.270 chunks (~1M chars) extraídos de fuentes oficiales
- **RAG opcional:** Si `OLLAMA_URL` está configurado, genera respuesta contextualizada vía Ollama
- **KB compartida:** `Arc<RwLock<CurriculumKB>>` cargada al inicio del servidor

### Frontend
- **Componente:** `frontend/src/components/CurriculumAgent.jsx`
- **Sidebar:** visible para todos los usuarios no-root
- **Chat:** consulta al backend, muestra respuestas con fuentes citadas

## Consultas típicas

| Categoría | Ejemplos |
|-----------|----------|
| Bases Curriculares | OA de Matemática 1° Básico, asignaturas de 7° Básico |
| Evaluación | Escala de notas, promoción, asistencia mínima |
| Legislación | Decreto 67, Ley TEA, Ley 21.719, Modo Aula |
| Inclusión | PIE, PACI, DUA, adecuaciones curriculares |
| Calidad | SIMCE, MBE, Categoría de Desempeño, OIC |
| Trámites | Certificados SIGE, validación de estudios |

## Comandos relacionados

```bash
# Reconstruir KB desde las fuentes
cd docs/curriculo && python3 rebuild-kb.py

# Ver estado del agente (desde backend)
curl http://localhost:8080/api/curriculum/info

# Buscar en la KB
curl -X POST http://localhost:8080/api/curriculum/search \
  -H 'Content-Type: application/json' \
  -d '{"q": "Decreto 67 promoción", "limit": 3}'
```
