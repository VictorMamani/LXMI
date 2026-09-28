use crate::{
    cache::{CachedAsset, DownloadCache},
    ErrorCode, OfficialPackageKind, ReleaseAsset, Result, UpstreamRelease, XxmiError,
};
use reqwest::{
    blocking::{Client, Response},
    header::CONTENT_LENGTH,
    redirect::Policy,
    Url,
};
use serde_json::Value;
use std::{
    collections::HashMap,
    io::{Cursor, Read},
    net::IpAddr,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

const ZZMI_REPOSITORY: &str = "leotorrez/ZZMI-Package";
const LIBS_REPOSITORY: &str = "SpectrumQT/XXMI-Libs-Package";
const MAX_METADATA_BYTES: u64 = 1024 * 1024;
const MAX_REDIRECTS: usize = 5;

pub trait ReleaseProvider: Send + Sync {
    fn latest_release(&self, kind: OfficialPackageKind) -> Result<UpstreamRelease>;
    fn release_by_tag(&self, kind: OfficialPackageKind, tag: &str) -> Result<UpstreamRelease>;
    fn download_asset(
        &self,
        asset: &ReleaseAsset,
        release_id: u64,
        cache: &DownloadCache,
        max_bytes: u64,
    ) -> Result<CachedAsset>;
}

/// Deterministic provider for tests and offline application workflows.
#[derive(Debug, Clone, Default)]
pub struct FixtureReleaseProvider {
    releases: Vec<UpstreamRelease>,
    assets: HashMap<u64, Vec<u8>>,
}

impl FixtureReleaseProvider {
    pub fn add_release(&mut self, release: UpstreamRelease) {
        self.releases.push(release);
    }

    pub fn add_asset(&mut self, asset_id: u64, bytes: Vec<u8>) {
        self.assets.insert(asset_id, bytes);
    }
}

impl ReleaseProvider for FixtureReleaseProvider {
    fn latest_release(&self, kind: OfficialPackageKind) -> Result<UpstreamRelease> {
        self.releases
            .iter()
            .filter(|release| release.package_kind == kind)
            .max_by_key(|release| release.release_id)
            .cloned()
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::ReleaseUnavailable,
                    None,
                    "Fixture sin release para este paquete.",
                )
            })
    }

    fn release_by_tag(&self, kind: OfficialPackageKind, tag: &str) -> Result<UpstreamRelease> {
        self.releases
            .iter()
            .find(|release| release.package_kind == kind && release.tag == tag)
            .cloned()
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::ReleaseUnavailable,
                    None,
                    "Fixture sin esa combinación paquete/tag.",
                )
            })
    }

    fn download_asset(
        &self,
        asset: &ReleaseAsset,
        release_id: u64,
        cache: &DownloadCache,
        max_bytes: u64,
    ) -> Result<CachedAsset> {
        let release = self
            .releases
            .iter()
            .find(|release| release.release_id == release_id)
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::ReleaseUnavailable,
                    None,
                    "Fixture asset sin release.",
                )
            })?;
        asset_for_release(release, &asset.name)?;
        let bytes = self
            .assets
            .get(&asset.id)
            .ok_or_else(|| XxmiError::new(ErrorCode::NotFound, None, "Fixture asset ausente."))?;
        let extension = asset
            .name
            .rsplit_once('.')
            .map(|(_, extension)| extension)
            .filter(|extension| matches!(*extension, "zip" | "json"))
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::UnsafePath,
                    None,
                    "Extensión de fixture no permitida.",
                )
            })?;
        let result = cache.store_reader(
            release_id,
            asset.id,
            extension,
            asset.sha256.as_deref(),
            max_bytes,
            &mut Cursor::new(bytes),
        )?;
        if result.size != asset.size {
            return Err(XxmiError::new(
                ErrorCode::ChecksumMismatch,
                Some(result.path),
                "El fixture no coincide con el tamaño declarado.",
            ));
        }
        Ok(result)
    }
}

#[derive(Clone)]
pub struct GitHubReleaseProvider {
    client: Client,
}

