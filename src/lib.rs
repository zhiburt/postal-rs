//! The crate provides an interface to [Postal]'s http [API].
//!
//! # Examples
//!
//! ```no_run
//! use postal_rs::{Client, DetailsInterest, Message, SendResult};
//! use std::env;
//! 
//! #[tokio::main]
//! async fn main() {
//!    let address = env::var("POSTAL_ADDRESS").unwrap_or_default();
//!    let token = env::var("POSTAL_TOKEN").unwrap_or_default();
//!
//!    let message = Message::default()
//!        .to(&["example@gmail.com".to_owned()])
//!        .from("test@yourserver.io")
//!        .subject("Hello World")
//!        .text("A test message");
//!    let client = Client::new(address, token).unwrap();
//!    let _ = client
//!        .send(message)
//!        .await
//!        .unwrap();
//!}
//!
//! ```
//!
//! [Postal]: https://postal.atech.media/
//! [API]: https://github.com/postalhq/postal/wiki/Using-the-API

mod error;

pub use error::PostalError;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::Value as Json;
use std::collections::HashMap;
use url::Url;

/// Client holds a session information
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Client {
    address: Url,
    token: String,
}

impl Client {
    /// Constructs a new instance of client
    pub fn new<U, S>(url: U, token: S) -> Result<Self, PostalError>
    where
        U: AsRef<str>,
        S: Into<String>,
    {
        let url = Url::parse(url.as_ref())?;
        let token = token.into();

        Ok(Self {
            address: url,
            token,
        })
    }

    /// Sends a message to Postal
    pub async fn send<M: Into<Message>>(&self, message: M) -> Result<Vec<SendResult>, PostalError> {
        let address = self.address.join("/api/v1/send/message")?;
        let message = message.into();

        let client = reqwest::Client::new();
        let res = client
            .post(address)
            .json(&message)
            .header("X-Server-API-Key", &self.token)
            .send()
            .await?;

        handle_send(res).await
    }

    /// Sends a standart SMTP message to Postal
    pub async fn send_raw<M: Into<RawMessage>>(
        &self,
        message: M,
    ) -> Result<Vec<SendResult>, PostalError> {
        let address = self.address.join("/api/v1/send/raw")?;
        let message = message.into();

        let client = reqwest::Client::new();
        let res = client
            .post(address)
            .json(&message)
            .header("X-Server-API-Key", &self.token)
            .send()
            .await?;

        handle_send(res).await
    }

    /// Asks a Postal server to provide an information details
    /// about a message
    ///
    /// By default it provides a limited information.
    /// To increase this volume you can specify expansions via [DetailsInterest]
    ///
    /// [DetailsInterest]: ./struct.DetailsInterest.html
    pub async fn get_message_details<I: Into<DetailsInterest>>(
        &self,
        interest: I,
    ) -> Result<HashMap<String, Json>, PostalError> {
        let interest = interest.into();
        let address = self.address.join("/api/v1/messages/message")?;

        let client = reqwest::Client::new();
        let body: Json = interest.into();
        let res = client
            .post(address)
            .json(&body)
            .header("X-Server-API-Key", &self.token)
            .send()
            .await?;

        check_status(res.status())?;

        let data: api_structures::Responce<HashMap<String, Json>> = res.json().await?;
        let data = check_responce(data)?;

        Ok(data)
    }

    /// Obtains a delivery information according to a message.
    pub async fn get_message_deliveries(
        &self,
        id: MessageHash,
    ) -> Result<Vec<HashMap<String, Json>>, PostalError> {
        let address = self.address.join("/api/v1/messages/deliveries")?;

        let client = reqwest::Client::new();
        let body: Json = serde_json::json!({ "id": id });
        let res = client
            .post(address)
            .json(&body)
            .header("X-Server-API-Key", &self.token)
            .send()
            .await?;

        check_status(res.status())?;

        let data: api_structures::Responce<Vec<HashMap<String, Json>>> = res.json().await?;
        let data = check_responce(data)?;

        Ok(data)
    }
}

