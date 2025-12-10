use instant_xml::{FromXml, ToXml};

use crate::domain::{DomainCheck, DomainCreate, DomainRenew};
use crate::request::{Extension, Transaction};

/// RFC 8748 namespace
pub const XMLNS: &str = "urn:ietf:params:xml:ns:epp:fee-1.0";

//
// REQUEST SIDE: <extension><fee:check>…</fee:check></extension>
//

#[derive(Debug, ToXml)]
#[xml(rename = "check", ns(XMLNS))]
pub struct Check<'a> {
    /// Optional global currency, e.g. "USD"
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    /// One or more commands (create/renew/transfer/restore/…)
    #[xml(rename = "command")]
    pub commands: Vec<Command<'a>>,
}

impl<'a> Extension for Check<'a> {
    type Response = CheckData;
}

// Tie this extension to <domain:check> so (&DomainCheck, &Check) works.
impl<'a> Transaction<Check<'a>> for DomainCheck<'a> {}

/// <fee:command name="create">…</fee:command>
#[derive(Debug, ToXml)]
#[xml(rename = "command", ns(XMLNS))]
pub struct Command<'a> {
    /// "create", "renew", "transfer", "restore", …
    #[xml(attribute, rename = "name")]
    pub name: &'a str,

    #[xml(attribute)]
    pub phase: Option<&'a str>,

    #[xml(attribute)]
    pub subphase: Option<&'a str>,

    /// Optional period, e.g. 1 year
    #[xml(rename = "period")]
    pub period: Option<Period>,
}

/// <fee:period unit="y">1</fee:period>
#[derive(Debug, ToXml, FromXml)]
#[xml(rename = "period", ns(XMLNS))]
pub struct Period {
    #[xml(attribute, rename = "unit")]
    pub unit: PeriodUnit,

    #[xml(direct)]
    pub value: u16,
}

#[derive(Debug, ToXml, FromXml)]
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

impl<'a> Check<'a> {
    /// Helper: typical "USD, create+renew 1y" request used with <domain:check>.
    pub fn new(currency: Option<&'a str>, period_years: Option<u16>) -> Self {
        Check {
            currency,
            commands: vec![
                Command {
                    name: "create",
                    phase: None,
                    subphase: None,
                    period: period_years.map(Period::years),
                },
                Command {
                    name: "renew",
                    phase: None,
                    subphase: None,
                    period: period_years.map(Period::years),
                },
                Command {
                    name: "transfer",
                    phase: None,
                    subphase: None,
                    period: period_years.map(Period::years),
                },
            ],
        }
    }
}

//
// RESPONSE SIDE: <extension><fee:chkData>…</fee:chkData></extension>
// RFC 8748 §4.3
//

#[derive(Debug, FromXml)]
#[xml(rename = "chkData", ns(XMLNS))]
pub struct CheckData {
    /// <fee:currency>USD</fee:currency>
    #[xml(rename = "currency")]
    pub currency: String,

    /// Repeated <fee:cd> elements
    #[xml(rename = "cd")]
    pub list: Vec<CheckDomainData>,
}

/// <fee:cd avail="1"><fee:objID>example.com</fee:objID>…</fee:cd>
#[derive(Debug, FromXml)]
#[xml(rename = "cd", ns(XMLNS))]
pub struct CheckDomainData {
    #[xml(attribute)]
    pub avail: Option<bool>,

    /// <fee:objID>example.com</fee:objID>
    #[xml(rename = "objID")]
    pub obj_id: String,

    /// Optional class, eg. "Premium" / "Standard"
    #[xml(rename = "class")]
    pub class: Option<String>,

    /// One or more per-command entries
    #[xml(rename = "command")]
    pub commands: Vec<CommandResp>,
}

/// <fee:command name="create" standard="1">…</fee:command>
#[derive(Debug, FromXml)]
#[xml(rename = "command", ns(XMLNS))]
pub struct CommandResp {
    #[xml(attribute, rename = "name")]
    pub name: String, // "create" / "renew" / "transfer" / "restore"

    #[xml(attribute)]
    pub phase: Option<String>,

    #[xml(attribute)]
    pub subphase: Option<String>,

    /// RFC 8748: standard="1" indicates non-premium pricing
    #[xml(attribute)]
    pub standard: Option<bool>,

    /// <fee:period> inside the response (often present for create/renew)
    #[xml(rename = "period")]
    pub period: Option<Period>,

