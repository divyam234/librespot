use bytes::Bytes;
use hex::FromHex;
use http::{Method, Request};
use serde::{Deserialize, Serialize};

use crate::{
    cdn_url::{CdnUrl, MaybeExpiringUrl, MaybeExpiringUrls},
    FileId, Session,
};
#[derive(Debug, Serialize, Deserialize)]
pub struct QobuzUrl {
    pub url: String,
    pub quality: i32,
    pub id: i32,
    pub file_id: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QobuzPayload {
    pub id: String,
    pub title: String,
    pub album: String,
    pub artists: Vec<String>,
    pub duration: u32,
}

pub struct QobuzClient {}

pub struct QobuzResult {
    pub cdn_url: CdnUrl,
    pub quality: i32,
    pub id: i32
}

impl QobuzClient {
    pub async fn get_track(
        &self,
        session: &Session,
        payload: &QobuzPayload,
    ) -> Option<QobuzResult> {
        let client = session.http_client();
        let url = "https://spotify-qobuz.redux.workers.dev/tracks";

        let json_payload = match serde_json::to_string(payload) {
            Ok(s) => s,
            Err(_) => return None,
        };

        let req = match Request::builder()
            .method(Method::POST)
            .uri(url)
            .header("Content-Type", "application/json")
            .body(Bytes::from(json_payload))
        {
            Ok(req) => req,
            Err(_) => return None,
        };

        let response = match client.request_body(req).await {
            Ok(res) => res,
            Err(_) => return None,
        };
        let qobuz_url: QobuzUrl = serde_json::from_slice(response.as_ref()).unwrap();
        let decoded = <[u8; 16]>::from_hex(qobuz_url.file_id).unwrap();
        Some(QobuzResult {
            cdn_url: CdnUrl {
                file_id: FileId::from_raw(&decoded),
                urls: MaybeExpiringUrls(vec![MaybeExpiringUrl(qobuz_url.url, None)]),
            },
            quality: qobuz_url.quality,
            id: qobuz_url.id,
        })
    }
}
