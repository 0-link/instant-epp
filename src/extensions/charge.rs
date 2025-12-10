use crate::domain::{DomainCheck, DomainCreate, DomainRenew};
use crate::request::{Extension, Transaction};
use instant_xml::{Deserializer, Error, FromXml, Id, Kind, ToXml};
use std::ops::Deref;
use crate::common::NoExtension;

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

#[derive(Debug, Clone, PartialEq, Eq)]
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

// -------- Agreement extension --------

#[derive(Debug, ToXml)]
#[xml(rename = "agreement", ns(XMLNS))]
pub struct Agreement {
    #[xml(rename = "set")]
    pub set: AgreementSet,
}

#[derive(Debug, ToXml)]
#[xml(rename = "set", ns(XMLNS))]
pub struct AgreementSet {
    #[xml(rename = "category")]
    pub category: AgreementCategory,

    #[xml(rename = "type")]
    pub charge_type: String,

    #[xml(rename = "amount")]
    pub amount: AgreementAmount,
}

#[derive(Debug, ToXml)]
#[xml(rename = "category", ns(XMLNS))]
pub struct AgreementCategory {
    // category code from check response, e.g. "PIR-BBB"
    #[xml(attribute, rename = "name")]
    pub name: Option<String>,

    // inner text, e.g. "premium" / "standard"
    #[xml(direct)]
    pub value: String,
}

#[derive(Debug, ToXml)]
#[xml(rename = "amount", ns(XMLNS))]
pub struct AgreementAmount {
    /// `command="create"` / `command="renew"` etc.
    #[xml(attribute, rename = "command")]
    pub command: &'static str,

    /// Numeric amount inside <charge:amount>.
    #[xml(direct)]
    pub amount: f64,
}

impl Agreement {
    /// Internal constructor to reuse for different commands.
    fn new_with_command(
        command: &'static str,
        category_name: Option<String>,
        category_value: String, // "premium" / "standard"
        charge_type: String,    // e.g. "price"
        amount: f64,
    ) -> Self {
        Self {
            set: AgreementSet {
                category: AgreementCategory {
                    name: category_name,
                    value: category_value,
                },
                charge_type,
                amount: AgreementAmount { command, amount },
            },
        }
    }

    pub fn create(
        category_name: Option<String>,
        category_value: String,
        charge_type: String,
        amount: f64,
    ) -> Self {
        Self::new_with_command("create", category_name, category_value, charge_type, amount)
    }

    pub fn renew(
        category_name: Option<String>,
        category_value: String,
        charge_type: String,
        amount: f64,
    ) -> Self {
        Self::new_with_command("renew", category_name, category_value, charge_type, amount)
    }
}

impl Extension for Agreement {
    // Ignore the charge extension response
    type Response = NoExtension;
}

impl<'a> Transaction<Agreement> for DomainCreate<'a> {}
impl<'a> Transaction<Agreement> for DomainRenew<'a> {}
