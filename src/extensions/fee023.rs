// ===============================================================================================
// fee-0.23 implementation (draft-ietf-regext-epp-fees-08 / pre-RFC8748)
// Namespace: urn:ietf:params:xml:ns:fee-0.23
// ===============================================================================================

use instant_xml::{FromXml, Id, ToXml};

use crate::domain::{
    DomainCheck, DomainCreate, DomainDelete, DomainRenew, DomainTransfer, DomainUpdate,
};
use crate::request::{Extension, Transaction};

/// fee-0.23 namespace (pre-RFC8748)
pub const XMLNS: &str = "urn:ietf:params:xml:ns:fee-0.23";

// -------------------------------------------------------------------------------------------
// Shared types
// -------------------------------------------------------------------------------------------

/// <fee:period unit="y">1</fee:period>
#[derive(Debug, ToXml, FromXml, Clone, Copy)]
#[xml(rename = "period", ns(XMLNS))]
pub struct Period {
    #[xml(attribute, rename = "unit")]
    pub unit: PeriodUnit,

    #[xml(direct)]
    pub value: u16,
}

#[derive(Debug, ToXml, FromXml, Clone, Copy)]
#[xml(scalar)]
pub enum PeriodUnit {
    #[xml(rename = "y")]
    Years,
    #[xml(rename = "m")]
    Months,
}

impl Period {
    pub fn years(v: u16) -> Self {
        Self {
            unit: PeriodUnit::Years,
            value: v,
        }
    }
}

/// <fee:reason lang="en">text</fee:reason>
#[derive(Debug, FromXml, Clone)]
#[xml(rename = "reason", ns(XMLNS))]
pub struct Reason {
    #[xml(attribute)]
    pub lang: Option<String>,

    #[xml(direct)]
    pub value: String,
}

#[derive(Debug, FromXml, Clone)]
#[xml(rename = "objID", ns(XMLNS))]
pub struct ObjectId {
    #[xml(attribute)]
    pub element: Option<String>,

    #[xml(direct)]
    pub value: String,
}

#[derive(Debug, FromXml, Clone)]
#[xml(rename = "fee", ns(XMLNS))]
pub struct Fee {
    #[xml(attribute)]
    pub description: Option<String>,

    #[xml(attribute)]
    pub refundable: Option<bool>,

    #[xml(attribute, rename = "grace-period")]
    pub grace_period: Option<String>,

    /// Optional fee-0.23 attribute ("immediate" or "delayed")
    #[xml(attribute)]
    pub applied: Option<String>,

    #[xml(direct)]
    pub amount: f64,
}

#[derive(Debug, FromXml, Clone)]
#[xml(rename = "credit", ns(XMLNS))]
pub struct Credit {
    #[xml(attribute)]
    pub description: Option<String>,

    #[xml(direct)]
    pub amount: f64,
}

// -------------------------------------------------------------------------------------------
// REQUEST SIDE: <extension><fee:check>…</fee:check></extension>
// -------------------------------------------------------------------------------------------

#[derive(Debug, ToXml)]
#[xml(rename = "check", ns(XMLNS))]
pub struct Check<'a> {
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    #[xml(rename = "command")]
    pub commands: Vec<Command<'a>>,
}

impl<'a> Extension for Check<'a> {
    type Response = CheckData;
}

impl<'a> Transaction<Check<'a>> for DomainCheck<'a> {}

/// <fee:command name="create">…</fee:command>
#[derive(Debug, ToXml)]
#[xml(rename = "command", ns(XMLNS))]
pub struct Command<'a> {
    /// "create", "renew", "transfer", "restore", "delete", "update", or "custom"
    #[xml(attribute, rename = "name")]
    pub name: &'a str,

    /// When name="custom", set customName to the actual command value.
    #[xml(attribute, rename = "customName")]
    pub custom_name: Option<&'a str>,

    #[xml(attribute)]
    pub phase: Option<&'a str>,

    #[xml(attribute)]
    pub subphase: Option<&'a str>,

    /// Optional period, e.g. 1 year
    #[xml(rename = "period")]
    pub period: Option<Period>,
}

