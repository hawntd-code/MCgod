//! Detección del entorno de desarrollo: versión de la app, sistema y Java.

use mcgod_gradle::java::{self, JavaInfo};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Environment {
    pub app_version: String,
    pub os: String,
    pub java: Option<JavaInfo>,
}

pub fn detect() -> Environment {
    Environment {
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os: format!("{} {}", os_name(), std::env::consts::ARCH),
        java: java::detect(),
    }
}

fn os_name() -> &'static str {
    match std::env::consts::OS {
        "windows" => "Windows",
        "macos" => "macOS",
        "linux" => "Linux",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_reports_app_version_and_os() {
        let env = detect();
        assert_eq!(env.app_version, env!("CARGO_PKG_VERSION"));
        assert!(!env.os.is_empty());
    }

    #[test]
    fn serializes_in_camel_case_for_the_ui() {
        let env = Environment {
            app_version: "0.1.0".into(),
            os: "Windows x86_64".into(),
            java: Some(JavaInfo {
                version: "21".into(),
                major: Some(21),
                vendor: None,
                home: Some("C:/jdk".into()),
            }),
        };
        let json = serde_json::to_value(&env).unwrap();
        assert_eq!(json["appVersion"], "0.1.0");
        assert_eq!(json["java"]["major"], 21);
        assert_eq!(json["java"]["home"], "C:/jdk");
        assert!(json["java"]["vendor"].is_null());
    }
}