    /// One or more <fee:fee> elements (amounts)
    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    /// Optional <fee:credit> elements (e.g. promotional credits)
    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    /// Optional <fee:reason>text</fee:reason>
    #[xml(rename = "reason")]
    pub reason: Option<String>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "fee", ns(XMLNS))]
pub struct Fee {
    #[xml(attribute)]
    pub description: Option<String>,

    #[xml(attribute)]
    pub refundable: Option<bool>,

    #[xml(attribute, rename = "grace-period")]
    pub grace_period: Option<String>, // ISO 8601 duration

    #[xml(direct)]
    pub amount: f64,
}

#[derive(Debug, FromXml)]
#[xml(rename = "credit", ns(XMLNS))]
pub struct Credit {
    #[xml(attribute)]
    pub description: Option<String>,

    #[xml(direct)]
    pub amount: f64,
}

//
// REQUEST SIDE: <extension><fee:create>…</fee:create></extension>
//

#[derive(Debug, ToXml)]
#[xml(rename = "create", ns(XMLNS))]
pub struct Create<'a> {
    /// Optional global currency, e.g. "USD"
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    /// Single <fee:fee> with the expected amount
    #[xml(rename = "fee")]
    pub fee: CreateFee<'a>,
}

/// Request-side <fee:fee> for create.
/// Shape is the same as in RFC 8748, but request-only.
#[derive(Debug, ToXml)]
#[xml(rename = "fee", ns(XMLNS))]
pub struct CreateFee<'a> {
    #[xml(attribute)]
    pub description: Option<&'a str>,

    #[xml(attribute)]
    pub refundable: Option<bool>,

    #[xml(attribute, rename = "grace-period")]
    pub grace_period: Option<&'a str>, // ISO 8601 duration

    #[xml(direct)]
    pub amount: f64,
}

impl<'a> Create<'a> {
    /// Helper: "currency + period + price" for premium create.
    pub fn new(currency: Option<&'a str>, amount: f64) -> Self {
        Create {
            currency,
            fee: CreateFee {
                description: None,
                refundable: None,
                grace_period: None,
                amount,
            },
        }
    }
}

#[derive(Debug, FromXml)]
#[xml(rename = "creData", ns(XMLNS))]
pub struct CreateData {
    /// <fee:currency>USD</fee:currency>
    #[xml(rename = "currency")]
    pub currency: String,

    /// One or more <fee:fee> elements
    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,
}

impl<'a> Extension for Create<'a> {
    type Response = CreateData;
}

// Tie this extension to <domain:create> so (&DomainCreate, &Create) works.
impl<'a> Transaction<Create<'a>> for DomainCreate<'a> {}

//
// REQUEST SIDE: <extension><fee:renew>…</fee:renew></extension>
//

#[derive(Debug, ToXml)]
#[xml(rename = "renew", ns(XMLNS))]
pub struct Renew<'a> {
    /// Optional global currency, e.g. "USD"
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    /// Single <fee:fee> with the expected amount
    #[xml(rename = "fee")]
    pub fee: RenewFee<'a>,
}

/// Request-side <fee:fee> for create.
/// Shape is the same as in RFC 8748, but request-only.
#[derive(Debug, ToXml)]
#[xml(rename = "fee", ns(XMLNS))]
pub struct RenewFee<'a> {
    #[xml(attribute)]
    pub description: Option<&'a str>,

    #[xml(attribute)]
    pub refundable: Option<bool>,

    #[xml(attribute, rename = "grace-period")]
    pub grace_period: Option<&'a str>, // ISO 8601 duration

    #[xml(direct)]
    pub amount: f64,
}

impl<'a> Renew<'a> {
    /// Helper: "currency + period + price" for premium create.
    pub fn new(currency: Option<&'a str>, amount: f64) -> Self {
        Renew {
            currency,
            fee: RenewFee {
                description: None,
                refundable: None,
                grace_period: None,
                amount,
            },
        }
    }
}

#[derive(Debug, FromXml)]
#[xml(rename = "renData", ns(XMLNS))]
pub struct RenewData {
    /// <fee:currency>USD</fee:currency>
    #[xml(rename = "currency")]
    pub currency: String,

    /// One or more <fee:fee> elements
    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,
}

impl<'a> Extension for Renew<'a> {
    type Response = RenewData;
}

// Tie this extension to <domain:renew> so (&DomainRenew, &Renew) works.
impl<'a> Transaction<Renew<'a>> for DomainRenew<'a> {}