impl GitHubReleaseProvider {
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .redirect(Policy::none())
            .build()
            .map_err(network_error)?;
        Ok(Self { client })
    }

    fn api_json(&self, url: Url, limit: u64) -> Result<Vec<u8>> {
        let mut response = self.get(url, true)?;
        bounded_body(&mut response, limit)
    }

    fn get(&self, mut url: Url, api_only: bool) -> Result<Response> {
        for redirects in 0..=MAX_REDIRECTS {
            validate_url(&url, api_only)?;
            let response = self
                .client
                .get(url.clone())
                .header(
                    "User-Agent",
                    "LXMI/0.5.2 (+https://github.com/VictorMamani/LXMI)",
                )
                .header(
                    "Accept",
                    if api_only {
                        "application/vnd.github+json"
                    } else {
                        "application/octet-stream"
                    },
                )
                .send()
                .map_err(network_error)?;
            if response.status().is_redirection() {
                if redirects == MAX_REDIRECTS {
                    return Err(XxmiError::new(
                        ErrorCode::RedirectRejected,
                        None,
                        "Se excedió el límite de redirects.",
                    ));
                }
                let location = response
                    .headers()
                    .get(reqwest::header::LOCATION)
                    .and_then(|value| value.to_str().ok())
                    .ok_or_else(|| {
                        XxmiError::new(
                            ErrorCode::RedirectRejected,
                            None,
                            "Redirect sin Location válido.",
                        )
                    })?;
                url = url.join(location).map_err(|_| {
                    XxmiError::new(
                        ErrorCode::RedirectRejected,
                        None,
                        "URL de redirect inválida.",
                    )
                })?;
                validate_url(&url, false)?;
                continue;
            }
            if !response.status().is_success() {
                return Err(XxmiError::new(
                    ErrorCode::Network,
                    None,
                    format!("GitHub respondió HTTP {}.", response.status()),
                ));
            }
            return Ok(response);
        }
        Err(XxmiError::new(
            ErrorCode::RedirectRejected,
            None,
            "Se excedió el límite de redirects.",
        ))
    }

    fn resolved_release(
        &self,
        kind: OfficialPackageKind,
        endpoint: &str,
    ) -> Result<UpstreamRelease> {
        let repository = repository(&kind);
        let url = Url::parse(&format!(
            "https://api.github.com/repos/{repository}/{endpoint}"
        ))
        .map_err(network_error)?;
        let bytes = self.api_json(url, MAX_METADATA_BYTES)?;
        let value: Value = serde_json::from_slice(&bytes).map_err(|error| {
            XxmiError::new(
                ErrorCode::InvalidMetadata,
                None,
                format!("Metadata de release inválida: {error}"),
            )
        })?;
        let tag = value
            .get("tag_name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                XxmiError::new(ErrorCode::InvalidMetadata, None, "Release sin tag_name.")
            })?;
        validate_tag(tag)?;
        let commit_url = Url::parse(&format!(
            "https://api.github.com/repos/{repository}/commits/{tag}"
        ))
        .map_err(network_error)?;
        let commit_bytes = self.api_json(commit_url, MAX_METADATA_BYTES)?;
        let commit_json: Value = serde_json::from_slice(&commit_bytes).map_err(|error| {
            XxmiError::new(
                ErrorCode::InvalidMetadata,
                None,
                format!("Metadata del commit inválida: {error}"),
            )
        })?;
        let commit = commit_json
            .get("sha")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                XxmiError::new(
                    ErrorCode::InvalidMetadata,
                    None,
                    "No se resolvió el commit del tag.",
                )
            })?;
        parse_github_release(&kind, &bytes, commit, unix_seconds()?)
    }

    pub fn download_asset(
        &self,
        asset: &ReleaseAsset,
        release_id: u64,
        cache: &DownloadCache,
        max_bytes: u64,
    ) -> Result<CachedAsset> {
        let mut response = self.get(
            Url::parse(&asset.download_url).map_err(network_error)?,
            false,
        )?;
        if let Some(length) = response
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
        {
            if length > max_bytes || (asset.size > 0 && length != asset.size) {
                return Err(XxmiError::new(
                    ErrorCode::LimitExceeded,
                    None,
                    "Content-Length del asset excede el límite o difiere de GitHub API.",
                ));
            }
        }
        let extension = match asset.name.rsplit_once('.') {
            Some((_, extension @ ("zip" | "json"))) => extension,
            _ => {
                return Err(XxmiError::new(
                    ErrorCode::UnsafePath,
                    None,
                    "Extensión de release asset no permitida.",
                ))
            }
        };
        let digest = asset.sha256.as_deref();
        let cached = cache.store_reader(
            release_id,
            asset.id,
            extension,
            digest,
            max_bytes,
            &mut response,
        )?;
        if cached.size != asset.size {
            return Err(XxmiError::new(
                ErrorCode::ChecksumMismatch,
                Some(cached.path),
                "El tamaño descargado difiere del tamaño publicado por GitHub.",
            ));
        }
        Ok(cached)
    }
}

