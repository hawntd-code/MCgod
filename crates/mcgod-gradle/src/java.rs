//! Detección del Java instalado: versión, proveedor y carpeta (`java.home`).

use serde::Serialize;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaInfo {
    pub version: String,
    pub major: Option<u32>,
    pub vendor: Option<String>,
    /// Carpeta de instalación; es la que se pasa a Gradle como `JAVA_HOME`.
    pub home: Option<String>,
}

/// Ejecuta el `java` del PATH y lee su versión y su carpeta.
pub fn detect() -> Option<JavaInfo> {
    let mut cmd = Command::new("java");
    cmd.args(["-XshowSettings:properties", "-version"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    let out = cmd.output().ok()?;
    // Java escribe esto en stderr.
    let mut text = String::from_utf8_lossy(&out.stderr).into_owned();
    text.push_str(&String::from_utf8_lossy(&out.stdout));
    parse(&text)
}

/// Interpreta la salida de `java -XshowSettings:properties -version` (o solo `-version`).
pub fn parse(text: &str) -> Option<JavaInfo> {
    let first = text.lines().find(|l| l.contains("version \""))?;
    let start = first.find('"')? + 1;
    let len = first[start..].find('"')?;
    let version = first[start..start + len].to_string();

    let mut parts = version.split(['.', '_', '-', '+']);
    let major = match parts.next().and_then(|p| p.parse::<u32>().ok()) {
        Some(1) => parts.next().and_then(|p| p.parse().ok()),
        other => other,
    };

    // "OpenJDK Runtime Environment Temurin-21.0.13.1+2 (build ...)"
    let vendor = text
        .lines()
        // Se salta la propiedad `java.runtime.name = OpenJDK Runtime Environment`.
        .find(|l| l.contains("Runtime Environment") && !l.contains(" = "))
        .and_then(|l| l.split("Runtime Environment").nth(1))
        .and_then(|rest| rest.split_whitespace().next())
        .filter(|tok| !tok.starts_with('('))
        .and_then(|tok| tok.split('-').next())
        .filter(|name| name.chars().any(|c| c.is_alphabetic()))
        .map(str::to_string);

    // "    java.home = C:\Program Files\Eclipse Adoptium\jdk-21..."
    let home = text
        .lines()
        .map(str::trim)
        .find_map(|l| l.strip_prefix("java.home = "))
        .map(|h| h.trim().to_string())
        .filter(|h| !h.is_empty());

    Some(JavaInfo {
        version,
        major,
        vendor,
        home,
    })
}

impl JavaInfo {
    pub fn home_path(&self) -> Option<PathBuf> {
        self.home.as_ref().map(PathBuf::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEMURIN: &str = "Property settings:\n\
        \x20   file.encoding = UTF-8\n\
        \x20   java.home = C:\\Program Files\\Eclipse Adoptium\\jdk-21.0.13.10002-hotspot\n\
        \x20   java.runtime.name = OpenJDK Runtime Environment\n\
        \x20   java.version = 21.0.13.1\n\
        \n\
        openjdk version \"21.0.13.1\" 2026-11-17 LTS\n\
        OpenJDK Runtime Environment Temurin-21.0.13.1+2 (build 21.0.13.1+2-LTS)\n\
        OpenJDK 64-Bit Server VM Temurin-21.0.13.1+2 (build 21.0.13.1+2-LTS, mixed mode, sharing)";

    #[test]
    fn temurin_21_with_home() {
        let info = parse(TEMURIN).unwrap();
        assert_eq!(info.version, "21.0.13.1");
        assert_eq!(info.major, Some(21));
        assert_eq!(info.vendor.as_deref(), Some("Temurin"));
        assert_eq!(
            info.home.as_deref(),
            Some("C:\\Program Files\\Eclipse Adoptium\\jdk-21.0.13.10002-hotspot")
        );
    }

    #[test]
    fn version_only_output_has_no_home() {
        let out = "openjdk version \"21.0.13.1\" 2026-11-17 LTS\n\
                   OpenJDK Runtime Environment Temurin-21.0.13.1+2 (build 21.0.13.1+2-LTS)\n";
        assert_eq!(parse(out).unwrap().home, None);
    }

    #[test]
    fn java_8_legacy_scheme() {
        let out = "java version \"1.8.0_421\"\n\
                   Java(TM) SE Runtime Environment (build 1.8.0_421-b09)\n";
        let info = parse(out).unwrap();
        assert_eq!(info.major, Some(8));
        assert_eq!(info.vendor, None);
    }

    #[test]
    fn early_access_build_without_vendor_token() {
        let out = "openjdk version \"26-ea\" 2026-03-17\n\
                   OpenJDK Runtime Environment (build 26-ea+30-2840)\n";
        let info = parse(out).unwrap();
        assert_eq!(info.version, "26-ea");
        assert_eq!(info.major, Some(26));
        assert_eq!(info.vendor, None);
    }

    #[test]
    fn skips_picked_up_lines_before_version() {
        let out = "Picked up JAVA_TOOL_OPTIONS: -Dfile.encoding=UTF-8\n\
                   openjdk version \"21.0.5\" 2024-10-15 LTS\n\
                   OpenJDK Runtime Environment Microsoft-10377968 (build 21.0.5+11-LTS)\n";
        let info = parse(out).unwrap();
        assert_eq!(info.major, Some(21));
        assert_eq!(info.vendor.as_deref(), Some("Microsoft"));
    }

    #[test]
    fn not_java() {
        assert_eq!(parse("'java' is not recognized"), None);
    }
}
