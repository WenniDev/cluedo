use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum JudgmentKind {
    Marvelous,
    Perfect,
    Great,
    Good,
    Miss,
    Ok,
    Ng,
}

impl JudgmentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            JudgmentKind::Marvelous => "Marvelous",
            JudgmentKind::Perfect => "Perfect",
            JudgmentKind::Great => "Great",
            JudgmentKind::Good => "Good",
            JudgmentKind::Miss => "Miss",
            JudgmentKind::Ok => "Ok",
            JudgmentKind::Ng => "Ng",
        }
    }
}

impl TryFrom<u32> for JudgmentKind {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0x102C => Ok(JudgmentKind::Marvelous),
            0x102D => Ok(JudgmentKind::Perfect),
            0x102E => Ok(JudgmentKind::Great),
            0x102F => Ok(JudgmentKind::Good),
            0x1031 => Ok(JudgmentKind::Miss),
            0x1032 | 0x1034 => Ok(JudgmentKind::Ok),
            0x1033 | 0x1035 => Ok(JudgmentKind::Ng),
            _ => Err(()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Judment {
    pub kind: JudgmentKind,
    pub timing: i32,
}

impl Judment {
    pub fn new(kind: JudgmentKind, timing: i32) -> Self {
        Self { kind, timing }
    }

    pub fn as_str(&self) -> String {
        let suffix = if self.timing < 0 {
            " fast"
        } else if self.timing > 0 {
            " slow"
        } else {
            ""
        };
        format!("{}{suffix} (timing={})", self.kind.as_str(), self.timing)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum Message {
    Connected,
    Disconnected,
    Next,
    Judgment(Judment),
}

pub fn encode(msg: &Message) -> Option<Vec<u8>> {
    serde_json::to_vec(msg).ok()
}

pub fn decode(buf: &[u8]) -> Option<Message> {
    serde_json::from_slice(buf).ok()
}