impl ReleaseProvider for GitHubReleaseProvider {
    fn latest_release(&self, kind: OfficialPackageKind) -> Result<UpstreamRelease> {
        self.resolved_release(kind, "releases/latest")
    }

    fn release_by_tag(&self, kind: OfficialPackageKind, tag: &str) -> Result<UpstreamRelease> {
        validate_tag(tag)?;
        self.resolved_release(kind, &format!("releases/tags/{tag}"))
    }

    fn download_asset(
        &self,
        asset: &ReleaseAsset,
        release_id: u64,
        cache: &DownloadCache,
        max_bytes: u64,
    ) -> Result<CachedAsset> {
        GitHubReleaseProvider::download_asset(self, asset, release_id, cache, max_bytes)
    }
}

pub fn repository(kind: &OfficialPackageKind) -> &'static str {
    match kind {
        OfficialPackageKind::Zzmi => ZZMI_REPOSITORY,
        OfficialPackageKind::XxmiLibraries => LIBS_REPOSITORY,
    }
}

pub fn expected_asset_name(kind: &OfficialPackageKind, tag: &str) -> Result<&'static str> {
    validate_tag(tag)?;
    // Only the currently recognized release asset patterns are accepted. The pattern itself is
    // checked against the upstream launcher source and never supplied by the renderer.
    match kind {
        OfficialPackageKind::Zzmi => Ok("ZZMI-PACKAGE"),
        OfficialPackageKind::XxmiLibraries => Ok("XXMI-PACKAGE"),
    }
}

