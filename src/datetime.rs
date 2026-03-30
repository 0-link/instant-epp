use chrono::{DateTime, Utc};
use instant_xml::{de::Deserializer, Error, OptionAccumulator};

fn parse_datetime_utc(value: &str) -> Result<DateTime<Utc>, Error> {
    DateTime::parse_from_rfc3339(value.trim())
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|_| Error::Other("invalid date/time".into()))
}

pub(crate) fn deserialize_datetime_utc<'xml>(
    into: &mut Option<DateTime<Utc>>,
    field: &'static str,
    deserializer: &mut Deserializer<'_, 'xml>,
) -> Result<(), Error> {
    if into.is_some() {
        return Err(Error::DuplicateValue(field));
    }

    let value = match deserializer.take_str()? {
        Some(value) => value,
        None => {
            deserializer.ignore()?;
            return Ok(());
        }
    };

    *into = Some(parse_datetime_utc(value.as_ref())?);
    deserializer.ignore()?;
    Ok(())
}

pub(crate) fn deserialize_datetime_utc_option<'xml>(
    into: &mut OptionAccumulator<DateTime<Utc>, Option<DateTime<Utc>>>,
    field: &'static str,
    deserializer: &mut Deserializer<'_, 'xml>,
) -> Result<(), Error> {
    deserialize_datetime_utc(into.get_mut(), field, deserializer)
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use instant_xml::{from_str, FromXml};

    #[test]
    fn parses_non_utc_offsets() {
        #[derive(Debug, FromXml, PartialEq)]
        #[xml(rename = "test")]
        struct Test {
            #[xml(deserialize_with = "crate::datetime::deserialize_datetime_utc")]
            dt: chrono::DateTime<Utc>,
        }

        let xml = "<test><dt>2026-03-30T02:36:20.000+01:00</dt></test>";
        let object = from_str::<Test>(xml).unwrap();

        assert_eq!(object.dt, Utc.with_ymd_and_hms(2026, 3, 30, 1, 36, 20).unwrap());
    }
}
