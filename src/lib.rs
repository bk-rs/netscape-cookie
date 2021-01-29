use std::{
    error, fmt,
    io::{self, BufRead, Cursor},
    num::ParseIntError,
    str::ParseBoolError,
};

use chrono::{DateTime, NaiveDateTime, Utc};

#[cfg(feature = "feature-cookie")]
mod feature_cookie;

const HTTP_ONLY_PREFIX: &str = "#HttpOnly_";

#[derive(Debug, Clone)]
pub struct Cookie {
    pub http_only: bool,
    pub domain: String,
    pub include_subdomains: bool,
    pub path: String,
    pub secure: bool,
    pub expires: CookieExpires,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone)]
pub enum CookieExpires {
    Session,
    DateTime(DateTime<Utc>),
}

#[derive(PartialEq, Debug)]
pub enum ParseError {
    IoError((io::ErrorKind, String)),
    DomainMissing,
    IncludeSubdomainsMissing,
    IncludeSubdomainsInvalid(ParseBoolError),
    PathMissing,
    SecureMissing,
    SecureInvalid(ParseBoolError),
    ExpiresMissing,
    ExpiresInvalid(ParseIntError),
    NameMissing,
    ValueMissing,
    TooManyElements,
}
impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError((kind, msg)) => write!(f, "IoError {:?} {}", kind, msg),
            Self::DomainMissing => write!(f, "DomainMissing"),
            Self::IncludeSubdomainsMissing => write!(f, "IncludeSubdomainsMissing"),
            Self::IncludeSubdomainsInvalid(err) => write!(f, "IncludeSubdomainsInvalid {}", err),
            Self::PathMissing => write!(f, "PathMissing"),
            Self::SecureMissing => write!(f, "SecureMissing"),
            Self::SecureInvalid(err) => write!(f, "SecureInvalid {}", err),
            Self::ExpiresMissing => write!(f, "ExpiresMissing"),
            Self::ExpiresInvalid(err) => write!(f, "ExpiresInvalid {}", err),
            Self::NameMissing => write!(f, "NameMissing"),
            Self::ValueMissing => write!(f, "ValueMissing"),
            Self::TooManyElements => write!(f, "TooManyElements"),
        }
    }
}
impl error::Error for ParseError {}

impl From<io::Error> for ParseError {
    fn from(err: io::Error) -> Self {
        Self::IoError((err.kind(), err.to_string()))
    }
}

pub fn parse(bytes: &[u8]) -> Result<Vec<Cookie>, ParseError> {
    let mut cursor = Cursor::new(bytes);
    let mut buf = String::new();

    let mut cookies: Vec<Cookie> = vec![];

    loop {
        buf.clear();
        let n = match cursor.read_line(&mut buf)? {
            0 => break,
            1 => continue,
            n => n - 1,
        };

        let mut s = &buf[..n];

        let mut http_only = false;
        if s.starts_with(HTTP_ONLY_PREFIX) {
            http_only = true;
            s = &buf[HTTP_ONLY_PREFIX.len()..n];
        } else if s.starts_with('#') {
            continue;
        }

        let mut split = s.split('\t');

        let domain = split.next().ok_or(ParseError::DomainMissing)?;

        let include_subdomains = split.next().ok_or(ParseError::IncludeSubdomainsMissing)?;
        let include_subdomains: bool = include_subdomains
            .to_ascii_lowercase()
            .parse()
            .map_err(ParseError::IncludeSubdomainsInvalid)?;

        let path = split.next().ok_or(ParseError::PathMissing)?;

        let secure = split.next().ok_or(ParseError::SecureMissing)?;
        let secure: bool = secure
            .to_ascii_lowercase()
            .parse()
            .map_err(ParseError::SecureInvalid)?;

        let expires = split.next().ok_or(ParseError::ExpiresMissing)?;
        let expires: u64 = expires.parse().map_err(ParseError::ExpiresInvalid)?;
        let expires = if expires == 0 {
            CookieExpires::Session
        } else {
            CookieExpires::DateTime(DateTime::<Utc>::from_utc(
                NaiveDateTime::from_timestamp(expires as i64, 0),
                Utc,
            ))
        };

        let name = split.next().ok_or(ParseError::NameMissing)?;

        let value = split.next().ok_or(ParseError::ValueMissing)?;

        if split.next().is_some() {
            return Err(ParseError::TooManyElements);
        }

        let cookie = Cookie {
            http_only,
            domain: domain.to_owned(),
            include_subdomains,
            path: path.to_owned(),
            secure,
            expires,
            name: name.to_owned(),
            value: value.to_owned(),
        };

        cookies.push(cookie);
    }

    Ok(cookies)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    #[test]
    fn test_parse_demo() -> Result<(), String> {
        let txt_content = fs::read_to_string("tests/files/demo_cookies.txt").unwrap();

        let cookies = parse(txt_content.as_bytes()).map_err(|err| err.to_string())?;

        println!("{:?}", cookies);

        assert_eq!(cookies.len(), 5);

        let cookie = cookies.last().unwrap();
        assert_eq!(cookie.http_only, true);
        assert_eq!(cookie.domain, ".github.com");
        assert_eq!(cookie.include_subdomains, true);
        assert_eq!(cookie.path, "/");
        assert_eq!(cookie.secure, true);
        match cookie.expires {
            CookieExpires::Session => assert!(false),
            CookieExpires::DateTime(dt) => {
                assert_eq!(dt.naive_utc().timestamp(), 1640586740);
            }
        }
        assert_eq!(cookie.name, "logged_in");
        assert_eq!(cookie.value, "no");

        Ok(())
    }
}
