use serde::Deserialize;

#[derive(Deserialize)]
struct Mappings(serde_json::Map<String, serde_json::Value>);

fn extract_to_header(raw: &str) -> Option<String> {
    for line in raw.lines() {
        let l = line.trim_start();
        if l.to_lowercase().starts_with("to:") {
            let after = l[3..].trim();

            let addr = after
                .trim()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .to_string();
            return Some(addr);
        }
        if l.is_empty() {
            break;
        }
    }
    None
}

pub fn route_incoming(raw: &str, mappings_json: Option<&str>) -> Result<String, String> {
    let to_addr = extract_to_header(raw).ok_or_else(|| "To header not found".to_string())?;

    let local = to_addr.split('@').next().unwrap_or("");

    if let Some(json) = mappings_json {
        let v: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("invalid mappings json: {}", e))?;
        if let Some(dest) = v.get(local) {
            if let Some(s) = dest.as_str() {
                return Ok(s.to_string());
            }
        }

        if let Some(pos) = local.find('+') {
            let token = &local[..pos];
            if let Some(dest) = v.get(token) {
                if let Some(s) = dest.as_str() {
                    return Ok(s.to_string());
                }
            }
        }
    }

    Ok(local.to_string())
}

#[rustler::nif]
pub fn route_incoming_nif(raw: String, mappings_json: Option<String>) -> Result<String, String> {
    route_incoming(&raw, mappings_json.as_deref())
}
