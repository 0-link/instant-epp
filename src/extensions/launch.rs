//! Types for the EPP launch phase extension

use chrono::{DateTime, Utc};
use instant_xml::{FromXml, ToXml};

use crate::common::NoExtension;
use crate::domain::{DomainCheck, DomainCreate};
use crate::request::{Extension, Transaction};

/// Launch Phase Mapping namespace
pub const XMLNS: &str = "urn:ietf:params:xml:ns:launch-1.0";

/// Signed Mark (SMD) namespace used by encodedSignedMark
pub const SMD_XMLNS: &str = "urn:ietf:params:xml:ns:signedMark-1.0";

impl<'a> Transaction<Check<'a>> for DomainCheck<'a> {}
impl<'a> Transaction<Create<'a>> for DomainCreate<'a> {}

impl Extension for Check<'_> {
    type Response = CheckData;
}

impl Extension for Create<'_> {
    type Response = NoExtension;
}

#[derive(Clone, Copy, Debug, FromXml, ToXml)]
#[xml(scalar)]
pub enum PhaseType {
    #[xml(rename = "sunrise")]
    Sunrise,
    #[xml(rename = "landrush")]
    Landrush,
    #[xml(rename = "claims")]
    Claims,
    #[xml(rename = "open")]
    Open,
    #[xml(rename = "custom")]
    Custom,
}

/// <launch:phase name="custom-name">sunrise</launch:phase>
#[derive(Debug, ToXml)]
#[xml(rename = "phase", ns(XMLNS))]
pub struct Phase<'a> {
    /// Optional custom phase name
    #[xml(attribute)]
    pub name: Option<&'a str>,
    /// Phase identifier
    #[xml(direct)]
    pub value: PhaseType,
}

#[derive(Debug, FromXml)]
#[xml(rename = "phase", ns(XMLNS))]
pub struct PhaseData {
    /// Optional custom phase name
    #[xml(attribute)]
    pub name: Option<String>,
    /// Phase identifier
    #[xml(direct)]
    pub value: PhaseType,
}

impl<'a> Phase<'a> {
    pub fn new(value: PhaseType, name: Option<&'a str>) -> Self {
        Self { name, value }
    }

    pub fn custom(name: &'a str) -> Self {
        Self {
            name: Some(name),
            value: PhaseType::Custom,
        }
    }
}

//
// REQUEST SIDE: <extension><launch:check>…</launch:check></extension>
//

#[derive(Debug, ToXml)]
#[xml(rename = "check", ns(XMLNS))]
pub struct Check<'a> {
    /// Optional launch phase to check against
    #[xml(rename = "phase")]
    pub phase: Option<Phase<'a>>,
}

impl<'a> Check<'a> {
    pub fn new(phase: Option<Phase<'a>>) -> Self {
        Self { phase }
    }
}

//
// REQUEST SIDE: <extension><launch:create>…</launch:create></extension>
//

#[derive(Debug, ToXml)]
#[xml(rename = "create", ns(XMLNS))]
pub struct Create<'a> {
    /// Launch phase (sunrise, claims, landrush, ...)
    #[xml(rename = "phase")]
    pub phase: Phase<'a>,

    /// PIR Sunrise requires smd:encodedSignedMark (SMD)
    #[xml(rename = "encodedSignedMark")]
    pub encoded_signed_mark: Option<EncodedSignedMark<'a>>,

    /// Base64-encoded signed mark data (SMD) for sunrise
    #[xml(rename = "codeMark")]
    pub code_mark: Option<CodeMark<'a>>,

    /// Claims notice data
    #[xml(rename = "notice")]
    pub notice: Option<Notice<'a>>,

    /// Optional application identifier
    #[xml(rename = "applicationID")]
    pub application_id: Option<&'a str>,
}

impl<'a> Create<'a> {
    pub fn new(phase: Phase<'a>) -> Self {
        Self {
            phase,
            encoded_signed_mark: None,
            code_mark: None,
            notice: None,
            application_id: None,
        }
    }

    /// Convenience for PIR sunrise
    pub fn with_encoded_signed_mark(mut self, smd_b64: &'a str) -> Self {
        self.encoded_signed_mark = Some(EncodedSignedMark { value: smd_b64 });
        self.code_mark = None; // avoid sending both
        self
    }

    /// Convenience for registries that want codeMark
    pub fn with_code_mark(mut self, smd_b64: &'a str) -> Self {
        self.code_mark = Some(CodeMark { code: smd_b64 });
        self.encoded_signed_mark = None; // avoid sending both
        self
    }
}

/// This must serialize as `<smd:encodedSignedMark xmlns:smd="...">BASE64</smd:encodedSignedMark>`
#[derive(Debug, ToXml)]
#[xml(rename = "encodedSignedMark", ns(SMD_XMLNS))]
pub struct EncodedSignedMark<'a> {
    #[xml(direct)]
    pub value: &'a str,
}

#[derive(Debug, ToXml)]
#[xml(rename = "codeMark", ns(XMLNS))]
pub struct CodeMark<'a> {
    #[xml(rename = "code")]
    pub code: &'a str,
}

// RESPONSE SIDE: <extension><launch:chkData>…</launch:chkData></extension>

#[derive(Debug, FromXml)]
#[xml(rename = "chkData", ns(XMLNS))]
pub struct CheckData {
    /// Launch phase for the check response
    #[xml(rename = "phase")]
    pub phase: Option<PhaseData>,

    /// Repeated <launch:cd> elements
    #[xml(rename = "cd")]
    pub list: Vec<CheckDomainData>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "cd", ns(XMLNS))]
pub struct CheckDomainData {
    #[xml(rename = "name")]
    pub name: CheckName,

    #[xml(rename = "claimKey")]
    pub claim_key: Option<String>,
}

#[derive(Debug, FromXml)]
#[xml(rename = "name", ns(XMLNS))]
pub struct CheckName {
    #[xml(attribute, rename = "exists")]
    pub exists: Option<bool>,

    #[xml(direct)]
    pub value: String,
}

#[derive(Debug, ToXml)]
#[xml(rename = "notice", ns(XMLNS))]
pub struct Notice<'a> {
    #[xml(rename = "noticeID")]
    pub notice_id: &'a str,
    #[xml(rename = "notAfter")]
    pub not_after: DateTime<Utc>,
    #[xml(rename = "acceptedDate")]
    pub accepted_date: DateTime<Utc>,
}
