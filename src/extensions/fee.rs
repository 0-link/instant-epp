use instant_xml::{FromXml, ToXml};

use crate::domain::DomainCheck;
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
    pub fn simple_1y_create_and_renew(currency: Option<&'a str>) -> Self {
        Check {
            currency,
            commands: vec![
                Command {
                    name: "create",
                    phase: None,
                    subphase: None,
                    period: Some(Period::years(1)),
                },
                Command {
                    name: "renew",
                    phase: None,
                    subphase: None,
                    period: Some(Period::years(1)),
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
