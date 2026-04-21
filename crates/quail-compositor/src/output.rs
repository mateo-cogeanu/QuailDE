/// OutputState models the monitors or virtual outputs the compositor will own.
#[derive(Debug, Clone)]
pub struct OutputState {
    pub detected_outputs: usize,
    pub layout: &'static str,
}

impl OutputState {
    pub fn placeholder() -> Self {
        Self {
            detected_outputs: 0,
            layout: "not enumerated",
        }
    }
}
