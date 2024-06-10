#![forbid(unsafe_code)]
#![cfg_attr(feature = "unstable", feature(error_in_core))]

use core::fmt;

use crate::{ClaimType, URIPart, *};

#[derive(Debug, PartialEq, Eq)]
pub enum ParseError {
	InvalidConnect,
	EmptyInput,
	Whitespace,
	NoHost,
	Invalid,
}

impl fmt::Display for ParseError {
	#[cfg(not(tarpaulin_include))]
	#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ParseError::InvalidConnect => {
				write!(f, "CONNECT requests can only contain \"hostname:port\"")
			},
			ParseError::EmptyInput => write!(f, "Empty input"),
			ParseError::Whitespace => write!(f, "Whitespace"),
			ParseError::NoHost => {
				write!(f, "host must be present if there is a schema")
			},
			ParseError::Invalid => write!(f, "Invalid URL"),
		}
	}
}

#[cfg(feature = "unstable")]
impl core::error::Error for ParseError {}

#[derive(Debug, PartialEq, Eq)]
enum State {
	SchemeSlash,
	SchemeSlashSlash,
	ServerStart,
	QueryStringStart,
	FragmentStart,
	Scheme,
	ServerWithAt,
	Server,
	Path,
	QueryString,
	Fragment,
}

#[derive(Debug, PartialEq, Eq)]
enum UrlFields {
	Scheme,
	Host,
	Path,
	Query,
	Fragment,
}

#[derive(Debug, PartialEq, Eq)]
/// A parsed URL
pub struct Url<'a> {
	pub scheme: Option<&'a str>,
	pub host: Option<&'a str>,
	pub port: Option<&'a str>,
	pub path: Option<&'a str>,
	pub query: Option<&'a str>,
	pub fragment: Option<&'a str>,
	pub userinfo: Option<&'a str>,
}

