use crate::provision::Network;
use trust_dns_server::{authority::MessageResponseBuilder, proto::{op::{Header, MessageType, OpCode, ResponseCode}, rr::{LowerName, Name}}, server::{Request, RequestHandler, ResponseHandler, ResponseInfo}};

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
    network: Network,
    default_ttl: u32,
}

impl Handler {
    /// Create new handler from command-line options.
    pub fn from_network(network: Network) -> Self {
        let net_name: Name = network.zone.clone().into();
        Handler {
            zone: LowerName::from(net_name),
            network,
            default_ttl: 3,
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

    async fn handle_err<R: ResponseHandler>(
        &self,
        request: &Request,
        mut header: Header,
        error: DNSError,
        mut responder: R,
    ) -> ResponseInfo {
        let response_code = match error {
            DNSError::InvalidOpCode(_) => ResponseCode::Refused,
            DNSError::InvalidMessageType(_) => ResponseCode::Refused,
            DNSError::InvalidZone(_) => ResponseCode::Refused,
            DNSError::Io(_) => ResponseCode::ServFail,
        };
        header.set_response_code(response_code);
        let builder = MessageResponseBuilder::from_message_request(request);
        match responder.send_response(builder.build(header, &[], &[], &[], &[])).await {
            Ok(info) => {
                return info
            },
            Err(error) => {
                eprintln!("Error sending response: {}", error);
                return ResponseInfo::from(header);
            }
        }
    }
}

#[async_trait::async_trait]
impl RequestHandler for Handler {
    async fn handle_request<R: ResponseHandler>(
        &self,
        request: &Request,
        mut responder: R,
    ) -> ResponseInfo {
        println!("Query: {:?}", request.query().name().to_string());
        match self.validate_request(request) {
            Ok(_) => (),
            Err(error) =>{
                return self.handle_err(request, Header::response_from_request(request.header()), error, responder).await
            }
        }

        let builder = MessageResponseBuilder::from_message_request(request);
        let mut header = Header::response_from_request(request.header());
        header.set_id(request.header().id());
        header.set_authoritative(true);
        header.set_recursion_available(false);

        let entry = self.network.resolve(request.query().name().into());
        if entry.is_none() {
            header.set_response_code(ResponseCode::NXDomain);
            let response = builder.build(header, &[], &[], &[], &[]);
            match responder.send_response(response).await {
                Ok(info) => return info,
                Err(error) => {
                    eprintln!("Error sending response: {}", error);
                    return ResponseInfo::from(header);
                }
            }
        }

        let records = entry.unwrap().get_records(request.query().name().clone().into(), self.default_ttl);
        println!("Records: {:?}", records);
        let response = builder.build(header, records.iter(), &[], &[], &[]);
        match responder.send_response(response).await {
            Ok(info) => {
                info
            },
            Err(error) => {
                eprintln!("Error sending response: {}", error);
                return ResponseInfo::from(header);
            }
        }

    }
}