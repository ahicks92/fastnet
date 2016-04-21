use packets::*;
use std::collections;
use std::iter::{self, Iterator, IntoIterator};
use std::net;
use std::convert;

pub struct StatusTranslator {
    listening: bool,
    version: String,
    supported_extensions: collections::HashSet<String>,
}

impl StatusTranslator {
    pub fn new<T>(listening: bool, version: &str, supported_extensions: &[T])->StatusTranslator
    where T: convert::Into<String>+Clone,
    {
        let mut set = collections::HashSet::<String>::new();
        for i in supported_extensions.into_iter() {
            set.insert(i.clone().into());
        }
        StatusTranslator {
            listening: listening,
            version: version.to_string(),
            supported_extensions: set,
        }
    }


    pub fn translate(&self, request: &StatusRequest)->StatusResponse {
        match *request {
            StatusRequest::FastnetQuery => StatusResponse::FastnetResponse(self.listening),
            StatusRequest::VersionQuery => StatusResponse::VersionResponse(self.version.clone()),
            StatusRequest::ExtensionQuery(ref name) => {
                let supported = self.supported_extensions.contains(name);
                StatusResponse::ExtensionResponse{name: name.clone(), supported: supported}
            }
        }
    }
}
