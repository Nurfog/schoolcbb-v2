# Validaciones de Datos — Formato SIGE/MINEDUC

## RUN/RUT
Formato XX.XXX.XXX-Y o XXXXXXXX-Y. Módulo 11, dígito verificador.

## Códigos de Curso
Ej: 1°Básico A = 1BAS_A. COD_NIVEL: PB01–PB08, MM01–MM04.

## Códigos de Asignatura
LEN01 (Lenguaje), MAT01 (Matemática), etc.

## Calificaciones (SIGE)
- TIPO_EVAL: PA (Parcial), SF (Semestral), AN (Anual), EX (Examen)
- CALIFICACION: DECIMAL(4,1), rango 1.0–7.0
- SITUACION_FINAL: APR, REP, EXM

## Asistencia (SIGE)
A=Asiste, F=Falta, ATR=Atraso, J=Justificado, L=Licencia

## Matrícula
CONDICION: AL (Regular), RE (Repitente), TR (Trasladado)
PRIORITARIO: 1 (Prioritario), 2 (Preferente), 0 (No)
NEE: T (Transitoria), P (Permanente), N (No)
