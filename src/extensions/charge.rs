use crate::domain::DomainCheck;
use crate::request::{Extension, Transaction};
use instant_xml::{Deserializer, Error, FromXml, Id, Kind, ToXml};
use std::ops::Deref;

pub const XMLNS: &str = "http://www.unitedtld.com/epp/charge-1.0";

#[derive(Debug, Eq, PartialEq, ToXml)]
pub struct ChargeExtension;

impl Extension for ChargeExtension {
    const DO_SEND: bool = false;
    type Response = CheckData;
}

impl<'a> Transaction<ChargeExtension> for DomainCheck<'a> {}

#[derive(Debug, FromXml)]
#[xml(rename = "chkData", ns(XMLNS))]
pub struct CheckData {
    /// Repeated <charge:cd> elements
    #[xml(rename = "cd")]
    pub list: Vec<ChargeCd>,
}

#[derive(Debug, Clone, FromXml)]
#[xml(rename = "cd", ns(XMLNS))]
pub struct ChargeCd {
    #[xml(rename = "name")]
    pub name: String,

    #[xml(rename = "set")]
    pub set: ChargeSet,
}

#[derive(Debug, Clone, FromXml)]
#[xml(rename = "set", ns(XMLNS))]
pub struct ChargeSet {
    #[xml(rename = "category")]
    pub category: ChargeCategory,

    /// `price`, etc.
    #[xml(rename = "type")]
    pub charge_type: String,

    #[xml(rename = "amount")]
    pub amounts: Vec<ChargeAmount>,
}

#[derive(Debug, Clone, FromXml)]
#[xml(rename = "category", ns(XMLNS))]
pub struct ChargeCategory {
    /// `name="PIR-BBBB"`
    #[xml(attribute, rename = "name")]
    pub name: String,

    /// Inner text `premium`
    #[xml(direct)]
    pub value: String,
}

#[derive(Debug, Clone, FromXml)]
#[xml(rename = "amount", ns(XMLNS))]
pub struct ChargeAmount {
    /// `command="create"`, `command="renew"`, etc.
    #[xml(attribute, rename = "command")]
    pub command: ChargeCommand,

    /// Optional `name` attribute, e.g. `name="restore"` for restore fee.
    #[xml(attribute, rename = "name")]
    pub name: Option<String>,

    /// The numeric amount inside the tag, e.g. `99.00`.
    #[xml(direct)]
    pub amount: f64,
}


#[derive(Debug, Clone)]
pub enum ChargeCommand {
    Create,
    Renew,
    Transfer,
    Update,
    Empty,
    Unknown(String),
}

impl<'xml> FromXml<'xml> for ChargeCommand {
    #[inline]
    fn matches(id: Id<'_>, field: Option<Id<'_>>) -> bool {
        match field {
            Some(field) => id == field,
            None => false,
        }
    }

    fn deserialize<'cx>(
        into: &mut Self::Accumulator,
        field: &'static str,
        deserializer: &mut Deserializer<'cx, 'xml>,
    ) -> Result<(), Error> {
        if into.is_some() {
            return Err(Error::DuplicateValue(field));
        }

        *into = Some(match deserializer.take_str()? {
            Some(value) => match value.deref() {
                "create" => ChargeCommand::Create,
                "renew" => ChargeCommand::Renew,
                "transfer" => ChargeCommand::Transfer,
                "update" => ChargeCommand::Update,
                _ => ChargeCommand::Unknown(value.to_string()),
            },
            None => ChargeCommand::Empty,
        });

        Ok(())
    }

    type Accumulator = Option<ChargeCommand>;
    const KIND: Kind = Kind::Scalar;
}
