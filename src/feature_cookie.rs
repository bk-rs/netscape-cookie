use cookie::Cookie;
use time::OffsetDateTime;

use crate::{Cookie as CrateCookie, CookieExpires as CrateCookieExpires};

impl<'a> From<&'a CrateCookie> for Cookie<'a> {
    fn from(cc: &'a CrateCookie) -> Self {
        let mut c = Self::new(&cc.name, &cc.value);

        c.set_domain(&cc.domain);
        match cc.expires {
            CrateCookieExpires::Session => {
                c.set_expires(None);
            }
            CrateCookieExpires::DateTime(dt) => c.set_expires(OffsetDateTime::from_unix_timestamp(
                dt.naive_utc().timestamp(),
            )),
        }
        c.set_http_only(cc.http_only);
        c.set_path(&cc.path);
        c.set_secure(cc.secure);

        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{DateTime, NaiveDateTime, Utc};

    #[test]
    fn test_convert() -> Result<(), String> {
        let mut cc = CrateCookie {
            http_only: true,
            domain: ".example.com".to_owned(),
            include_subdomains: true,
            path: "/".to_owned(),
            secure: true,
            expires: CrateCookieExpires::Session,
            name: "foo".to_owned(),
            value: "bar".to_owned(),
        };

        let c = Cookie::from(&cc);
        assert_eq!(c.http_only(), Some(true));
        assert_eq!(c.domain(), Some(".example.com"));
        assert_eq!(c.path(), Some("/"));
        assert_eq!(c.secure(), Some(true));
        assert_eq!(c.expires(), None);
        assert_eq!(c.name(), "foo");
        assert_eq!(c.value(), "bar");

        cc.expires = CrateCookieExpires::DateTime(DateTime::<Utc>::from_utc(
            NaiveDateTime::from_timestamp(1640586740, 0),
            Utc,
        ));
        let c = Cookie::from(&cc);
        assert_eq!(
            c.expires(),
            Some(OffsetDateTime::from_unix_timestamp(1640586740))
        );

        Ok(())
    }
}
