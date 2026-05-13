use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// RUT chileno (Rol Único Tributario).
///
/// Corresponde al identificador tributario de personas naturales y jurídicas
/// en Chile. Su formato canónico es `XX.XXX.XXX-Y`, donde los dígitos antes
/// del guión son el cuerpo del RUT y el carácter final es el dígito verificador
/// (0-9 o K). Internamente se almacena sin puntos ni guiones, en mayúsculas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rut(String);

/// Errores posibles al validar o construir un [`Rut`].
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum RutError {
    /// El formato ingresado no coincide con el patrón esperado `XX.XXX.XXX-Y` o `XXXXXXXX-Y`.
    #[error("Formato inválido: debe ser XX.XXX.XXX-Y o XXXXXXXX-Y")]
    InvalidFormat,
    /// El dígito verificador calculado no coincide con el proporcionado.
    #[error("Dígito verificador inválido")]
    InvalidDigit,
    /// La cadena limpia tiene menos de 2 caracteres (cuerpo + DV).
    #[error("RUT demasiado corto")]
    TooShort,
}

impl Rut {
    /// Construye un [`Rut`] validando formato y dígito verificador.
    ///
    /// Acepta formatos con o sin puntos y guión (ej: `11.111.111-1`, `111111111`,
    /// `16.666.666-K`). Internamente limpia la entrada y verifica el DV
    /// con el algoritmo del módulo 11.
    pub fn new(value: &str) -> Result<Self, RutError> {
        let cleaned = Self::clean(value);
        if cleaned.len() < 2 {
            return Err(RutError::TooShort);
        }
        let (body, dv) = cleaned.split_at(cleaned.len() - 1);
        let dv_char = match dv.chars().next() {
            Some(c) => c,
            None => return Err(RutError::TooShort),
        };
        if !Self::validate_dv(body, dv_char) {
            return Err(RutError::InvalidDigit);
        }
        Ok(Self(cleaned))
    }

    /// Limpia un RUT eliminando puntos, guiones y caracteres no alfanuméricos,
    /// y convierte el resultado a mayúsculas.
    ///
    /// Ejemplo: `"16.666.666-K"` → `"16666666K"`.
    pub fn clean(value: &str) -> String {
        value
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_uppercase()
    }

    /// Valida que el dígito verificador (`dv`) corresponda al cuerpo del RUT.
    ///
    /// Internamente delega en [`calculate_dv`] y compara el resultado.
    pub fn validate_dv(rut: &str, dv: char) -> bool {
        let expected = Self::calculate_dv(rut);
        expected == dv
    }

    /// Calcula el dígito verificador para un cuerpo de RUT usando el algoritmo
    /// del módulo 11. Cada dígito se multiplica por la secuencia 2,3,4,5,6,7
    /// (que se reinicia al llegar a 7). El resultado es `0`-`9` o `K`.
    pub fn calculate_dv(rut: &str) -> char {
        let mut sum: i32 = 0;
        let mut multiplier = 2;

        for c in rut.chars().rev() {
            if let Some(d) = c.to_digit(10) {
                sum += (d as i32) * multiplier;
                multiplier = if multiplier == 7 { 2 } else { multiplier + 1 };
            }
        }

        let remainder = sum % 11;
        let dv = 11 - remainder;

        match dv {
            11 => '0',
            10 => 'K',
            _ => std::char::from_digit(dv as u32, 10).unwrap_or('0'),
        }
    }

    /// Retorna una referencia al RUT almacenado (sin formato).
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consume el [`Rut`] y retorna el String interno (sin formato).
    pub fn into_inner(self) -> String {
        self.0
    }

    /// Crea un [`Rut`] sin validar. Útil para tests o cuando el valor ya fue validado externamente.
    pub fn new_unchecked(s: &str) -> Self {
        Self(s.to_string())
    }

