use reqwest::Method;
use std::fmt::Display;
use std::borrow::Cow;

/// Represents an API route, used for rate limiting grouping.
#[derive(Debug, Clone)]
pub struct Route<'a> {
    pub path: Cow<'a, str>,
    pub method: Method,
}

impl<'a> Route<'a> {
    pub fn new(method: Method, path: impl Into<Cow<'a, str>>) -> Self {
        Self {
            path: path.into(),
            method,
        }
    }

    pub fn bucket_key(&self) -> String {
        
        let parts: Vec<&str> = self.path.split('/').collect();
        let mut key_parts = Vec::new();
        
        let mut i = 0;
        while i < parts.len() {
            let part = parts[i];
            if part.is_empty() {
                i += 1;
                continue;
            }

            key_parts.push(part);

            if matches!(part, "channels" | "guilds" | "webhooks") {
                if let Some(next) = parts.get(i + 1) {
                    key_parts.push(next);
                    i += 1;
                }
            } 
            
            else if part.chars().all(char::is_numeric) {
                 key_parts.pop();
                 key_parts.push(":id");
            }
            
            i += 1;
        }

        format!("{}:{}", self.method, key_parts.join("/"))
    }
}

impl<'a> Display for Route<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}