impl<'a> Check<'a> {
    /// Helper: typical "USD, create+renew+transfer 1y" request used with <domain:check>.
    pub fn new(currency: Option<&'a str>, period_years: Option<u16>) -> Self {
        Check {
            currency,
            commands: vec![
                Command {
                    name: "create",
                    custom_name: None,
                    phase: None,
                    subphase: None,
                    period: period_years.map(Period::years),
                },
                Command {
                    name: "renew",
                    custom_name: None,
                    phase: None,
                    subphase: None,
                    period: period_years.map(Period::years),
                },
                Command {
                    name: "transfer",
                    custom_name: None,
                    phase: None,
                    subphase: None,
                    period: period_years.map(Period::years),
                },
            ],
        }
    }
}

// -------------------------------------------------------------------------------------------
// RESPONSE SIDE: <extension><fee:chkData>…</fee:chkData></extension>
// -------------------------------------------------------------------------------------------

#[derive(Debug, FromXml)]
#[xml(rename = "chkData", ns(XMLNS))]
pub struct CheckData {
    #[xml(rename = "currency")]
    pub currency: String,

    #[xml(rename = "cd")]
    pub list: Vec<CheckDomainData>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "cd", ns(XMLNS))]
pub struct CheckDomainData {
    #[xml(attribute)]
    pub avail: Option<bool>,

    #[xml(rename = "objID")]
    pub obj_id: ObjectId,

    #[xml(rename = "command")]
    pub commands: Vec<CommandResp>,

    #[xml(rename = "reason")]
    pub reason: Option<Reason>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "command", ns(XMLNS))]
pub struct CommandResp {
    #[xml(attribute, rename = "name")]
    pub name: String,

    #[xml(attribute, rename = "customName")]
    pub custom_name: Option<String>,

    #[xml(attribute)]
    pub phase: Option<String>,

    #[xml(attribute)]
    pub subphase: Option<String>,

    #[xml(rename = "period")]
    pub period: Option<Period>,

    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    #[xml(rename = "class")]
    pub class: Option<String>,

    #[xml(rename = "reason")]
    pub reason: Option<Reason>,
}

// -------------------------------------------------------------------------------------------
// REQUEST SIDE: <extension><fee:create|renew|transfer|update>…</fee:...></extension>
// -------------------------------------------------------------------------------------------

#[derive(Debug, ToXml)]
#[xml(rename = "create", ns(XMLNS))]
pub struct Create<'a> {
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    /// fee-0.23 allows multiple fee elements.
    #[xml(rename = "fee")]
    pub fees: Vec<FeeReq<'a>>,

    #[xml(rename = "credit")]
    pub credits: Vec<CreditReq<'a>>,
}

#[derive(Debug, ToXml)]
#[xml(rename = "renew", ns(XMLNS))]
pub struct Renew<'a> {
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    #[xml(rename = "fee")]
    pub fees: Vec<FeeReq<'a>>,

    #[xml(rename = "credit")]
    pub credits: Vec<CreditReq<'a>>,
}

#[derive(Debug, ToXml)]
#[xml(rename = "transfer", ns(XMLNS))]
pub struct Transfer<'a> {
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    #[xml(rename = "fee")]
    pub fees: Vec<FeeReq<'a>>,

    #[xml(rename = "credit")]
    pub credits: Vec<CreditReq<'a>>,
}

#[derive(Debug, ToXml)]
#[xml(rename = "update", ns(XMLNS))]
pub struct Update<'a> {
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    #[xml(rename = "fee")]
    pub fees: Vec<FeeReq<'a>>,

    #[xml(rename = "credit")]
    pub credits: Vec<CreditReq<'a>>,
}

#[derive(Debug, ToXml)]
#[xml(rename = "fee", ns(XMLNS))]
pub struct FeeReq<'a> {
    #[xml(attribute)]
    pub description: Option<&'a str>,

    #[xml(attribute)]
    pub refundable: Option<bool>,

    #[xml(attribute, rename = "grace-period")]
    pub grace_period: Option<&'a str>,

    /// Optional fee-0.23 attribute ("immediate" or "delayed")
    #[xml(attribute)]
    pub applied: Option<&'a str>,

    #[xml(direct)]
    pub amount: f64,
}

#[derive(Debug, ToXml)]
#[xml(rename = "credit", ns(XMLNS))]
pub struct CreditReq<'a> {
    #[xml(attribute)]
    pub description: Option<&'a str>,

    #[xml(direct)]
    pub amount: f64,
}

impl<'a> Create<'a> {
    pub fn new(currency: Option<&'a str>, amount: f64) -> Self {
        Self {
            currency,
            fees: vec![FeeReq {
                description: None,
                refundable: None,
                grace_period: None,
                applied: None,
                amount,
            }],
            credits: vec![],
        }
    }
}

