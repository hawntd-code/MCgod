//! Detección del entorno de desarrollo: versión de la app, sistema y Java.

use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInfo {
    pub version: String,
    pub major: Option<u32>,
    pub vendor: Option<String>,
}

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
        java: detect_java(),
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

fn detect_java() -> Option<JavaInfo> {
    let mut cmd = Command::new("java");
    cmd.arg("-version");
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().ok()?;
    // `java -version` escribe en stderr.
    let mut text = String::from_utf8_lossy(&out.stderr).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stdout));
    parse_java_version(&text)
}

/// Interpreta la salida de `java -version`.
///
/// Ejemplos aceptados:
/// - `openjdk version "21.0.13.1" 2026-11-17 LTS`
/// - `java version "1.8.0_421"` (Java 8, el mayor es 8)
pub fn parse_java_version(text: &str) -> Option<JavaInfo> {
    let first = text.lines().find(|l| l.contains("version \""))?;
    let start = first.find('"')? + 1;
    let len = first[start..].find('"')?;
    let version = first[start..start + len].to_string();

    let mut parts = version.split(['.', '_', '-', '+']);
    let major = match parts.next().and_then(|p| p.parse::<u32>().ok()) {
        Some(1) => parts.next().and_then(|p| p.parse().ok()),
        other => other,
    };

    // Segunda línea, p. ej. "OpenJDK Runtime Environment Temurin-21.0.13.1+2 (build ...)"
    let vendor = text
        .lines()
        .find(|l| l.contains("Runtime Environment"))
        .and_then(|l| l.split("Runtime Environment").nth(1))
        .and_then(|rest| rest.split_whitespace().next())
        .filter(|tok| !tok.starts_with('('))
        .and_then(|tok| tok.split('-').next())
        .filter(|name| name.chars().any(|c| c.is_alphabetic()))
        .map(str::to_string);

    Some(JavaInfo { version, major, vendor })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temurin_21() {
        let out = "openjdk version \"21.0.13.1\" 2026-11-17 LTS\n\
                   OpenJDK Runtime Environment Temurin-21.0.13.1+2 (build 21.0.13.1+2-LTS)\n\
                   OpenJDK 64-Bit Server VM Temurin-21.0.13.1+2 (build 21.0.13.1+2-LTS, mixed mode, sharing)";
        let info = parse_java_version(out).unwrap();
        assert_eq!(info.version, "21.0.13.1");
        assert_eq!(info.major, Some(21));
        assert_eq!(info.vendor.as_deref(), Some("Temurin"));
    }

    #[test]
    fn java_8_legacy_scheme() {
        let out = "java version \"1.8.0_421\"\n\
                   Java(TM) SE Runtime Environment (build 1.8.0_421-b09)\n";
        let info = parse_java_version(out).unwrap();
        assert_eq!(info.major, Some(8));
        assert_eq!(info.vendor, None);
    }

    #[test]
    fn not_java() {
        assert_eq!(parse_java_version("'java' is not recognized"), None);
    }
}