impl<'a> Url<'a> {
	#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
	/// Parse a URL from a string
	///
	/// # Examples
	///
	/// ```rust
	/// use url_lite::Url;
	/// # use url_lite::ParseError;
	///
	/// # fn run() -> Result<(), ParseError> {
	/// let url = Url::parse("http://example.com").expect("Invalid URL");
	/// # Ok(())
	/// # }
	/// # run().unwrap();
	/// ```
	pub fn parse(buf: &'a str) -> Result<Url<'a>, ParseError> {
		parse_url(buf, false)
	}

	/// Parse as a HTTP CONNECT method URL
	///
	/// Will return an error if the URL contains anything other than hostname
	/// and port
	pub fn parse_connect(buf: &'a str) -> Result<Url<'a>, ParseError> {
		parse_url(buf, true)
	}
}

impl<'a> Url<'a> {
	pub fn convert(&self, claim_type: &ClaimType) -> Result<URIPart, ParseError> {
		let mut sub_domain: Option<Vec<u8>> = None;

		let default_scheme: Vec<u8> = "https".as_bytes().to_vec();
		let scheme = self.scheme.map_or(default_scheme, |s| s.as_bytes().to_vec());

		let path =
			self.path.map_or(
				None,
				|p| {
					if p.len() == 1 {
						None
					} else {
						Some(p.as_bytes().to_vec())
					}
				},
			);

		let full_host = self.host.map_or("", |h| h);
		if full_host.is_empty() {
			return Err(ParseError::NoHost)
		}
		let sub_domain_index = self.sub_domain_start_index(full_host, claim_type);

		let host = if let Some(i) = sub_domain_index {
			sub_domain = Some(full_host[0..=i].as_bytes().to_vec());
			Some(full_host[i + 1..].as_bytes().to_vec())
		} else {
			if *claim_type == ClaimType::Domain {
				sub_domain = Some("www.".as_bytes().to_vec());
			}
			Some(full_host.as_bytes().to_vec())
		};

		Ok(URIPart::new(scheme, sub_domain, host, path))
	}

	/// Find the start index of where sub-domain is started.
	/// ### Example
	/// 1. file -> None
	/// 2. website.com -> None
	/// 3. sub1.website.com -> 4
	fn sub_domain_start_index(&self, host: &str, claim_type: &ClaimType) -> Option<usize> {
		let mut count = 0;
		let mut temp = host;
		let mut maybe_index: Option<usize> = None;
		while let Some(i) = temp.rfind('.') {
			count += 1;
			if matches!(claim_type, ClaimType::Contents { .. }) {
				maybe_index = Some(i);
				break
			}
			if count == 2 {
				maybe_index = Some(i);
				break
			}
			temp = &temp[0..i];
		}
		maybe_index
	}
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn parse_url(buf: &str, is_connect: bool) -> Result<Url, ParseError> {
	if buf.is_empty() {
		return Err(ParseError::EmptyInput)
	}

	let mut url = Url {
		scheme: None,
		host: None,
		port: None,
		path: None,
		query: None,
		fragment: None,
		userinfo: None,
	};

	let mut state = State::ServerStart;
	let mut old_uf: Option<UrlFields> = None;
	let mut found_at = false;

	let mut len = 0;
	let mut off = 0;

	for (i, p) in buf.chars().enumerate() {
		let uf: UrlFields;

		if p.is_whitespace() {
			return Err(ParseError::Whitespace)
		}

		if i == 0 && !is_connect {
			state = parse_url_start(p)?;
		} else {
			state = parse_url_char(state, p)?;
		}

		// Figure out the next field that we're operating on
		match state {
			// Skip delimeters
			State::SchemeSlash |
			State::SchemeSlashSlash |
			State::ServerStart |
			State::QueryStringStart |
			State::FragmentStart => continue,
			State::Scheme => {
				uf = UrlFields::Scheme;
			},
			State::ServerWithAt => {
				found_at = true;
				uf = UrlFields::Host;
			},
			State::Server => {
				uf = UrlFields::Host;
			},
			State::Path => {
				uf = UrlFields::Path;
			},
			State::QueryString => {
				uf = UrlFields::Query;
			},
			State::Fragment => {
				uf = UrlFields::Fragment;
			},
		}

		off += 1;
		len += 1;

		// Nothing's changed; soldier on
		if old_uf.as_ref() == Some(&uf) {
			continue
		}

		if let Some(old_uf) = old_uf {
			let value = Some(buf.get(off - len..off).ok_or(ParseError::Invalid)?);
			set_url_field(&old_uf, &mut url, value)
		}
		old_uf = Some(uf);
		len = 0;
		off = i;
	}

	if let Some(old_uf) = old_uf {
		let value = Some(buf.get(off - len..off + 1).ok_or(ParseError::Invalid)?);
		set_url_field(&old_uf, &mut url, value)
	}

	// host must be present if there is a schema
	// parsing http:///toto will fail
	if url.scheme.is_some() && url.host.is_none() {
		return Err(ParseError::NoHost)
	}

	if let Some(host_buf) = url.host.take() {
		url.host = None;

		let mut host_state =
			if found_at { HttpHostState::UserinfoStart } else { HttpHostState::HostStart };

		let mut off = 0;
		let mut len = 0;

		for (i, p) in host_buf.chars().enumerate() {
			let new_host_state = parse_host_char(&host_state, p)?;

			match new_host_state {
				HttpHostState::Host => {
					if host_state != HttpHostState::Host {
						off = i;
						len = 0;
					}
					len += 1;
					url.host = Some(host_buf.get(off..off + len).ok_or(ParseError::Invalid)?);
				},
				HttpHostState::Hostv6 => {
					if host_state != HttpHostState::Hostv6 {
						off = i;
					}
					len += 1;
					url.host = Some(host_buf.get(off..off + len).ok_or(ParseError::Invalid)?);
				},
				HttpHostState::Hostv6ZoneStart | HttpHostState::Hostv6Zone => {
					len += 1;
					url.host = Some(host_buf.get(off..off + len).ok_or(ParseError::Invalid)?);
				},
				HttpHostState::HostPort => {
					if host_state != HttpHostState::HostPort {
						off = i;
						len = 0;
					}
					len += 1;
					url.port = Some(host_buf.get(off..off + len).ok_or(ParseError::Invalid)?);
				},
				HttpHostState::Userinfo => {
					if host_state != HttpHostState::Userinfo {
						off = i;
						len = 0;
					}
					len += 1;
					url.userinfo = Some(host_buf.get(off..off + len).ok_or(ParseError::Invalid)?);
				},
				_ => {},
			}
			host_state = new_host_state;
		}

		// Make sure we don't end somewhere unexpected
		match host_state {
			HttpHostState::HostStart |
			HttpHostState::Hostv6Start |
			HttpHostState::Hostv6 |
			HttpHostState::Hostv6ZoneStart |
			HttpHostState::Hostv6Zone |
			HttpHostState::HostPortStart |
			HttpHostState::Userinfo |
			HttpHostState::UserinfoStart => return Err(ParseError::Invalid),
			_ => {},
		}
	}

	if is_connect &&
		(url.scheme.is_some() ||
			url.path.is_some() ||
			url.query.is_some() ||
			url.fragment.is_some() ||
			url.userinfo.is_some())
	{
		return Err(ParseError::InvalidConnect)
	}

	Ok(url)
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn set_url_field<'a>(uf: &UrlFields, url: &mut Url<'a>, value: Option<&'a str>) {
	match uf {
		UrlFields::Scheme => url.scheme = value,
		UrlFields::Host => url.host = value,
		UrlFields::Path => url.path = value,
		UrlFields::Query => url.query = value,
		UrlFields::Fragment => url.fragment = value,
	};
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn is_mark(c: char) -> bool {
	c == '-' ||
		c == '_' || c == '.' ||
		c == '!' || c == '~' ||
		c == '*' || c == '\'' ||
		c == '(' || c == ')'
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn is_userinfo_char(c: char) -> bool {
	c.is_ascii_alphanumeric() ||
		is_mark(c) ||
		c == '%' || c == ';' ||
		c == ':' || c == '&' ||
		c == '=' || c == '+' ||
		c == '$' || c == ','
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn is_url_char(c: char) -> bool {
	!matches!(c, '\0'..='\u{001F}' | '#' | '?' | '\x7F')
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn parse_url_start(ch: char) -> Result<State, ParseError> {
	// Proxied requests are followed by scheme of an absolute URI (alpha).
	// All methods except CONNECT are followed by '/' or '*'.
	if ch == '/' || ch == '*' {
		return Ok(State::Path)
	}

	if ch.is_ascii_alphabetic() {
		return Ok(State::Scheme)
	}

	Err(ParseError::Invalid)
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn parse_url_char(state: State, ch: char) -> Result<State, ParseError> {
	match state {
		State::Scheme => {
			if ch.is_ascii_alphabetic() {
				return Ok(state)
			}

			if ch == ':' {
				return Ok(State::SchemeSlash)
			}
		},
		State::SchemeSlash =>
			if ch == '/' {
				return Ok(State::SchemeSlashSlash)
			},
		State::SchemeSlashSlash =>
			if ch == '/' {
				return Ok(State::ServerStart)
			},
		State::ServerWithAt | State::ServerStart | State::Server => {
			if state == State::ServerWithAt && ch == '@' {
				return Err(ParseError::Invalid)
			}

			if ch == '/' {
				return Ok(State::Path)
			}

			if ch == '?' {
				return Ok(State::QueryStringStart)
			}

			if ch == '@' {
				return Ok(State::ServerWithAt)
			}

			if is_userinfo_char(ch) || ch == '[' || ch == ']' {
				return Ok(State::Server)
			}
		},
		State::Path => {
			if is_url_char(ch) {
				return Ok(state)
			}

			if ch == '?' {
				return Ok(State::QueryStringStart)
			}

			if ch == '#' {
				return Ok(State::FragmentStart)
			}
		},
		State::QueryStringStart | State::QueryString => {
			if is_url_char(ch) {
				return Ok(State::QueryString)
			}

			if ch == '?' {
				// allow extra '?' in query string
				return Ok(State::QueryString)
			}

			if ch == '#' {
				return Ok(State::FragmentStart)
			}
		},
		State::FragmentStart =>
			if is_url_char(ch) {
				return Ok(State::Fragment)
			},
		State::Fragment =>
			if is_url_char(ch) {
				return Ok(state)
			},
	};

	// We should never fall out of the switch above unless there's an error
	Err(ParseError::Invalid)
}

#[derive(Debug, PartialEq, Eq)]
enum HttpHostState {
	UserinfoStart,
	Userinfo,
	HostStart,
	Hostv6Start,
	Host,
	Hostv6,
	Hostv6End,
	Hostv6ZoneStart,
	Hostv6Zone,
	HostPortStart,
	HostPort,
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn is_host_char(c: char) -> bool {
	c.is_ascii_alphanumeric() || c == '.' || c == '-'
}

#[cfg_attr(feature = "_nopanic", no_panic::no_panic)]
fn parse_host_char(s: &HttpHostState, ch: char) -> Result<HttpHostState, ParseError> {
	match s {
		HttpHostState::Userinfo | HttpHostState::UserinfoStart => {
			if ch == '@' {
				return Ok(HttpHostState::HostStart)
			}

			if is_userinfo_char(ch) {
				return Ok(HttpHostState::Userinfo)
			}
		},
		HttpHostState::HostStart => {
			if ch == '[' {
				return Ok(HttpHostState::Hostv6Start)
			}

			if is_host_char(ch) {
				return Ok(HttpHostState::Host)
			}
		},
		HttpHostState::Host => {
			if is_host_char(ch) {
				return Ok(HttpHostState::Host)
			}
			if ch == ':' {
				return Ok(HttpHostState::HostPortStart)
			}
		},
		HttpHostState::Hostv6End =>
			if ch == ':' {
				return Ok(HttpHostState::HostPortStart)
			},
		HttpHostState::Hostv6 | HttpHostState::Hostv6Start => {
			if s == &HttpHostState::Hostv6 && ch == ']' {
				return Ok(HttpHostState::Hostv6End)
			}

			if ch.is_ascii_hexdigit() || ch == ':' || ch == '.' {
				return Ok(HttpHostState::Hostv6)
			}

			if s == &HttpHostState::Hostv6 && ch == '%' {
				return Ok(HttpHostState::Hostv6ZoneStart)
			}
		},
		HttpHostState::Hostv6Zone | HttpHostState::Hostv6ZoneStart => {
			if s == &HttpHostState::Hostv6Zone && ch == ']' {
				return Ok(HttpHostState::Hostv6End)
			}

			// RFC 6874 Zone ID consists of 1*( unreserved / pct-encoded)
			if ch.is_ascii_alphanumeric() ||
				ch == '%' || ch == '.' ||
				ch == '-' || ch == '_' ||
				ch == '~'
			{
				return Ok(HttpHostState::Hostv6Zone)
			}
		},
		HttpHostState::HostPort | HttpHostState::HostPortStart =>
			if ch.is_ascii_digit() {
				return Ok(HttpHostState::HostPort)
			},
	}

	Err(ParseError::Invalid)
}
