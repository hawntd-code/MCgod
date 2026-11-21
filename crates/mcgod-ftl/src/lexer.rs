//! Divide el texto de una plantilla en piezas: texto, `${…}`, `<#…>`, `</#…>`,
//! `<@…>`, `</@…>` y comentarios. Aplica el recorte de espacios de FreeMarker.

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Text(String),
    Interp(String, u32),
    Comment,
    Open {
        name: String,
        params: String,
        self_closing: bool,
        line: u32,
    },
    Close {
        name: String,
        line: u32,
    },
    UserOpen {
        params: String,
        self_closing: bool,
        line: u32,
    },
    UserClose {
        line: u32,
    },
}

impl Tok {
    fn is_tag(&self) -> bool {
        !matches!(self, Tok::Text(_) | Tok::Interp(..))
    }
}

pub struct LexError {
    pub line: u32,
    pub message: String,
}

fn line_at(starts: &[usize], pos: usize) -> u32 {
    (starts.partition_point(|&s| s <= pos)) as u32
}

pub fn tokenize(src: &str) -> Result<Vec<Tok>, LexError> {
    let starts: Vec<usize> = std::iter::once(0)
        .chain(src.match_indices('\n').map(|(i, _)| i + 1))
        .collect();
    let b = src.as_bytes();
    let mut toks = Vec::new();
    let mut text_start = 0;
    let mut i = 0;
    let err = |pos: usize, message: String| LexError {
        line: line_at(&starts, pos),
        message,
    };

    let flush = |toks: &mut Vec<Tok>, from: usize, to: usize| {
        if to > from {
            toks.push(Tok::Text(src[from..to].to_string()));
        }
    };

    while i < b.len() {
        let rest = &src[i..];
        if rest.starts_with("<#--") {
            flush(&mut toks, text_start, i);
            let end = rest
                .find("-->")
                .ok_or_else(|| err(i, "comentario <#-- sin cerrar".into()))?;
            toks.push(Tok::Comment);
            i += end + 3;
            text_start = i;
        } else if rest.starts_with("</#") {
            flush(&mut toks, text_start, i);
            let end = rest
                .find('>')
                .ok_or_else(|| err(i, "etiqueta </# sin cerrar".into()))?;
            toks.push(Tok::Close {
                name: rest[3..end].trim().to_string(),
                line: line_at(&starts, i),
            });
            i += end + 1;
            text_start = i;
        } else if rest.starts_with("<#") && rest[2..].starts_with(|c: char| c.is_ascii_alphabetic())
        {
            flush(&mut toks, text_start, i);
            let name_len = rest[2..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len() - 2);
            let name = rest[2..2 + name_len].to_string();
            let params_start = i + 2 + name_len;
            let (end, self_closing) = find_tag_end(src, params_start)
                .ok_or_else(|| err(i, format!("directiva <#{name} sin cerrar")))?;
            let params_end = if self_closing { end - 1 } else { end };
            toks.push(Tok::Open {
                name,
                params: src[params_start..params_end].trim().to_string(),
                self_closing,
                line: line_at(&starts, i),
            });
            i = end + 1;
            text_start = i;
        } else if rest.starts_with("</@") {
            flush(&mut toks, text_start, i);
            let end = rest
                .find('>')
                .ok_or_else(|| err(i, "etiqueta </@ sin cerrar".into()))?;
            toks.push(Tok::UserClose {
                line: line_at(&starts, i),
            });
            i += end + 1;
            text_start = i;
        } else if rest.starts_with("<@") {
            flush(&mut toks, text_start, i);
            let (end, self_closing) =
                find_tag_end(src, i + 2).ok_or_else(|| err(i, "llamada <@ sin cerrar".into()))?;
            let params_end = if self_closing { end - 1 } else { end };
            toks.push(Tok::UserOpen {
                params: src[i + 2..params_end].trim().to_string(),
                self_closing,
                line: line_at(&starts, i),
            });
            i = end + 1;
            text_start = i;
        } else if rest.starts_with("${") {
            flush(&mut toks, text_start, i);
            let end = find_interp_end(src, i + 2)
                .ok_or_else(|| err(i, "interpolación ${ sin cerrar".into()))?;
            toks.push(Tok::Interp(
                src[i + 2..end].to_string(),
                line_at(&starts, i),
            ));
            i = end + 1;
            text_start = i;
        } else {
            i += rest.chars().next().map(char::len_utf8).unwrap_or(1);
        }
    }
    flush(&mut toks, text_start, b.len());
    Ok(strip_whitespace(toks))
}

/// Recorre un literal de texto y devuelve la posición después de la comilla de cierre.
fn skip_string(b: &[u8], start: usize) -> Option<usize> {
    let quote = b[start];
    let raw = start > 0 && b[start - 1] == b'r';
    let mut j = start + 1;
    while j < b.len() {
        if b[j] == b'\\' && !raw {
            j += 2;
            continue;
        }
        if b[j] == quote {
            return Some(j + 1);
        }
        j += 1;
    }
    None
}

