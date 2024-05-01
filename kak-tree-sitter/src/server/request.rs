//! Requests that can be sent to the server from Kakoune.

use std::{fmt::Debug, io::Write, os::unix::net::UnixStream};

use serde::{Deserialize, Serialize};

use crate::{error::OhNo, kakoune::text_objects::OperationMode, tree_sitter::nav};

use super::resources::ServerResources;

/// Unidentified request (i.e. not linked to a given session).
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnixRequest {
  /// Inform KTS that a session exists and that we should be sending back the Kakoune commands to get KTS features.
  RegisterSession {
    name: String,
    client: Option<String>,
  },

  /// Inform KTS that a session has exited.
  SessionExit { name: String },

  /// Ask KTS to reload its configuration and reload grammars / queries.
  Reload,

  /// Ask KTS to shutdown.
  Shutdown,
}

impl UnixRequest {
  /// Add a session name to a [`UnidentifiedRequest`], replacing it if one was already provided.
  pub fn with_session(self, name: impl Into<String>) -> Self {
    let name = name.into();

    match self {
      UnixRequest::RegisterSession { client, .. } => UnixRequest::RegisterSession { client, name },
      UnixRequest::SessionExit { .. } => UnixRequest::SessionExit { name },
      _ => self,
    }
  }

  pub fn send(&self, resources: &ServerResources) -> Result<(), OhNo> {
    // serialize the request
    let serialized = serde_json::to_string(&self).map_err(|err| OhNo::CannotSendRequest {
      err: err.to_string(),
    })?;

    log::debug!("sending request {self:?}");

    // connect and send the request to the daemon
    UnixStream::connect(resources.socket_path())
      .map_err(|err| OhNo::CannotConnectToServer { err })?
      .write_all(serialized.as_bytes())
      .map_err(|err| OhNo::CannotSendRequest {
        err: err.to_string(),
      })
  }
}

/// Request payload.
///
/// Request payload are parameterized with the « origin » at which requests are expected.
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
  /// Try enabling highlighting for a given filetype.
  ///
  /// This request starts a “highlighting session.” The response will not replay with « supports highlighting » or
  /// « does not support highlighting », but instead will insert the Kakoune commands to ask for highlights only if the
  /// filetype is supported.
  TryEnableHighlight { lang: String, client: String },

  /// Ask to highlight the given buffer.
  ///
  /// The content of the buffer is streamed right after in the same command FIFO file the request was sent in.
  Highlight {
    client: String,
    buffer: String,
    lang: String,
    timestamp: u64,
  },

  /// Request to apply text-objects on selections.
  TextObjects {
    client: String,
    buffer: String,
    lang: String,
    pattern: String,
    selections: String,
    mode: OperationMode,
  },

  /// Request to navigate the tree-sitter tree on selections.
  Nav {
    client: String,
    buffer: String,
    lang: String,
    selections: String,
    dir: nav::Dir,
  },
}

impl Request {
  pub fn client_name(&self) -> Option<&str> {
    match self {
      Request::TryEnableHighlight { client, .. } => Some(client.as_str()),
      Request::Highlight { client, .. } => Some(client.as_str()),
      Request::TextObjects { client, .. } => Some(client.as_str()),
      Request::Nav { client, .. } => Some(client.as_str()),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::Request;

  #[test]
  fn serialization() {
    let req = Request::Highlight {
      client: "client0".to_owned(),
      buffer: "/tmp/a.rs".to_owned(),
      lang: "rust".to_owned(),
      timestamp: 0,
    };
    let expected =
      r#"{"type":"highlight","client":"client0","buffer":"/tmp/a.rs","lang":"rust","timestamp":0}"#;
    let serialized = serde_json::to_string(&req);

    assert_eq!(serialized.unwrap(), expected);
  }
}
