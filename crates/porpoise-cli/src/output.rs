#[derive(Debug, Clone)]
pub enum OutputFormat {
    Plain,
    #[allow(dead_code)]
    Json,
    JsonPretty,
}

impl OutputFormat {
    pub fn format(&self, value: &serde_json::Value) -> String {
        match self {
            OutputFormat::Plain => self.format_plain(value),
            OutputFormat::Json => {
                serde_json::to_string(value).unwrap_or_else(|e| format!("error: {e}"))
            }
            OutputFormat::JsonPretty => {
                serde_json::to_string_pretty(value)
                    .unwrap_or_else(|e| format!("error: {e}"))
            }
        }
    }

    fn format_plain(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Object(m) => {
                let mut out = String::new();
                for (k, v) in m {
                    match v {
                        serde_json::Value::String(s) => {
                            out.push_str(&format!("{k}: {s}\n"));
                        }
                        serde_json::Value::Array(arr) => {
                            out.push_str(&format!("{k}:\n"));
                            for item in arr {
                                out.push_str(&format!("  {}\n", plain_line(item)));
                            }
                        }
                        other => {
                            out.push_str(&format!("{k}: {other}\n"));
                        }
                    }
                }
                out
            }
            serde_json::Value::Array(arr) => {
                arr.iter().map(plain_line).collect::<Vec<_>>().join("\n")
            }
            other => plain_line(other),
        }
    }
}

fn plain_line(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}