pub fn parse_github_release(
    kind: &OfficialPackageKind,
    bytes: &[u8],
    resolved_commit: &str,
    retrieved_at: u64,
) -> Result<UpstreamRelease> {
    let json: Value = serde_json::from_slice(bytes).map_err(|error| {
        XxmiError::new(
            ErrorCode::InvalidMetadata,
            None,
            format!("JSON de GitHub inválido: {error}"),
        )
    })?;
    if json.get("draft").and_then(Value::as_bool).unwrap_or(true)
        || json
            .get("prerelease")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    {
        return Err(XxmiError::new(
            ErrorCode::ReleaseUnavailable,
            None,
            "Las releases draft/prerelease no se consideran candidatas oficiales.",
        ));
    }
    let tag = required_string(&json, "tag_name")?;
    validate_tag(tag)?;
    let release_id = json.get("id").and_then(Value::as_u64).ok_or_else(|| {
        XxmiError::new(ErrorCode::InvalidMetadata, None, "Release sin ID numérico.")
    })?;
    let release_url = required_string(&json, "html_url")?.to_owned();
    let expected_url = format!("https://github.com/{}/releases/tag/{tag}", repository(kind));
    if release_url != expected_url {
        return Err(XxmiError::new(
            ErrorCode::ReleaseUnavailable,
            None,
            "La URL de release no coincide con el repositorio oficial permitido.",
        ));
    }
    let published_at = required_string(&json, "published_at")?.to_owned();
    let version = tag.strip_prefix('v').unwrap_or(tag).to_owned();
    let expected_stem = expected_asset_name(kind, tag)?;
    let assets_value = json
        .get("assets")
        .and_then(Value::as_array)
        .ok_or_else(|| XxmiError::new(ErrorCode::InvalidMetadata, None, "Release sin assets."))?;
    let mut assets = Vec::new();
    for value in assets_value {
        let asset = ReleaseAsset {
            id: value
                .get("id")
                .and_then(Value::as_u64)
                .ok_or_else(|| XxmiError::new(ErrorCode::InvalidMetadata, None, "Asset sin ID."))?,
            name: required_string(value, "name")?.to_owned(),
            download_url: required_string(value, "browser_download_url")?.to_owned(),
            size: value.get("size").and_then(Value::as_u64).ok_or_else(|| {
                XxmiError::new(ErrorCode::InvalidMetadata, None, "Asset sin size.")
            })?,
            content_type: value
                .get("content_type")
                .and_then(Value::as_str)
                .map(str::to_owned),
            sha256: value
                .get("digest")
                .and_then(Value::as_str)
                .and_then(|digest| digest.strip_prefix("sha256:"))
                .map(str::to_owned),
        };
        let expected_name = format!("{expected_stem}-v{version}.zip");
        let companion =
            kind == &OfficialPackageKind::XxmiLibraries && asset.name == "Manifest.json";
        if asset.name == expected_name || companion {
            let url = Url::parse(&asset.download_url).map_err(network_error)?;
            validate_url(&url, false)?;
            if url.host_str() != Some("github.com")
                || url.path()
                    != format!(
                        "/{}/releases/download/{tag}/{}",
                        repository(kind),
                        asset.name
                    )
                || asset.size == 0
                || asset.size
                    > if companion {
                        64 * 1024
                    } else {
                        32 * 1024 * 1024
                    }
            {
                return Err(XxmiError::new(
                    ErrorCode::ReleaseUnavailable,
                    None,
                    "Asset no coincide con el nombre/ruta/tamaño oficial esperado.",
                ));
            }
            assets.push(asset);
        }
    }
    let zip_name = format!("{expected_stem}-v{version}.zip");
    if assets.iter().filter(|asset| asset.name == zip_name).count() != 1
        || (kind == &OfficialPackageKind::XxmiLibraries
            && assets
                .iter()
                .filter(|asset| asset.name == "Manifest.json")
                .count()
                != 1)
    {
        return Err(XxmiError::new(
            ErrorCode::ReleaseUnavailable,
            None,
            "Release no contiene exactamente los assets oficiales requeridos.",
        ));
    }
    let body = json.get("body").and_then(Value::as_str).unwrap_or_default();
    let signature_base64 = extract_signature(body);
    Ok(UpstreamRelease {
        package_kind: kind.clone(),
        repository: repository(kind).to_owned(),
        release_id,
        tag: tag.to_owned(),
        commit: resolved_commit.to_owned(),
        release_url,
        version,
        published_at,
        metadata_retrieved_at: format_utc(retrieved_at),
        signature_base64,
        assets,
        source_trust: crate::SourceTrust::Official,
    })
}

pub fn asset_for_release<'a>(release: &'a UpstreamRelease, name: &str) -> Result<&'a ReleaseAsset> {
    if release.repository != repository(&release.package_kind) {
        return Err(XxmiError::new(
            ErrorCode::ReleaseUnavailable,
            None,
            "Repositorio fuera de allowlist oficial.",
        ));
    }
    release
        .assets
        .iter()
        .find(|asset| asset.name == name)
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::ReleaseUnavailable,
                None,
                "Asset no pertenece a la metadata seleccionada.",
            )
        })
}

