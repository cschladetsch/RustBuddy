use crate::config::TranscriptionConfig;
use std::{path::Path, sync::Arc};
use vosk::{CompleteResult, Model, Recognizer};

pub struct Transcriber {
    model: Arc<Model>,
    sample_rate: f32,
}

impl Transcriber {
    pub fn new(cfg: &TranscriptionConfig, sample_rate: u32) -> Result<Self, TranscriptionError> {
        let model_path = resolve_path(&cfg.model_path);
        let model = Model::new(&model_path).ok_or_else(|| {
            TranscriptionError::Model(format!("unable to load model at {}", model_path))
        })?;
        Ok(Self {
            model: Arc::new(model),
            sample_rate: sample_rate as f32,
        })
    }

    pub fn transcribe(&self, audio: &[i16]) -> Result<String, TranscriptionError> {
        if audio.is_empty() {
            return Ok(String::new());
        }

        let mut recognizer = Recognizer::new(&self.model, self.sample_rate)
            .ok_or_else(|| TranscriptionError::Recognizer("failed to create recognizer".into()))?;
        recognizer.accept_waveform(audio);
        let result = recognizer.final_result();
        Ok(extract_text(result))
    }
}

fn resolve_path(path: &Path) -> String {
    if path.is_absolute() {
        path.to_string_lossy().to_string()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| Path::new(".").to_path_buf())
            .join(path)
            .to_string_lossy()
            .to_string()
    }
}

fn extract_text(result: CompleteResult<'_>) -> String {
    match result {
        CompleteResult::Single(single) => single.text.to_string(),
        CompleteResult::Multiple(multi) => multi
            .alternatives
            .first()
            .map(|alt| alt.text.to_string())
            .unwrap_or_default(),
    }
}

#[derive(Debug)]
pub enum TranscriptionError {
    Model(String),
    Recognizer(String),
}

impl std::fmt::Display for TranscriptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Model(err) => write!(f, "failed loading Vosk model: {}", err),
            Self::Recognizer(err) => write!(f, "transcription error: {}", err),
        }
    }
}

impl std::error::Error for TranscriptionError {}