    /// Calcula y retorna el dígito verificador del RUT almacenado.
    pub fn verifier_digit(&self) -> char {
        let (body, _) = self.0.split_at(self.0.len() - 1);
        Self::calculate_dv(body)
    }

    /// Retorna el cuerpo del RUT (todos los dígitos excepto el DV).
    pub fn body(&self) -> &str {
        let (body, _) = self.0.split_at(self.0.len() - 1);
        body
    }

    /// Retorna el RUT formateado con puntos y guión: `XX.XXX.XXX-Y`.
    pub fn format_dotted(&self) -> String {
        let (body, dv) = self.0.split_at(self.0.len() - 1);
        let formatted_body = body
            .chars()
            .rev()
            .collect::<Vec<_>>()
            .chunks(3)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join(".")
            .chars()
            .rev()
            .collect::<String>();
        format!("{formatted_body}-{dv}")
    }
}

impl From<String> for Rut {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for Rut {
    /// Muestra el RUT en formato `XX.XXX.XXX-Y`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_dotted())
    }
}

/// Referencia ligera a un usuario, utilizada en contextos donde se necesita
/// identificar a una persona por su ID, RUT y nombre completo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserReference {
    pub id: Uuid,
    pub rut: Rut,
    pub full_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_rut() {
        let rut = Rut::new("11111111-1");
        assert!(rut.is_ok());
        assert_eq!(rut.unwrap().format_dotted(), "11.111.111-1");
    }

    #[test]
    fn test_invalid_dv() {
        let rut = Rut::new("11111111-2");
        assert!(rut.is_err());
    }

    #[test]
    fn test_rut_with_k_dv() {
        let rut = Rut::new("16666666-K");
        assert!(rut.is_ok());
    }

    #[test]
    fn test_clean_format() {
        let rut = Rut::new("16.666.666-K");
        assert!(rut.is_ok());
    }

    #[test]
    fn test_new_unchecked() {
        let rut = Rut::new_unchecked("invalid-rut-123");
        assert_eq!(rut.as_str(), "invalid-rut-123");
        assert_eq!(rut.into_inner(), "invalid-rut-123");
    }

    #[test]
    fn test_calculate_dv_known_values() {
        assert_eq!(Rut::calculate_dv("11111111"), '1');
        assert_eq!(Rut::calculate_dv("16666666"), 'K');
        assert_eq!(Rut::calculate_dv("1"), '9');
        assert_eq!(Rut::calculate_dv("12345678"), '5');
        assert_eq!(Rut::calculate_dv("0"), '0');
    }

    #[test]
    fn test_as_str() {
        let rut = Rut::new("11111111-1").unwrap();
        assert_eq!(rut.as_str(), "111111111");
    }

    #[test]
    fn test_into_inner() {
        let rut = Rut::new("1-9").unwrap();
        let inner: String = rut.into_inner();
        assert_eq!(inner, "19");
    }

    #[test]
    fn test_body() {
        let rut = Rut::new("11111111-1").unwrap();
        assert_eq!(rut.body(), "11111111");
    }

    #[test]
    fn test_verifier_digit() {
        let rut = Rut::new("11111111-1").unwrap();
        assert_eq!(rut.verifier_digit(), '1');
        let rut = Rut::new("16666666-K").unwrap();
        assert_eq!(rut.verifier_digit(), 'K');
    }

    #[test]
    fn test_format_dotted() {
        let rut = Rut::new("11111111-1").unwrap();
        assert_eq!(rut.format_dotted(), "11.111.111-1");
        let rut = Rut::new("1-9").unwrap();
        assert_eq!(rut.format_dotted(), "1-9");
        let rut = Rut::new("12345678-5").unwrap();
        assert_eq!(rut.format_dotted(), "12.345.678-5");
    }

    #[test]
    fn test_display() {
        let rut = Rut::new("11111111-1").unwrap();
        assert_eq!(format!("{rut}"), "11.111.111-1");
    }
}
