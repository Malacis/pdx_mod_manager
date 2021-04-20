//! Network functionality.

use std::{collections::HashMap, thread, time::Duration};

use anyhow::Result;
use bytes::Bytes;
use reqwest::Client;
use serde::Deserialize;
use serde::Serialize;

/// This struct saves the client for network operations so we don't have to recreate it for every function.
pub struct Remote {
    /// The `reqwest::Client`.
    client: Option<Client>,
}

impl Remote {
    /// Instanciates a new `Remote`struct.
    pub const fn new() -> Self {
        Self { client: None }
    }

    /// Creates a `reqwest::Client` and saves it in the struct if none exists yet.
    fn client(&mut self) -> &Client {
        #[allow(clippy::option_if_let_else)]
        if let Some(ref client) = self.client {
            client
        } else {
            self.client = Some(Client::new());
            self.client.as_ref().expect("value just inserted missing")
        }
    }

    /// Gets the mod info from the steam worshop API.
    pub async fn get_item_info(&mut self, item_id: u64) -> Result<(String, u64)> {
        /// Struct for deserializing response from the steam workshop API.
        #[derive(Debug, Deserialize)]
        pub struct WorkshopItemInfoResponseList {
            /// Struct for deserializing response from the steam workshop API.
            response: WorkshopItemInfoResponse,
        }

        /// Struct for deserializing response from the steam workshop API.
        #[derive(Debug, Deserialize)]
        pub struct WorkshopItemInfoResponse {
            /// Struct for deserializing response from the steam workshop API.
            publishedfiledetails: Vec<WorkshopItemInfo>,
        }

        /// Struct for deserializing response from the steam workshop API.
        #[derive(Debug, Deserialize)]
        pub struct WorkshopItemInfo {
            /// Title of the requested mod.
            pub title: String,
            /// Last update time of the requested mod as unix timestamp.
            pub time_updated: u64,
        }

        let client = self.client();

        let item_info = client
            .post("https://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v1/")
            .form(&[
                ("itemcount", "1"),
                ("publishedfileids[0]", &item_id.to_string()),
            ])
            .send()
            .await?
            .json::<WorkshopItemInfoResponseList>()
            .await?;

        let title = &item_info
            .response
            .publishedfiledetails
            .get(0)
            .expect("workshop response empty")
            .title;
        let time_updated = item_info
            .response
            .publishedfiledetails
            .get(0)
            .expect("workshop response empty")
            .time_updated;
        Ok((title.to_string(), time_updated))
    }

    /// Downloads mods from steamworkshopdownloader.io
    pub async fn download_item(&mut self, item_id: u64) -> Result<Bytes> {
        /// Used to serialize the initial request.
        #[allow(clippy::struct_excessive_bools, clippy::missing_docs_in_private_items)]
        #[derive(Debug, Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RequestBody {
            published_file_id: u64,
            collection_id: Option<u64>,
            extract: bool,
            hidden: bool,
            direct: bool,
            autodownload: bool,
        }

        /// Used to deserialize the initial response.
        #[derive(Debug, Deserialize)]
        struct RequestResponse {
            /// This is used in the status and download request.
            uuid: String,
        }

        /// Used to serialize subsequent requests.
        #[derive(Debug, Serialize)]
        struct Uuids {
            /// This is used in status and download request.
            uuids: Vec<String>,
        }

        /// Used to deserialize status responses.
        #[derive(Debug, Deserialize)]
        struct StatusResponse {
            /// The status of the requested file.
            status: String,
        }

        let client = self.client();
        println!("Requesting download via steamworkshopdownloader.io");

        let request_body = RequestBody {
            published_file_id: item_id,
            collection_id: None,
            extract: true,
            hidden: false,
            direct: false,
            autodownload: false,
        };

        let download_request_response = client
            .post("https://backend-01-prd.steamworkshopdownloader.io/api/download/request")
            .body(serde_json::to_string(&request_body)?)
            .send()
            .await?
            .json::<RequestResponse>()
            .await?;

        let download_link = format!(
            "https://backend-01-prd.steamworkshopdownloader.io/api/download/transmit?uuid={}",
            download_request_response.uuid
        );

        println!("Waiting for file to be ready.");

        let status_request_body = Uuids {
            uuids: vec![download_request_response.uuid.clone()],
        };

        loop {
            let status_response = client
                .post("https://backend-01-prd.steamworkshopdownloader.io/api/download/status")
                .body(serde_json::to_string(&status_request_body)?)
                .send()
                .await?
                .json::<HashMap<String, StatusResponse>>()
                .await?;

            if status_response
                .get(status_request_body.uuids.get(0).expect("no uuid in response"))
                .expect("Uuid incorrect")
                .status
                == "prepared"
            {
                break;
            }
            println!("...");
            thread::sleep(Duration::from_secs(1));
        }

        println!("File ready, downloading now!");
        Ok(client.get(download_link).send().await?.bytes().await?)
    }
}
