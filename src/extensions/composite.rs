use instant_xml::{Serializer, ToXml};
use std::fmt::Debug;

use crate::common::NoExtension;
use crate::domain::{DomainCheck, DomainCreate, DomainUpdate};
use crate::request::{Extension, Transaction};

/// A composite payload for <extension> that renders multiple child extensions.
#[derive(Debug)]
pub struct CompositeExt<E1: Extension, E2: Extension> {
    pub first: E1,
    pub second: E2,
}

impl<E1: Extension, E2: Extension> ToXml for CompositeExt<E1, E2> {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _id: Option<instant_xml::Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        if self.first.do_send() {
            self.first.serialize(None, serializer)?;
        }
        if self.second.do_send() {
            self.second.serialize(None, serializer)?;
        }
        Ok(())
    }
}

impl<E1: Extension, E2: Extension> Extension for CompositeExt<E1, E2> {
    const DO_SEND: bool = true;

    type Response = NoExtension;
}

impl<E1: Extension, E2: Extension> Transaction<CompositeExt<E1, E2>> for DomainUpdate<'_> {}
impl<'a, E1: Extension, E2: Extension> Transaction<CompositeExt<E1, E2>> for DomainCreate<'a> {}

/// Composite extension that renders two child extensions but uses the second
/// extension's response type.
#[derive(Debug)]
pub struct CompositeExtWithSecondResponse<E1: Extension, E2: Extension> {
    pub first: E1,
    pub second: E2,
}

impl<E1: Extension, E2: Extension> ToXml for CompositeExtWithSecondResponse<E1, E2> {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _id: Option<instant_xml::Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        if self.first.do_send() {
            self.first.serialize(None, serializer)?;
        }
        if self.second.do_send() {
            self.second.serialize(None, serializer)?;
        }
        Ok(())
    }
}

impl<E1: Extension, E2: Extension> Extension for CompositeExtWithSecondResponse<E1, E2> {
    const DO_SEND: bool = true;

    type Response = E2::Response;
}

impl<'a, E1: Extension, E2: Extension> Transaction<CompositeExtWithSecondResponse<E1, E2>>
    for DomainCheck<'a>
{
}

/// Composite extension that renders two child extensions but uses the first
/// extension's response type.
#[derive(Debug)]
pub struct CompositeExtWithFirstResponse<E1: Extension, E2: Extension> {
    pub first: E1,
    pub second: E2,
}

impl<E1: Extension, E2: Extension> ToXml for CompositeExtWithFirstResponse<E1, E2> {
    fn serialize<W: std::fmt::Write + ?Sized>(
        &self,
        _id: Option<instant_xml::Id<'_>>,
        serializer: &mut Serializer<W>,
    ) -> Result<(), instant_xml::Error> {
        if self.first.do_send() {
            self.first.serialize(None, serializer)?;
        }
        if self.second.do_send() {
            self.second.serialize(None, serializer)?;
        }
        Ok(())
    }
}

impl<E1: Extension, E2: Extension> Extension for CompositeExtWithFirstResponse<E1, E2> {
    const DO_SEND: bool = true;

    type Response = E1::Response;
}

impl<'a, E1: Extension, E2: Extension> Transaction<CompositeExtWithFirstResponse<E1, E2>>
    for DomainCheck<'a>
{
}

impl<'a, E1: Extension, E2: Extension> Transaction<CompositeExtWithFirstResponse<E1, E2>>
    for DomainCreate<'a>
{
}

impl<E1: Extension, E2: Extension> Transaction<CompositeExtWithFirstResponse<E1, E2>>
    for DomainUpdate<'_>
{
}