async fn handle_send(resp: reqwest::Response) -> Result<Vec<SendResult>, PostalError> {
    check_status(resp.status())?;

    let data: api_structures::Responce<api_structures::MessageSucessData> = resp.json().await?;
    let data = check_responce(data)?;

    let messages = data
        .messages
        .into_iter()
        .map(|(to, m)| SendResult { to, id: m.id })
        .collect();

    Ok(messages)
}

fn check_status(sc: StatusCode) -> Result<(), PostalError> {
    match sc {
        StatusCode::OK => Ok(()),
        StatusCode::INTERNAL_SERVER_ERROR => Err(PostalError::InternalServerError),
        StatusCode::MOVED_PERMANENTLY | StatusCode::PERMANENT_REDIRECT => {
            Err(PostalError::ExpectedAlternativeUrl)
        }
        StatusCode::SERVICE_UNAVAILABLE => Err(PostalError::ServiceUnavailableError),
        // according to postal docs it's imposible to get a different status code
        // https://krystal.github.io/postal-api/index.html
        _ => unreachable!(),
    }
}

fn check_responce<T>(data: api_structures::Responce<T>) -> Result<T, PostalError> {
    match data {
        api_structures::Responce::Success { data, .. } => Ok(data),
        api_structures::Responce::Error { data, .. } => Err(PostalError::Error {
            code: data.code,
            message: data.message,
        }),
        // the format of this error is unclear
        api_structures::Responce::ParameterError {} => unimplemented!(),
    }
}

/// MessageHash represents a hash which can be used to
/// get a different information bout a message.
pub type MessageHash = u64;

/// Message represents a email which can be sent
#[derive(Debug, Eq, PartialEq, Clone, Default, Deserialize, Serialize)]
pub struct Message {
    ///The e-mail addresses of the recipients (max 50)
    pub to: Option<Vec<String>>,
    /// The e-mail addresses of any CC contacts (max 50)
    pub cc: Option<Vec<String>>,
    /// The e-mail addresses of any BCC contacts (max 50)
    pub bcc: Option<Vec<String>>,
    /// The e-mail address for the From header
    pub from: Option<String>,
    /// The e-mail address for the Sender header
    pub sender: Option<String>,
    /// The subject of the e-mail
    pub subject: Option<String>,
    /// The tag of the e-mail
    pub tag: Option<String>,
    /// Set the reply-to address for the mail
    pub reply_to: Option<String>,
    /// The plain text body of the e-mail
    pub plain_body: Option<String>,
    /// The HTML body of the e-mail
    pub html_body: Option<String>,
    /// An array of attachments for this e-mail
    pub attachments: Option<Vec<Vec<u8>>>,
    /// A hash of additional headers
    pub headers: Option<MessageHash>,
    /// Is this message a bounce?
    pub bounce: Option<bool>,
}

impl Message {
    pub fn from<S: Into<String>>(mut self, s: S) -> Self {
        self.from = Some(s.into());
        self
    }

    pub fn to(mut self, to: &[String]) -> Self {
        self.to = Some(to.to_vec());
        self
    }

    pub fn subject<S: Into<String>>(mut self, s: S) -> Self {
        self.subject = Some(s.into());
        self
    }

    pub fn text<S: Into<String>>(mut self, s: S) -> Self {
        self.plain_body = Some(s.into());
        self
    }

    pub fn html<S: Into<String>>(mut self, s: S) -> Self {
        self.html_body = Some(s.into());
        self
    }
}

/// RawMessage allows you to send us a raw RFC2822 formatted message along with
/// the recipients that it should be sent to.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct RawMessage {
    /// The address that should be logged as sending the message
    pub mail_from: String,
    /// The addresses this message should be sent to
    pub rcpt_to: Vec<String>,
    /// A base64 encoded RFC2822 message to send
    pub data: String,
    /// Is this message a bounce?
    pub bounce: Option<bool>,
}

impl RawMessage {
    pub fn new<S1: Into<String>, S2: Into<String>>(to: &[String], from: S1, data: S2) -> Self {
        Self {
            rcpt_to: to.to_owned(),
            mail_from: from.into(),
            data: data.into(),
            bounce: None,
        }
    }
}