/// Fin de una etiqueta: el `>` fuera de paréntesis, corchetes, llaves y textos.
fn find_tag_end(src: &str, from: usize) -> Option<(usize, bool)> {
    let b = src.as_bytes();
    let mut depth = 0i32;
    let mut j = from;
    while j < b.len() {
        // Comentarios dentro de la etiqueta (FreeMarker los permite entre argumentos).
        if src[j..].starts_with("<#--") {
            j += src[j..].find("-->")? + 3;
            continue;
        }
        match b[j] {
            b'"' | b'\'' => {
                j = skip_string(b, j)?;
                continue;
            }
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b'>' if depth <= 0 => {
                let self_closing = j > from && b[j - 1] == b'/';
                return Some((j, self_closing));
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// `}` que cierra un `${`, saltando llaves internas y textos.
pub fn find_interp_end(src: &str, from: usize) -> Option<usize> {
    let b = src.as_bytes();
    let mut depth = 0i32;
    let mut j = from;
    while j < b.len() {
        match b[j] {
            b'"' | b'\'' => {
                j = skip_string(b, j)?;
                continue;
            }
            b'{' => depth += 1,
            b'}' => {
                if depth == 0 {
                    return Some(j);
                }
                depth -= 1;
            }
            _ => {}
        }
        j += 1;
    }
    None
}

/// Recorte de espacios de FreeMarker: en una línea que solo tiene etiquetas FTL
/// (directivas, llamadas a macros, comentarios) y espacios, se quitan la sangría,
/// los espacios finales y el salto de línea. `<#t>` recorta su propia línea.
fn strip_whitespace(toks: Vec<Tok>) -> Vec<Tok> {
    // 1. Partir los textos en trozos que terminan en salto de línea.
    let mut pieces: Vec<Tok> = Vec::with_capacity(toks.len());
    for t in toks {
        if let Tok::Text(s) = t {
            let mut rest = s.as_str();
            while let Some(nl) = rest.find('\n') {
                pieces.push(Tok::Text(rest[..=nl].to_string()));
                rest = &rest[nl + 1..];
            }
            if !rest.is_empty() {
                pieces.push(Tok::Text(rest.to_string()));
            }
        } else {
            pieces.push(t);
        }
    }

    // 2. Recorrer línea por línea.
    let mut start = 0;
    while start < pieces.len() {
        let mut end = start;
        while end < pieces.len() {
            let ends_line = matches!(&pieces[end], Tok::Text(s) if s.ends_with('\n'));
            end += 1;
            if ends_line {
                break;
            }
        }
        let line = &pieces[start..end];
        let has_tag = line.iter().any(Tok::is_tag);
        let has_trim = line
            .iter()
            .any(|t| matches!(t, Tok::Open { name, .. } if name == "t"));
        let only_ws = line.iter().all(|t| match t {
            Tok::Text(s) => s.chars().all(char::is_whitespace),
            Tok::Interp(..) => false,
            _ => true,
        });
        if has_tag && only_ws {
            for p in &mut pieces[start..end] {
                if let Tok::Text(s) = p {
                    s.clear();
                }
            }
        } else if has_trim {
            if let Some(Tok::Text(s)) = pieces[start..end].first_mut() {
                *s = s.trim_start().to_string();
            }
            if let Some(Tok::Text(s)) = pieces[start..end].last_mut() {
                *s = s.trim_end().to_string();
            }
        }
        start = end;
    }

    // 3. Volver a unir los textos.
    let mut out: Vec<Tok> = Vec::with_capacity(pieces.len());
    for p in pieces {
        match (out.last_mut(), p) {
            (_, Tok::Text(s)) if s.is_empty() => {}
            (Some(Tok::Text(prev)), Tok::Text(s)) => prev.push_str(&s),
            (_, p) => out.push(p),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(src: &str) -> String {
        tokenize(src)
            .ok()
            .unwrap()
            .into_iter()
            .map(|t| match t {
                Tok::Text(s) => s,
                Tok::Interp(e, _) => format!("{{{e}}}"),
                Tok::Open { name, .. } => format!("<{name}>"),
                Tok::Close { name, .. } => format!("</{name}>"),
                Tok::UserOpen { params, .. } => format!("<@{params}>"),
                Tok::UserClose { .. } => "</@>".into(),
                Tok::Comment => "<!>".into(),
            })
            .collect()
    }

    #[test]
    fn lines_with_only_tags_disappear() {
        let src = "a\n  <#if x>\n  b\n  </#if>\nc\n";
        assert_eq!(texts(src), "a\n<if>  b\n</if>c\n");
    }

    #[test]
    fn tags_mixed_with_text_keep_their_line() {
        assert_eq!(texts("x <#if y>z</#if>\n"), "x <if>z</if>\n");
    }

    #[test]
    fn comments_count_as_tags() {
        assert_eq!(texts("a\n<#-- nota -->\nb"), "a\n<!>b");
    }

    #[test]
    fn interpolations_keep_their_line() {
        assert_eq!(texts("  ${x}\n"), "  {x}\n");
    }

    #[test]
    fn greater_than_inside_parentheses_does_not_close_the_tag() {
        let t = tokenize("<#if (a > 1) && xs?filter(e -> e.ok)?size gt 0>ok</#if>")
            .ok()
            .unwrap();
        match &t[0] {
            Tok::Open { params, .. } => {
                assert_eq!(params, "(a > 1) && xs?filter(e -> e.ok)?size gt 0")
            }
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn self_closing_tags_and_strings_with_brackets() {
        let t = tokenize(r#"<@m a=">" b='}'/>${ {"k": "v"}["k"] }"#)
            .ok()
            .unwrap();
        assert!(
            matches!(&t[0], Tok::UserOpen { self_closing: true, params, .. } if params == r#"m a=">" b='}'"#)
        );
        assert!(matches!(&t[1], Tok::Interp(e, _) if e.trim() == r#"{"k": "v"}["k"]"#));
    }

    #[test]
    fn t_trims_its_own_line() {
        assert_eq!(texts("    foo<#t>\nbar"), "foo<t>bar");
    }

    #[test]
    fn reports_the_line_of_an_unclosed_tag() {
        let e = tokenize("a\nb\n<#if x").err().unwrap();
        assert_eq!(e.line, 3);
    }
}
