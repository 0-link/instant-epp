// ===============================================================================================
// fee-0.7 implementation (draft-brown-epp-fees / pre-RFC8748)
// Namespace: urn:ietf:params:xml:ns:fee-0.7
// ===============================================================================================

use instant_xml::{FromXml, ToXml};

use crate::domain::{DomainCheck, DomainCreate, DomainRenew, DomainTransfer};
use crate::request::{Extension, Transaction};

/// fee-0.7 namespace (pre-RFC8748)
pub const XMLNS: &str = "urn:ietf:params:xml:ns:fee-0.7";

// -------------------------------------------------------------------------------------------
// Shared types
// -------------------------------------------------------------------------------------------

/// <fee:period unit="y">1</fee:period>
///
/// In fee-0.7 XSD this element is in the fee namespace but typed as domain:periodType.  [oai_citation:1‡IETF Datatracker](https://datatracker.ietf.org/doc/draft-brown-epp-fees/04/)
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

/// <fee:command phase="sunrise" subphase="x">create</fee:command>
///
/// fee-0.7 uses element *text* for the command value (not a name="..." attribute).
#[derive(Debug, ToXml, FromXml, Clone)]
#[xml(rename = "command", ns(XMLNS))]
pub struct Command {
    #[xml(attribute)]
    pub phase: Option<String>,

    #[xml(attribute)]
    pub subphase: Option<String>,

    /// e.g. "create", "renew", "transfer", ...
    #[xml(direct)]
    pub value: String,
}

#[derive(Debug, FromXml, Clone)]
#[xml(rename = "command", ns(XMLNS))]
pub struct CommandResp {
    #[xml(attribute)]
    pub phase: Option<String>,

    #[xml(attribute)]
    pub subphase: Option<String>,

    #[xml(direct)]
    pub value: String,
}

/// feeType in fee-0.7 supports additional attributes like "applied".  [oai_citation:3‡IETF Datatracker](https://datatracker.ietf.org/doc/draft-brown-epp-fees/04/)
#[derive(Debug, FromXml, Clone)]
#[xml(rename = "fee", ns(XMLNS))]
pub struct Fee {
    #[xml(attribute)]
    pub description: Option<String>,

    #[xml(attribute)]
    pub refundable: Option<bool>,

    #[xml(attribute, rename = "grace-period")]
    pub grace_period: Option<String>, // xs:duration

    /// fee-0.7 has an "applied" attribute (e.g. "immediate"). Keep it optional.
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

/// fee-0.7: <fee:check><fee:domain>...</fee:domain>...</fee:check>  [oai_citation:4‡IETF Datatracker](https://datatracker.ietf.org/doc/draft-brown-epp-fees/04/)
#[derive(Debug, ToXml)]
#[xml(rename = "check", ns(XMLNS))]
pub struct Check<'a> {
    #[xml(rename = "domain")]
    pub domains: Vec<Domain<'a>>,
}

impl<'a> Extension for Check<'a> {
    type Response = CheckData;
}

impl<'a> Transaction<Check<'a>> for DomainCheck<'a> {}

/// One fee query for one domaincommand (fee-0.7 requires name inside the extension).  [oai_citation:5‡IETF Datatracker](https://datatracker.ietf.org/doc/draft-brown-epp-fees/04/)
#[derive(Debug, ToXml)]
#[xml(rename = "domain", ns(XMLNS))]
pub struct Domain<'a> {
    #[xml(rename = "name")]
    pub name: &'a str,

    /// Optional currency (if omitted, server policy/default applies).
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    #[xml(rename = "command")]
    pub command: Command,

    #[xml(rename = "period")]
    pub period: Option<Period>,
}

impl<'a> Check<'a> {
    /// Convenience helper similar to your fee-1.0 `Check::new(...)`, but fee-0.7 needs the names.
    ///
    /// It emits (create, renew, transfer) entries per domain name.
    pub fn new(
        names: impl IntoIterator<Item = &'a str>,
        currency: Option<&'a str>,
        period_years: Option<u16>,
    ) -> Self {
        let period = period_years.map(Period::years);

        let mut domains = Vec::new();
        for name in names {
            for cmd in ["create", "renew", "transfer"] {
                domains.push(Domain {
                    name,
                    currency,
                    command: Command {
                        phase: None,
                        subphase: None,
                        value: cmd.to_string(),
                    },
                    period,
                });
            }
        }

        Self { domains }
    }
}

// -------------------------------------------------------------------------------------------
// RESPONSE SIDE: <extension><fee:chkData>…</fee:chkData></extension>
// -------------------------------------------------------------------------------------------

#[derive(Debug, FromXml)]
#[xml(rename = "chkData", ns(XMLNS))]
pub struct CheckData {
    #[xml(rename = "cd")]
    pub list: Vec<CheckDomainData>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "cd", ns(XMLNS))]
pub struct CheckDomainData {
    #[xml(rename = "name")]
    pub name: String,

    #[xml(rename = "currency")]
    pub currency: String,

    #[xml(rename = "command")]
    pub command: CommandResp,

    #[xml(rename = "period")]
    pub period: Option<Period>,

    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    #[xml(rename = "class")]
    pub class: Option<String>,
}

// -------------------------------------------------------------------------------------------
// REQUEST SIDE: <extension><fee:create|renew|transfer>…</fee:...></extension>
// -------------------------------------------------------------------------------------------

#[derive(Debug, ToXml)]
#[xml(rename = "create", ns(XMLNS))]
pub struct Create<'a> {
    #[xml(rename = "currency")]
    pub currency: Option<&'a str>,

    /// fee-0.7 allows multiple fee elements.
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
#[xml(rename = "fee", ns(XMLNS))]
pub struct FeeReq<'a> {
    #[xml(attribute)]
    pub description: Option<&'a str>,

    #[xml(attribute)]
    pub refundable: Option<bool>,

    #[xml(attribute, rename = "grace-period")]
    pub grace_period: Option<&'a str>,

    /// Optional fee-0.7 attribute
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

// -------------------------------------------------------------------------------------------
// RESPONSE SIDE: <fee:creData>, <fee:renData>, <fee:trnData>
// -------------------------------------------------------------------------------------------

/// fee-0.7 "transform result" type (create/renew/update) includes balance/creditLimit optionally.  [oai_citation:6‡IETF Datatracker](https://datatracker.ietf.org/doc/draft-brown-epp-fees/04/)
#[derive(Debug, FromXml)]
#[xml(rename = "creData", ns(XMLNS))]
pub struct CreateData {
    #[xml(rename = "currency")]
    pub currency: String,

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
    pub currency: String,

    #[xml(rename = "fee")]
    pub fees: Vec<Fee>,

    #[xml(rename = "credit")]
    pub credits: Vec<Credit>,

    #[xml(rename = "balance")]
    pub balance: Option<f64>,

    #[xml(rename = "creditLimit")]
    pub credit_limit: Option<f64>,
}

/// fee-0.7 transfer result can include <fee:period> in op="query" responses.  [oai_citation:7‡IETF Datatracker](https://datatracker.ietf.org/doc/draft-brown-epp-fees/04/)
#[derive(Debug, FromXml)]
#[xml(rename = "trnData", ns(XMLNS))]
pub struct TransferData {
    #[xml(rename = "currency")]
    pub currency: String,

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

impl<'a> Transaction<Create<'a>> for DomainCreate<'a> {}
impl<'a> Transaction<Renew<'a>> for DomainRenew<'a> {}
impl<'a> Transaction<Transfer<'a>> for DomainTransfer<'a> {}