/// DetailsInterest contains an options which can be used to
/// turn on expansions while obtaining details of a message.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
pub struct DetailsInterest {
    id: MessageHash,
    status: Option<()>,
    details: Option<()>,
    inspection: Option<()>,
    plain_body: Option<()>,
    html_body: Option<()>,
    attachments: Option<()>,
    headers: Option<()>,
    raw_message: Option<()>,
}

impl DetailsInterest {
    pub fn new(id: MessageHash) -> Self {
        id.into()
    }

    pub fn with_status(mut self) -> Self {
        self.status = Some(());
        self
    }

    pub fn with_details(mut self) -> Self {
        self.details = Some(());
        self
    }

    pub fn with_inspection(mut self) -> Self {
        self.inspection = Some(());
        self
    }

    pub fn with_plain_body(mut self) -> Self {
        self.plain_body = Some(());
        self
    }

    pub fn with_html_body(mut self) -> Self {
        self.html_body = Some(());
        self
    }

    pub fn with_headers(mut self) -> Self {
        self.headers = Some(());
        self
    }

    pub fn with_raw_message(mut self) -> Self {
        self.raw_message = Some(());
        self
    }

    fn build_expansions_list(&self) -> Option<Vec<Json>> {
        let mut expansions: Option<Vec<Json>> = None;
        if self.status.is_some() {
            expansions = Some(expansions.unwrap_or_default());
            expansions
                .as_mut()
                .unwrap()
                .push(Json::String("status".to_owned()));
        }
        if self.details.is_some() {
            expansions = Some(expansions.unwrap_or_default());
            expansions
                .as_mut()
                .unwrap()
                .push(Json::String("details".to_owned()));
        }
        if self.inspection.is_some() {
            expansions = Some(expansions.unwrap_or_default());
            expansions
                .as_mut()
                .unwrap()
                .push(Json::String("inspection".to_owned()));
        }
        if self.plain_body.is_some() {
            expansions = Some(expansions.unwrap_or_default());
            expansions
                .as_mut()
                .unwrap()
                .push(Json::String("plain_body".to_owned()));
        }
        if self.html_body.is_some() {
            expansions = Some(expansions.unwrap_or_default());
            expansions
                .as_mut()
                .unwrap()
                .push(Json::String("html_body".to_owned()));
        }
        if self.headers.is_some() {
            expansions = Some(expansions.unwrap_or_default());
            expansions
                .as_mut()
                .unwrap()
                .push(Json::String("headers".to_owned()));
        }
        if self.raw_message.is_some() {
            expansions = Some(expansions.unwrap_or_default());
            expansions
                .as_mut()
                .unwrap()
                .push(Json::String("raw_message".to_owned()));
        }

        expansions
    }
}

impl Into<DetailsInterest> for MessageHash {
    fn into(self) -> DetailsInterest {
        DetailsInterest {
            id: self,
            status: None,
            details: None,
            inspection: None,
            plain_body: None,
            html_body: None,
            attachments: None,
            headers: None,
            raw_message: None,
        }
    }
}

impl Into<Json> for DetailsInterest {
    fn into(self) -> Json {
        let mut map: HashMap<String, Json> = HashMap::new();
        map.insert("id".to_owned(), self.id.into());

        let expansions = self.build_expansions_list();
        if let Some(expansions) = expansions {
            map.insert("_expansions".to_owned(), Json::Array(expansions));
        }

        serde_json::json!(map)
    }
}

/// SendResult represent a result of sending request
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct SendResult {
    /// An email To which an email was sent
    pub to: String,
    /// A message id which can be used to retrieve message details
    /// and message deliveries
    pub id: MessageHash,
}

mod api_structures {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(tag = "status", rename_all = "camelCase")]
    pub enum Responce<D> {
        Success {
            time: f64,
            flags: HashMap<String, u64>,
            data: D,
        },
        ParameterError {},
        Error {
            time: f64,
            flags: HashMap<String, u64>,
            data: ResponceError,
        },
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MessageSucessData {
        pub message_id: String,
        pub messages: HashMap<String, MessageDataTo>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct MessageDataTo {
        pub id: u64,
        pub token: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ResponceError {
        pub code: String,
        pub message: String,
    }
}
