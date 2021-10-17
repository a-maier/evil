use serde::{Deserialize, Serialize};

#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Config {
    pub window_size: Option<(f32, f32)>
}
