use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Rut(pub String);

#[derive(Debug, thiserror::Error)]
pub enum RutError {
    #[error("Formato inválido: debe ser XX.XXX.XXX-Y o XXXXXXXX-Y")]
    InvalidFormat,
    #[error("Dígito verificador inválido")]
    InvalidDigit,
    #[error("RUT demasiado corto")]
    TooShort,
}

impl Rut {
    pub fn new(value: &str) -> Result<Self, RutError> {
        let cleaned = Self::clean(value);
        if cleaned.len() < 2 {
            return Err(RutError::TooShort);
        }
        let (body, dv) = cleaned.split_at(cleaned.len() - 1);
        if !Self::validate_dv(body, dv.chars().next().unwrap()) {
            return Err(RutError::InvalidDigit);
        }
        Ok(Self(cleaned))
    }

    pub fn clean(value: &str) -> String {
        value
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .collect::<String>()
            .to_uppercase()
    }

    pub fn validate_dv(rut: &str, dv: char) -> bool {
        let expected = Self::calculate_dv(rut);
        expected == dv
    }

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
            _ => std::char::from_digit(dv as u32, 10).unwrap(),
        }
    }

    pub fn verifier_digit(&self) -> char {
        let (body, _) = self.0.split_at(self.0.len() - 1);
        Self::calculate_dv(body)
    }

    pub fn body(&self) -> &str {
        let (body, _) = self.0.split_at(self.0.len() - 1);
        body
    }

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
        format!("{}-{}", formatted_body, dv)
    }
}

impl fmt::Display for Rut {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format_dotted())
    }
}

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
        let rut = Rut::new("18327622-K");
        assert!(rut.is_ok());
    }

    #[test]
    fn test_clean_format() {
        let rut = Rut::new("18.327.622-K");
        assert!(rut.is_ok());
    }
}
