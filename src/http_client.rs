use reqwest::{Client, Method, Proxy, RequestBuilder};
use url::Url;

use crate::{
    config::{ProxyConfig, ProxyMode},
    error::AppResult,
};

#[derive(Clone)]
pub struct ProxyAwareHttpClient {
    direct: Client,
    proxied: Client,
    proxy_mode: ProxyMode,
    no_proxy: Vec<String>,
}

impl ProxyAwareHttpClient {
    pub fn new(timeout_ms: u64, proxy: &ProxyConfig) -> AppResult<Self> {
        let direct = Client::builder()
            .timeout(std::time::Duration::from_millis(timeout_ms))
            .no_proxy()
            .build()?;

        let proxied = match proxy.mode {
            ProxyMode::System => Client::builder()
                .timeout(std::time::Duration::from_millis(timeout_ms))
                .build()?,
            ProxyMode::Direct => direct.clone(),
            ProxyMode::Custom => {
                let mut builder = Client::builder()
                    .timeout(std::time::Duration::from_millis(timeout_ms))
                    .no_proxy();
                if let Some(value) = &proxy.all_proxy {
                    builder = builder.proxy(Proxy::all(value)?);
                }
                if let Some(value) = &proxy.http_proxy {
                    builder = builder.proxy(Proxy::http(value)?);
                }
                if let Some(value) = &proxy.https_proxy {
                    builder = builder.proxy(Proxy::https(value)?);
                }
                builder.build()?
            }
        };

        Ok(Self {
            direct,
            proxied,
            proxy_mode: proxy.mode.clone(),
            no_proxy: proxy.no_proxy.clone(),
        })
    }

    pub fn request(&self, method: Method, url: &str) -> AppResult<RequestBuilder> {
        let client = self.client_for(url)?;
        Ok(client.request(method, url))
    }

    fn client_for(&self, url: &str) -> AppResult<&Client> {
        let parsed = Url::parse(url)?;
        if self.proxy_mode == ProxyMode::Direct {
            return Ok(&self.direct);
        }

        if self.proxy_mode == ProxyMode::Custom {
            let host = parsed.host_str().unwrap_or_default();
            if self
                .no_proxy
                .iter()
                .any(|rule| host_matches_no_proxy(host, rule))
            {
                return Ok(&self.direct);
            }
        }

        Ok(&self.proxied)
    }
}

fn host_matches_no_proxy(host: &str, rule: &str) -> bool {
    let normalized_rule = rule.trim();
    if normalized_rule == "*" || normalized_rule == host {
        return true;
    }

    if let Some(suffix) = normalized_rule.strip_prefix('.') {
        return host == suffix || host.ends_with(&format!(".{suffix}"));
    }

    host == normalized_rule || host.ends_with(&format!(".{normalized_rule}"))
}

#[cfg(test)]
mod tests {
    use super::host_matches_no_proxy;

    #[test]
    fn no_proxy_rules_match_expected_hosts() {
        assert!(host_matches_no_proxy("localhost", "localhost"));
        assert!(host_matches_no_proxy("api.internal", ".internal"));
        assert!(host_matches_no_proxy("foo.example.com", "example.com"));
        assert!(!host_matches_no_proxy("foo.example.com", "bar.com"));
    }
}
