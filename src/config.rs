use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub bot: BotConfig,
    pub image: ImageConfig,
    pub logging: LoggingConfig,
    pub ai: AICfg,
    pub grpc: GrpcCfg,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BotConfig {
    pub discord_token: String,
    pub bot_activity: String,
    pub config: BotConfigDetails,
}

#[derive(Debug, Deserialize, Clone)]
pub struct BotConfigDetails {
    pub remove_old_commands: bool,
    pub db_type: String,
    pub cache_type: String,
    pub tui: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ImageConfig {
    pub save_image: String,
    pub save_server: Option<String>,
    pub token: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub log_level: String,
    pub max_log_retention: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AICfg {
    pub ai_token: String,
    pub ai_base_url: String,
    pub image: AICfgImage,
    pub question: AICfgQuestion,
    pub transcription: AICfgTranscription,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AICfgImage {
    pub ai_image_token: Option<String>,
    pub ai_image_base_url: Option<String>,
    pub ai_image_model: Option<String>,
    pub ai_image_quality: Option<String>,
    pub ai_image_size: Option<String>,
    pub ai_image_style: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AICfgQuestion {
    pub ai_question_token: Option<String>,
    pub ai_question_base_url: Option<String>,
    pub ai_question_model: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AICfgTranscription {
    pub ai_transcription_token: Option<String>,
    pub ai_transcription_base_url: Option<String>,
    pub ai_transcription_model: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrpcCfg {
    pub grpc_is_on: bool,
    pub grpc_port: u16,
    pub use_tls: bool,
    pub tls_cert_path: String,
    pub tls_key_path: String,
    pub federation: FederationCfg,
}

#[derive(Debug, Deserialize, Clone)]
pub struct FederationCfg {
    pub federation_is_on: bool,
    pub federation_name: String,
    pub self_address: String,
    pub new_federation: bool,
    pub federation_address: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            bot: BotConfig {
                discord_token: "".to_string(),
                bot_activity: "".to_string(),
                config: BotConfigDetails {
                    remove_old_commands: false,
                    db_type: "sqlite".to_string(),
                    cache_type: "in-memory".to_string(),
                    tui: false,
                },
            },
            image: ImageConfig {
                save_image: "local".to_string(),
                save_server: None,
                token: None,
            },
            logging: LoggingConfig {
                log_level: "info".to_string(),
                max_log_retention: 30,
            },
            ai: AICfg {
                ai_token: "".to_string(),
                ai_base_url: "".to_string(),
                image: AICfgImage {
                    ai_image_token: None,
                    ai_image_base_url: None,
                    ai_image_model: None,
                    ai_image_quality: None,
                    ai_image_size: None,
                    ai_image_style: None,
                },
                question: AICfgQuestion {
                    ai_question_token: None,
                    ai_question_base_url: None,
                    ai_question_model: None,
                },
                transcription: AICfgTranscription {
                    ai_transcription_token: None,
                    ai_transcription_base_url: None,
                    ai_transcription_model: None,
                },
            },
            grpc: GrpcCfg {
                grpc_is_on: false,
                grpc_port: 443,
                use_tls: false,
                tls_cert_path: "cert/cert.pem".to_string(),
                tls_key_path: "cert/key.pem".to_string(),
                federation: FederationCfg {
                    federation_is_on: false,
                    federation_name: "".to_string(),
                    self_address: "".to_string(),
                    new_federation: false,
                    federation_address: None,
                },
            },
        }
    }
}

impl Config {
    pub fn set_default_value_on_none(&mut self) {
        if self.ai.image.ai_image_token.is_none()
            || self.ai.image.ai_image_token.as_ref().unwrap().is_empty()
        {
            self.ai.image.ai_image_token = Some(self.ai.ai_token.clone());
        }
        if self.ai.image.ai_image_base_url.is_none()
            || self.ai.image.ai_image_base_url.as_ref().unwrap().is_empty()
        {
            self.ai.image.ai_image_base_url = Some(self.ai.ai_base_url.clone());
        }
        if self.ai.image.ai_image_model.is_none()
            || self.ai.image.ai_image_model.as_ref().unwrap().is_empty()
        {
            self.ai.image.ai_image_model = Some("dall-e-3".to_string());
        }

        if self.ai.question.ai_question_token.is_none()
            || self
                .ai
                .question
                .ai_question_token
                .as_ref()
                .unwrap()
                .is_empty()
        {
            self.ai.question.ai_question_token = Some(self.ai.ai_token.clone());
        }
        if self.ai.question.ai_question_base_url.is_none()
            || self
                .ai
                .question
                .ai_question_base_url
                .as_ref()
                .unwrap()
                .is_empty()
        {
            self.ai.question.ai_question_base_url = Some(self.ai.ai_base_url.clone());
        }
        if self.ai.question.ai_question_model.is_none()
            || self
                .ai
                .question
                .ai_question_model
                .as_ref()
                .unwrap()
                .is_empty()
        {
            self.ai.question.ai_question_model = Some("gpt-3.5-turbo".to_string());
        }

        if self.ai.transcription.ai_transcription_token.is_none()
            || self
                .ai
                .transcription
                .ai_transcription_token
                .as_ref()
                .unwrap()
                .is_empty()
        {
            self.ai.transcription.ai_transcription_token = Some(self.ai.ai_token.clone());
        }
        if self.ai.transcription.ai_transcription_base_url.is_none()
            || self
                .ai
                .transcription
                .ai_transcription_base_url
                .as_ref()
                .unwrap()
                .is_empty()
        {
            self.ai.transcription.ai_transcription_base_url = Some(self.ai.ai_base_url.clone());
        }
        if self.ai.transcription.ai_transcription_model.is_none()
            || self
                .ai
                .transcription
                .ai_transcription_model
                .as_ref()
                .unwrap()
                .is_empty()
        {
            self.ai.transcription.ai_transcription_model = Some("whisper-1".to_string());
        }
    }
}