pub fn download_and_import_official(
    store: &crate::ManagedStore,
    cache: &DownloadCache,
    provider: &dyn ReleaseProvider,
    kind: OfficialPackageKind,
    tag: &str,
) -> Result<crate::PackageManifest> {
    let release = provider.release_by_tag(kind.clone(), tag)?;
    if release.package_kind != kind
        || release.source_trust != crate::SourceTrust::Official
        || release.repository != repository(&kind)
    {
        return Err(XxmiError::new(
            ErrorCode::ReleaseUnavailable,
            None,
            "El provider devolvió una fuente no autorizada.",
        ));
    }
    let zip_name = format!(
        "{}-v{}.zip",
        expected_asset_name(&kind, &release.tag)?,
        release.version
    );
    let zip_asset = asset_for_release(&release, &zip_name)?.clone();
    if release.signature_base64.is_none() {
        return Err(XxmiError::new(
            ErrorCode::MissingSignature,
            None,
            "La release oficial no publica una firma del asset ZIP.",
        ));
    }
    let archive =
        provider.download_asset(&zip_asset, release.release_id, cache, 32 * 1024 * 1024)?;
    let companion = if kind == OfficialPackageKind::XxmiLibraries {
        let metadata = asset_for_release(&release, "Manifest.json")?.clone();
        let downloaded =
            provider.download_asset(&metadata, release.release_id, cache, 64 * 1024)?;
        Some((metadata, downloaded.path))
    } else {
        None
    };
    let manifest = store.import_official_archive(
        &release,
        &archive.path,
        companion
            .as_ref()
            .map(|(asset, path)| (asset, path.as_path())),
    )?;
    Ok(manifest.manifest().clone())
}

fn extract_signature(body: &str) -> Option<String> {
    let section = body.split("## Signature").nth(1)?;
    section
        .lines()
        .map(str::trim)
        .filter_map(|line| {
            line.strip_prefix("- ")
                .or_else(|| line.strip_prefix('*'))
                .map(str::trim)
        })
        .find(|line| {
            !line.is_empty()
                && line
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='))
        })
        .map(str::to_owned)
}

fn required_string<'a>(value: &'a Value, name: &str) -> Result<&'a str> {
    value
        .get(name)
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
        .ok_or_else(|| {
            XxmiError::new(
                ErrorCode::InvalidMetadata,
                None,
                format!("Metadata sin {name}."),
            )
        })
}

fn validate_tag(tag: &str) -> Result<()> {
    if tag.len() > 80
        || !tag.starts_with('v')
        || !tag
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'_' | b'+'))
    {
        return Err(XxmiError::new(
            ErrorCode::UnsafePath,
            None,
            "Tag de release inválido.",
        ));
    }
    Ok(())
}

fn validate_url(url: &Url, api_only: bool) -> Result<()> {
    let host = url.host_str().unwrap_or_default();
    let allowed = if api_only {
        host == "api.github.com"
    } else {
        matches!(
            host,
            "github.com"
                | "release-assets.githubusercontent.com"
                | "objects.githubusercontent.com"
                | "github-releases.githubusercontent.com"
        )
    };
    if url.scheme() != "https"
        || !allowed
        || url.username() != ""
        || url.password().is_some()
        || url.port().is_some_and(|port| port != 443)
        || host.parse::<IpAddr>().is_ok()
    {
        return Err(XxmiError::new(
            ErrorCode::RedirectRejected,
            None,
            "URL fuera del allowlist HTTPS de GitHub.",
        ));
    }
    Ok(())
}

fn bounded_body(response: &mut Response, limit: u64) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > limit)
    {
        return Err(XxmiError::new(
            ErrorCode::LimitExceeded,
            None,
            "Respuesta de metadata demasiado grande.",
        ));
    }
    let mut bytes = Vec::new();
    response
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(network_error)?;
    if bytes.len() as u64 > limit {
        return Err(XxmiError::new(
            ErrorCode::LimitExceeded,
            None,
            "Respuesta de metadata excede el límite.",
        ));
    }
    Ok(bytes)
}

fn network_error(error: impl std::fmt::Display) -> XxmiError {
    XxmiError::new(ErrorCode::Network, None, error.to_string())
}

fn unix_seconds() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|_| XxmiError::new(ErrorCode::Io, None, "Reloj anterior a Unix epoch."))
}

