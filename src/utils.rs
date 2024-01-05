use anyhow::Context;

pub(crate) fn sha256(s: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    format!("{:x}", hasher.finalize())
}

const MAX_RETRY_SLEEP_DURATION: std::time::Duration = std::time::Duration::from_secs(30);

enum RequestError {
    ClientSideError(anyhow::Error),
    ServerSideError(anyhow::Error),
}

pub(crate) fn http_retry<F: Fn(&reqwest::blocking::Client) -> reqwest::blocking::RequestBuilder>(
    func: F,
) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();

    let mut sleep_duration = std::time::Duration::from_millis(500);

    loop {
        let res = func(&client)
            .send()
            .context("Failed sending HTTP request")
            .map_err(RequestError::ServerSideError)
            .and_then(|resp| {
                let status = resp.status();
                if status.is_success() {
                    Ok(())
                } else {
                    let body = resp
                        .text()
                        .ok()
                        .map(|s| format!(": {s:?}"))
                        .unwrap_or_default();
                    let e = anyhow::format_err!("HTTP Error: {status}{body}");
                    Err(if status.is_client_error() {
                        RequestError::ClientSideError(e)
                    } else {
                        RequestError::ServerSideError(e)
                    })
                }
            });

        match res {
            Ok(_) => break Ok(()),
            Err(RequestError::ServerSideError(e)) => {
                log::error!("Error while sending HTTP request: {e:?}. Going to retry...");
                std::thread::sleep(sleep_duration);
                sleep_duration = (sleep_duration * 2).min(MAX_RETRY_SLEEP_DURATION);
            }
            Err(RequestError::ClientSideError(e)) => {
                anyhow::bail!(e)
            }
        }
    }
}

pub(crate) fn json_set_if_not_present(json: &mut serde_json::Value, path: &[&str], value: &str) {
    if path.is_empty() {
        return;
    }
    let mut parent = json;
    for path_part in path.iter().take(path.len() - 1) {
        let component = &parent[path_part];
        if component.is_null() {
            parent[path_part] = serde_json::json!({});
        } else if !component.is_object() {
            return;
        }

        parent = &mut parent[path_part];
    }

    let last_part = path.last().unwrap();
    if parent[last_part].is_null() {
        parent[last_part] = serde_json::json!(value);
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn test_json_set_if_not_present() {
        use super::json_set_if_not_present;

        let mut value = json!({});

        json_set_if_not_present(&mut value, &["some", "path"], "test");

        assert_eq!(value, json!({"some": {"path": "test"}}));

        let mut value = json!({"some": 2});

        json_set_if_not_present(&mut value, &["some", "path"], "test");

        assert_eq!(value, json!({"some": 2}));

        let mut value = json!({"some": {}});

        json_set_if_not_present(&mut value, &["some", "path"], "test");

        assert_eq!(value, json!({"some": {"path": "test"}}));

        let mut value = json!({"some": {"path": "a"}});

        json_set_if_not_present(&mut value, &["some", "path"], "b");

        assert_eq!(value, json!({"some": {"path": "a"}}));
    }
}
