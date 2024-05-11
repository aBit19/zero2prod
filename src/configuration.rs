use secrecy::Secret;
use serde_aux::field_attributes::deserialize_number_from_string;
#[derive(serde::Deserialize)]
pub struct Settings {
    pub database: DatabaseSettings,
    pub application: ApplicationSettings,
    pub email_client: EmailClientSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub base_url: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    pub host: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
    pub require_ssl: bool,
}

pub fn get_configuration() -> Settings {
    let configuration_dir = std::env::current_dir()
        .expect("Failed to determine the current directory.")
        .join("configuration");

    let env_config: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "dev".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT");

    let settings = config::Config::builder()
        .add_source(config::File::from(configuration_dir.join("base.yaml")))
        .add_source(config::File::from(
            configuration_dir.join(env_config.as_str()),
        ))
        .add_source(
            config::Environment::with_prefix("APP")
                .prefix_separator("_")
                .separator("__"),
        )
        .build()
        .expect("Failed to build configuration.");
    settings
        .try_deserialize::<Settings>()
        .expect("Failed to deserialize settings.")
}

enum Environment {
    Dev,
    Production,
}

impl Environment {
    fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "dev" => Ok(Environment::Dev),
            "production" => Ok(Environment::Production),
            _ => Err(format!("{} is not a valid environment", value)),
        }
    }
}

#[derive(serde::Deserialize)]
pub struct EmailClientSettings {
    pub base_url: String,
    pub sender_email: String,
    pub authorization_token: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub timeout_millis: u64,
}