impl<'a> Renew<'a> {
    pub fn new(currency: Option<&'a str>, amount: f64) -> Self {
        Self {
            currency,
            fees: vec![FeeReq {
                description: None,
                refundable: None,
                grace_period: None,
                applied: None,
                amount,
            }],
            credits: vec![],
        }
    }
}

impl<'a> Transfer<'a> {
    pub fn new(currency: Option<&'a str>, amount: f64) -> Self {
        Self {
            currency,
            fees: vec![FeeReq {
                description: None,
                refundable: None,
                grace_period: None,
                applied: None,
                amount,
            }],
            credits: vec![],
        }
    }
}

impl<'a> Update<'a> {
    pub fn new(currency: Option<&'a str>, amount: f64) -> Self {
        Self {
            currency,
            fees: vec![FeeReq {
                description: None,
                refundable: None,
                grace_period: None,
                applied: None,
                amount,
            }],
            credits: vec![],
        }
    }
}

// -------------------------------------------------------------------------------------------
// RESPONSE SIDE: <fee:creData>, <fee:renData>, <fee:trnData>, <fee:updData>, <fee:delData>
// -------------------------------------------------------------------------------------------

#[derive(Debug, FromXml)]
#[xml(rename = "creData", ns(XMLNS))]
pub struct CreateData {
    #[xml(rename = "currency")]
    pub currency: Option<String>,

    #[xml(rename = "period")]
    pub period: Option<Period>,

    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    #[xml(rename = "balance")]
    pub balance: Option<f64>,

    #[xml(rename = "creditLimit")]
    pub credit_limit: Option<f64>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "renData", ns(XMLNS))]
pub struct RenewData {
    #[xml(rename = "currency")]
    pub currency: Option<String>,

    #[xml(rename = "period")]
    pub period: Option<Period>,

    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    #[xml(rename = "balance")]
    pub balance: Option<f64>,

    #[xml(rename = "creditLimit")]
    pub credit_limit: Option<f64>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "trnData", ns(XMLNS))]
pub struct TransferData {
    #[xml(rename = "currency")]
    pub currency: Option<String>,

    #[xml(rename = "period")]
    pub period: Option<Period>,

    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    #[xml(rename = "balance")]
    pub balance: Option<f64>,

    #[xml(rename = "creditLimit")]
    pub credit_limit: Option<f64>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "updData", ns(XMLNS))]
pub struct UpdateData {
    #[xml(rename = "currency")]
    pub currency: Option<String>,

    #[xml(rename = "period")]
    pub period: Option<Period>,

    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    #[xml(rename = "balance")]
    pub balance: Option<f64>,

    #[xml(rename = "creditLimit")]
    pub credit_limit: Option<f64>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "delData", ns(XMLNS))]
pub struct DeleteData {
    #[xml(rename = "currency")]
    pub currency: Option<String>,

    #[xml(rename = "period")]
    pub period: Option<Period>,

    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    #[xml(rename = "balance")]
    pub balance: Option<f64>,

    #[xml(rename = "creditLimit")]
    pub credit_limit: Option<f64>,
}

impl<'a> Extension for Create<'a> {
    type Response = CreateData;
}
impl<'a> Extension for Renew<'a> {
    type Response = RenewData;
}
impl<'a> Extension for Transfer<'a> {
    type Response = TransferData;
}
impl<'a> Extension for Update<'a> {
    type Response = UpdateData;
}

impl<'a> Transaction<Create<'a>> for DomainCreate<'a> {}
impl<'a> Transaction<Renew<'a>> for DomainRenew<'a> {}
impl<'a> Transaction<Transfer<'a>> for DomainTransfer<'a> {}
impl<'a> Transaction<Update<'a>> for DomainUpdate<'a> {}

// -------------------------------------------------------------------------------------------
// RESPONSE-ONLY DELETE EXTENSION (fee:delData)
// -------------------------------------------------------------------------------------------

#[derive(Debug, Eq, PartialEq)]
pub struct DeleteExtension;

impl ToXml for DeleteExtension {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _field: Option<Id<'_>>,
        _serializer: &mut instant_xml::Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        Ok(())
    }

    fn present(&self) -> bool {
        false
    }
}

impl Extension for DeleteExtension {
    const DO_SEND: bool = false;
    type Response = DeleteData;
}

impl<'a> Transaction<DeleteExtension> for DomainDelete<'a> {}

