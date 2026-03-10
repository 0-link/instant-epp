use crate::common::NoExtension;
use crate::domain::{DomainCheck, DomainCreate, DomainRenew, DomainTransfer, DomainUpdate};
use crate::request::{Extension, Transaction};
use instant_xml::{Deserializer, Error, FromXml, Id, Kind, Serializer, ToXml};
use std::fmt::Write;
use std::ops::Deref;

pub const XMLNS: &str = "http://www.unitedtld.com/epp/charge-1.0";

#[derive(Debug, Eq, PartialEq)]
pub struct ChargeExtension;

impl ToXml for ChargeExtension {
    fn serialize<W: Write + ?Sized>(
        &self,
        _field: Option<Id<'_>>,
        _serializer: &mut Serializer<W>,
    ) -> Result<(), Error> {
        Ok(())
    }

    fn present(&self) -> bool {
        false
    }
}

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

    /// Optional `name="restore"` for restore pricing.
    #[xml(attribute, rename = "name")]
    pub name: Option<&'static str>,

    /// Numeric amount inside <charge:amount>.
    #[xml(direct)]
    pub amount: f64,
}

impl Agreement {
    /// Internal constructor to reuse for different commands.
    fn new_with_command(
        command: &'static str,
        name: Option<&'static str>,
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
                amount: AgreementAmount {
                    command,
                    name,
                    amount,
                },
            },
        }
    }

    pub fn create(
        category_name: Option<String>,
        category_value: String,
        charge_type: String,
        amount: f64,
    ) -> Self {
        Self::new_with_command(
            "create",
            None,
            category_name,
            category_value,
            charge_type,
            amount,
        )
    }

    pub fn renew(
        category_name: Option<String>,
        category_value: String,
        charge_type: String,
        amount: f64,
    ) -> Self {
        Self::new_with_command(
            "renew",
            None,
            category_name,
            category_value,
            charge_type,
            amount,
        )
    }

    pub fn transfer(
        category_name: Option<String>,
        category_value: String,
        charge_type: String,
        amount: f64,
    ) -> Self {
        Self::new_with_command(
            "transfer",
            None,
            category_name,
            category_value,
            charge_type,
            amount,
        )
    }

    pub fn restore(
        category_name: Option<String>,
        category_value: String,
        charge_type: String,
        amount: f64,
    ) -> Self {
        Self::new_with_command(
            "update",
            Some("restore"),
            category_name,
            category_value,
            charge_type,
            amount,
        )
    }
}

impl Extension for Agreement {
    // Ignore the charge extension response
    type Response = NoExtension;
}

impl<'a> Transaction<Agreement> for DomainCreate<'a> {}
impl<'a> Transaction<Agreement> for DomainRenew<'a> {}
impl<'a> Transaction<Agreement> for DomainTransfer<'a> {}
impl<'a> Transaction<Agreement> for DomainUpdate<'a> {}

#[cfg(test)]
mod tests {
    use super::{Agreement, XMLNS};
    use crate::client::RequestData;
    use crate::domain::update::{DomainChangeInfo, DomainUpdate};
    use crate::extensions::composite::CompositeExtWithFirstResponse;
    use crate::extensions::rgp::request::{RgpRestoreRequest, Update as RgpUpdate};
    use crate::request::{Command, CommandWrapper, Extension, Transaction};
    use crate::tests::CLTRID;
    use crate::xml;

    fn serialize_request<'c, 'e, Cmd, Ext>(req: impl Into<RequestData<'c, 'e, Cmd, Ext>>) -> String
    where
        Cmd: Transaction<Ext> + Command + 'c,
        Ext: Extension + 'e,
    {
        let req = req.into();
        xml::serialize(CommandWrapper::new(req.command, req.extension, CLTRID)).unwrap()
    }

    fn empty_domain_update<'a>() -> DomainUpdate<'a> {
        let mut object = DomainUpdate::new("eppdev.com");
        object.info(DomainChangeInfo {
            registrant: None,
            auth_info: None,
        });
        object
    }

    #[test]
    fn restore_serializes_as_charge_update() {
        let object = empty_domain_update();
        let ext = Agreement::restore(
            Some("PIR-BBBB".to_string()),
            "premium".to_string(),
            "price".to_string(),
            80.0,
        );

        let xml = serialize_request((&object, &ext));

        assert!(xml.contains(&format!(r#"<agreement xmlns="{}">"#, XMLNS)));
        assert!(xml.contains(r#"command="update" name="restore""#));
        assert!(xml.contains(">premium</category>"));
        assert!(xml.contains(">price</type>"));
    }

    #[test]
    fn composite_restore_serializes_rgp_before_charge() {
        let object = empty_domain_update();
        let ext = CompositeExtWithFirstResponse {
            first: RgpUpdate {
                data: RgpRestoreRequest::default(),
            },
            second: Agreement::restore(
                Some("PIR-BBBB".to_string()),
                "premium".to_string(),
                "price".to_string(),
                80.0,
            ),
        };

        let xml = serialize_request((&object, &ext));

        let rgp_idx = xml.find(crate::extensions::rgp::XMLNS).unwrap();
        let charge_idx = xml.find(XMLNS).unwrap();

        assert!(rgp_idx < charge_idx);
        assert!(xml.contains(r#"op="request""#));
        assert!(xml.contains(r#"command="update" name="restore""#));
    }
}
