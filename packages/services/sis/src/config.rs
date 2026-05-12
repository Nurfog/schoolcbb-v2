use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub host: String,
    pub port: u16,
    pub jwt_secret: String,
    pub upload_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3002".into())
                .parse()
                .expect("PORT must be a valid number"),
            jwt_secret: env::var("JWT_SECRET").unwrap_or_else(|_| "cambio-en-produccion".into()),
            upload_dir: env::var("UPLOAD_DIR").unwrap_or_else(|_| "/tmp/uploads".into()),
        }
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
