use crate::{handler::DNSError, ip_addr_serde::{SerializableIpAddr, SerializableCNAME}};
use serde_json;
use serde::Deserialize;
use anyhow::Result;
use trust_dns_server::proto::rr::{Name, Record};
use std::{collections::HashMap, fs::File, io::Read};

#[derive(Clone, Debug, Deserialize)]
pub struct NetworkEntry {
    #[serde(rename = "A")]
    pub a_recs: Vec<SerializableIpAddr>,
    #[serde(rename = "CNAME")]
    pub cname_recs: Vec<SerializableCNAME>,
    #[serde(rename = "AAAA")]
    pub aaaa_recs: Vec<SerializableIpAddr>,
}

impl NetworkEntry {
    pub fn get_records(&self, name: Name, ttl: u32) -> Vec<Record> {
        let mut records = vec![];
        for ip in &self.a_recs {
            records.push(Record::from_rdata(name.clone(), ttl, ip.clone().into()));
        }
        for cname in &self.cname_recs {
            records.push(Record::from_rdata(name.clone(), ttl, cname.clone().into()));
        }
        for aaaa in &self.aaaa_recs {
            records.push(Record::from_rdata(name.clone(), ttl, aaaa.clone().into()));
        }
        records
    }
}

pub fn get_network() -> Result<HashMap<String, NetworkEntry>> {
    let mut file = File::open("network.json")?;
    let mut contents = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => (),
        Err(e) => {
            return Err(DNSError::Io(e).into());
        }
    }
    let network: HashMap<String, NetworkEntry> = serde_json::from_str(&contents)?;
    Ok(network)
}