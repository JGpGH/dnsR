use std::{net::IpAddr, ops::{Deref, DerefMut}, str::FromStr};
use serde::{Deserialize, Serialize, Deserializer};
use trust_dns_server::proto::rr::{rdata::CNAME, Name, RData};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SerializableIpAddr(
    #[serde(with = "ip_addr_serde")]
    IpAddr
);

impl Deref for SerializableIpAddr {
    type Target = IpAddr;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SerializableIpAddr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<IpAddr> for SerializableIpAddr {
    fn from(ip: IpAddr) -> Self {
        SerializableIpAddr(ip)
    }
}

impl Into<IpAddr> for SerializableIpAddr {
    fn into(self) -> IpAddr {
        self.0
    }
}

impl Into<RData> for SerializableIpAddr {
    fn into(self) -> RData {
        match self.0 {
            IpAddr::V4(ip) => RData::A(ip.into()),
            IpAddr::V6(ip) => RData::AAAA(ip.into()),
        }
    }
}

mod ip_addr_serde {
    use serde::{self, Deserialize, Deserializer, Serializer};
    use std::net::IpAddr;
    use std::str::FromStr;

    pub fn serialize<S>(ip: &IpAddr, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&ip.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<IpAddr, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        IpAddr::from_str(&s).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone)]
pub struct SerializableCNAME {
    rdata: RData,
}

impl<'de> Deserialize<'de> for SerializableCNAME {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Name::from_str(&s)
            .map(CNAME)
            .map(|rdata| SerializableCNAME { rdata: RData::CNAME(rdata) })
            .map_err(serde::de::Error::custom)
    }
}

impl Into<RData> for SerializableCNAME {
    fn into(self) -> RData {
        self.rdata
    }
}