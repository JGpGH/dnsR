use std::{collections::HashMap, str::FromStr};

use crate::{options::Options, provision::NetworkEntry};
use trust_dns_server::{authority::MessageResponseBuilder, proto::{op::{Header, MessageType, OpCode, ResponseCode}, rr::{LowerName, Name}, serialize::binary::BinEncodable}, server::{Request, RequestHandler, ResponseHandler, ResponseInfo}};

#[derive(thiserror::Error, Debug)]
pub enum DNSError {
    #[error("Invalid OpCode {0:}")]
    InvalidOpCode(OpCode),
    #[error("Invalid MessageType {0:}")]
    InvalidMessageType(MessageType),
    #[error("Invalid Zone {0:}")]
    InvalidZone(LowerName),
    #[error("IO error: {0:}")]
    Io(#[from] std::io::Error)
}

/// DNS Request Handler
#[derive(Clone, Debug)]
pub struct Handler {
    zone: LowerName,
    network: HashMap<String, NetworkEntry>,
}

impl Handler {
    /// Create new handler from command-line options.
    pub fn from_options(options: &Options, network: HashMap<String, NetworkEntry>) -> Self {
        Handler {
            zone: LowerName::from(Name::from_str(&options.domain).unwrap()),
            network,
        }
    }

    fn validate_request(&self, request: &Request) -> Result<(), DNSError> {
        if request.op_code() != OpCode::Query {
            println!("Invalid OpCode: {:?}", request.op_code());
            return Err(DNSError::InvalidOpCode(request.op_code()));
        }

        if request.message_type() != MessageType::Query {
            println!("Invalid MessageType: {:?}", request.message_type());
            return Err(DNSError::InvalidMessageType(request.message_type()));
        }

        if !self.zone.zone_of(request.query().name()) {
            println!("Invalid Zone: {:?}", request.query().name());
            return Err(DNSError::InvalidZone(request.query().name().clone()))
        }

        Ok(())
    }

    async fn match_and_respond<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> Result<ResponseInfo, DNSError> {
        self.validate_request(request)?;

        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_authoritative(true);
        header.set_recursion_available(false);
        println!("Query: {:?}", request.query().name().to_string());
        let entry = self.network.get(&request.query().name().to_string());
        if entry.is_none() {
            header.set_response_code(ResponseCode::NXDomain);
            let response = builder.build(header, &[], &[], &[], &[]);
            return Ok(responder.send_response(response).await?);
        }

        let records = entry.unwrap().get_records(request.query().name().clone().into(), 300);
        let response = builder.build(header, records.iter(), &[], &[], &[]);
        Ok(responder.send_response(response).await?)
    }
}

#[async_trait::async_trait]
impl RequestHandler for Handler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        response: R,
    ) -> ResponseInfo {
        match self.match_and_respond(request, response).await {
            Ok(info) => info,
            Err(error) => {
                eprintln!("Error: {}", error);
                let mut header = Header::new();
                let response_code = match error {
                    DNSError::InvalidOpCode(_) => ResponseCode::NoError,
                    DNSError::InvalidMessageType(_) => ResponseCode::NoError,
                    DNSError::InvalidZone(_) => ResponseCode::NoError,
                    DNSError::Io(_) => ResponseCode::ServFail,
                };
                header.set_response_code(response_code);
                header.into()
            }
        }
    }
}