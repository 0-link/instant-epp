use instant_xml::{Serializer, ToXml};
use std::fmt::Debug;

use crate::common::NoExtension;
use crate::domain::DomainUpdate;
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
        self.first.serialize(None, serializer)?;
        self.second.serialize(None, serializer)?;
        Ok(())
    }
}

impl<E1: Extension, E2: Extension> Extension for CompositeExt<E1, E2> {
    const DO_SEND: bool = true;

    type Response = NoExtension;
}

impl<E1: Extension, E2: Extension> Transaction<CompositeExt<E1, E2>> for DomainUpdate<'_> {}