pub(crate) fn format_utc(timestamp: u64) -> String {
    let days = (timestamp / 86_400) as i64;
    let day_seconds = timestamp % 86_400;
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3600,
        (day_seconds % 3600) / 60,
        day_seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_sources_are_fixed_and_tag_paths_are_not_user_controlled() {
        assert_eq!(repository(&OfficialPackageKind::Zzmi), ZZMI_REPOSITORY);
        assert_eq!(
            repository(&OfficialPackageKind::XxmiLibraries),
            LIBS_REPOSITORY
        );
        assert!(validate_tag("v1.4.5").is_ok());
        assert!(validate_tag("../../attacker").is_err());
        assert!(validate_tag("v1.0/other").is_err());
    }

    #[test]
    fn timestamp_is_stored_as_reproducible_utc() {
        assert_eq!(format_utc(1_790_590_156), "2026-09-28T10:09:16Z");
    }

    fn fixture(tag: &str, url: &str, body: &str) -> Vec<u8> {
        serde_json::to_vec(&serde_json::json!({
            "id": 1234,
            "tag_name": tag,
            "draft": false,
            "prerelease": false,
            "html_url": url,
            "published_at": "2026-09-01T00:00:00Z",
            "body": body,
            "assets": [{
                "id": 987,
                "name": "ZZMI-PACKAGE-v1.4.5.zip",
                "browser_download_url": "https://github.com/leotorrez/ZZMI-Package/releases/download/v1.4.5/ZZMI-PACKAGE-v1.4.5.zip",
                "size": 543066,
                "content_type": "application/zip",
                "digest": "sha256:a2d203e2a09428f3b1999b1b317fd37b9889ad934f383db901e47e66d8011fd4"
            }]
        })).unwrap()
    }

    #[test]
    fn parses_pinned_official_release_identity_and_signature_metadata() {
        let bytes = fixture(
            "v1.4.5",
            "https://github.com/leotorrez/ZZMI-Package/releases/tag/v1.4.5",
            "## Signature\n- AQIDBA==",
        );
        let release = parse_github_release(
            &OfficialPackageKind::Zzmi,
            &bytes,
            "752475bed3a57f14ccf9c72645d35a798942ae6e",
            1_790_590_156,
        )
        .unwrap();
        assert_eq!(release.release_id, 1234);
        assert_eq!(release.tag, "v1.4.5");
        assert_eq!(release.version, "1.4.5");
        assert_eq!(release.signature_base64.as_deref(), Some("AQIDBA=="));
        assert_eq!(release.source_trust, crate::SourceTrust::Official);
        assert_eq!(
            release.assets[0].sha256.as_deref(),
            Some("a2d203e2a09428f3b1999b1b317fd37b9889ad934f383db901e47e66d8011fd4")
        );
    }

    #[test]
    fn release_url_and_asset_names_cannot_escape_the_official_repository() {
        let bytes = fixture(
            "v1.4.5",
            "https://github.com/attacker/ZZMI-Package/releases/tag/v1.4.5",
            "## Signature\n- AQIDBA==",
        );
        assert!(parse_github_release(&OfficialPackageKind::Zzmi, &bytes, "abc", 0).is_err());
    }

    #[test]
    fn fixture_release_provider_resolves_metadata_without_network() {
        let bytes = fixture(
            "v1.4.5",
            "https://github.com/leotorrez/ZZMI-Package/releases/tag/v1.4.5",
            "## Signature\n- AQIDBA==",
        );
        let release = parse_github_release(
            &OfficialPackageKind::Zzmi,
            &bytes,
            "752475bed3a57f14ccf9c72645d35a798942ae6e",
            1_790_590_156,
        )
        .unwrap();
        let mut provider = FixtureReleaseProvider::default();
        provider.add_release(release.clone());
        assert_eq!(
            provider.latest_release(OfficialPackageKind::Zzmi).unwrap(),
            release
        );
        assert_eq!(
            provider
                .release_by_tag(OfficialPackageKind::Zzmi, "v1.4.5")
                .unwrap()
                .release_id,
            1234
        );
        assert!(provider
            .latest_release(OfficialPackageKind::XxmiLibraries)
            .is_err());
    }
}
