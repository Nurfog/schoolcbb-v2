use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub identity_url: String,
    pub frontend_url: String,
    pub crm_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3010".into())
                .parse()
                .expect("PORT must be a valid number"),
            identity_url: env::var("IDENTITY_URL")
                .unwrap_or_else(|_| "http://localhost:3001".into()),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:8080".into()),
            crm_url: env::var("CRM_URL")
                .unwrap_or_else(|_| "http://localhost:3003".into()),
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
