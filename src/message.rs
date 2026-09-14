#[derive(Debug)]
pub enum MessageKind {
    FailedOpen,
    FailedSave,
    Saved,
}

#[derive(Debug)]
pub struct Message {
    info: String,
    kind: MessageKind,
}

impl Message {
    pub fn new(info: &str, kind: MessageKind) -> Self {
        Self {
            info: info.to_string(),
            kind,
        }
    }

    pub fn text(&self) -> String {
        match self.kind {
            MessageKind::FailedOpen => format!("failed to open: {}", self.info),
            MessageKind::FailedSave => format!("failed to save at: {}", self.info),
            MessageKind::Saved => format!("saved at: {}", self.info),
        }
    }
}
