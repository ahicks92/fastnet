use packets;
use std::collections;
use std::iter::{self, Iterator, IntoIterator};
use std::net;
use std::convert;

pub static PROTOCOL_VERSION: &'static str = "1.0";
pub static SUPPORTED_EXTENSIONS: &'static [&'static str] = &[];

pub fn translate(request: &packets::StatusRequest)->packets::StatusResponse {
    match *request {
        packets::StatusRequest::FastnetQuery => packets::StatusResponse::FastnetResponse(true),
        packets::StatusRequest::VersionQuery => packets::StatusResponse::VersionResponse(PROTOCOL_VERSION.to_string()),
        packets::StatusRequest::ExtensionQuery(ref name) => {
            let mut supported = false;
            for i in SUPPORTED_EXTENSIONS {
                if i.eq(&name) {
                    supported = true;
                }
            }
            packets::StatusResponse::ExtensionResponse{name: name.clone(), supported: supported}
        }
    }
}
