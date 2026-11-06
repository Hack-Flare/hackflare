//! Client IP extraction with a configurable proxy header source.

use std::{
    fmt,
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::{HeaderMap, StatusCode, request::Parts},
    response::{IntoResponse, Response},
};

/// Header or connection source used to determine the client IP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientIpSource {
    /// IP from the `CF-Connecting-IP` header.
    CfConnectingIp,
    /// IP from the `CloudFront-Viewer-Address` header.
    CloudFrontViewerAddress,
    /// IP from the connection information.
    ConnectInfo,
    /// IP from the `Fly-Client-IP` header.
    FlyClientIp,
    /// Rightmost IP from the `X-Forwarded-For` header.
    RightmostXForwardedFor,
    /// IP from the `True-Client-IP` header.
    TrueClientIp,
    /// IP from the `X-Envoy-External-Address` header.
    XEnvoyExternalAddress,
    /// IP from the `X-Real-Ip` header.
    XRealIp,
}

/// Invalid [`ClientIpSource`] value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseClientIpSourceError(String);

impl fmt::Display for ParseClientIpSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid ClientIpSource value {}", self.0)
    }
}

impl std::error::Error for ParseClientIpSourceError {}

impl FromStr for ClientIpSource {
    type Err = ParseClientIpSourceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "CfConnectingIp" => Self::CfConnectingIp,
            "CloudFrontViewerAddress" => Self::CloudFrontViewerAddress,
            "ConnectInfo" => Self::ConnectInfo,
            "FlyClientIp" => Self::FlyClientIp,
            "RightmostXForwardedFor" => Self::RightmostXForwardedFor,
            "TrueClientIp" => Self::TrueClientIp,
            "XEnvoyExternalAddress" => Self::XEnvoyExternalAddress,
            "XRealIp" => Self::XRealIp,
            _ => return Err(ParseClientIpSourceError(value.to_string())),
        })
    }
}

impl fmt::Display for ClientIpSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::CfConnectingIp => "CfConnectingIp",
            Self::CloudFrontViewerAddress => "CloudFrontViewerAddress",
            Self::ConnectInfo => "ConnectInfo",
            Self::FlyClientIp => "FlyClientIp",
            Self::RightmostXForwardedFor => "RightmostXForwardedFor",
            Self::TrueClientIp => "TrueClientIp",
            Self::XEnvoyExternalAddress => "XEnvoyExternalAddress",
            Self::XRealIp => "XRealIp",
        })
    }
}

/// Extracted client IP address.
#[derive(Debug, Clone, Copy)]
pub struct ClientIp(pub IpAddr);

impl<S> FromRequestParts<S> for ClientIp
where
    S: Sync,
{
    type Rejection = ClientIpError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let source = parts
            .extensions
            .get::<ClientIpSource>()
            .copied()
            .ok_or(ClientIpError::MissingSource)?;

        let ip = match source {
            ClientIpSource::ConnectInfo => parts
                .extensions
                .get::<ConnectInfo<SocketAddr>>()
                .map(|ConnectInfo(address)| address.ip())
                .ok_or(ClientIpError::MissingConnectInfo)?,
            ClientIpSource::CfConnectingIp => single_header(&parts.headers, "cf-connecting-ip")?,
            ClientIpSource::CloudFrontViewerAddress => cloudfront_viewer(&parts.headers)?,
            ClientIpSource::FlyClientIp => single_header(&parts.headers, "fly-client-ip")?,
            ClientIpSource::RightmostXForwardedFor => {
                rightmost_x_forwarded_for(&parts.headers)?
            }
            ClientIpSource::TrueClientIp => single_header(&parts.headers, "true-client-ip")?,
            ClientIpSource::XEnvoyExternalAddress => {
                single_header(&parts.headers, "x-envoy-external-address")?
            }
            ClientIpSource::XRealIp => single_header(&parts.headers, "x-real-ip")?,
        };

        Ok(Self(ip))
    }
}

/// Error returned when a client IP cannot be extracted.
#[derive(Debug)]
pub enum ClientIpError {
    /// The [`ClientIpSource`] extension is missing.
    MissingSource,
    /// The connection information is missing.
    MissingConnectInfo,
    /// The configured header is missing.
    MissingHeader(&'static str),
    /// The configured header appears more than once.
    MultipleHeaders(&'static str),
    /// The configured header is not a valid IP address.
    InvalidHeader(&'static str),
}

impl fmt::Display for ClientIpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSource => write!(f, "missing client IP source configuration"),
            Self::MissingConnectInfo => write!(f, "missing connection information"),
            Self::MissingHeader(name) => write!(f, "missing required header: {name}"),
            Self::MultipleHeaders(name) => {
                write!(f, "multiple occurrences of the header are not allowed: {name}")
            }
            Self::InvalidHeader(name) => write!(f, "invalid value for header: {name}"),
        }
    }
}

impl std::error::Error for ClientIpError {}

impl IntoResponse for ClientIpError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

fn single_header(headers: &HeaderMap, name: &'static str) -> Result<IpAddr, ClientIpError> {
    let mut values = headers.get_all(name).iter();
    let value = values.next().ok_or(ClientIpError::MissingHeader(name))?;
    if values.next().is_some() {
        return Err(ClientIpError::MultipleHeaders(name));
    }
    parse_ip(value, name)
}

fn last_header<'a>(headers: &'a HeaderMap, name: &'static str) -> Result<&'a str, ClientIpError> {
    headers
        .get_all(name)
        .iter()
        .next_back()
        .ok_or(ClientIpError::MissingHeader(name))?
        .to_str()
        .map_err(|_| ClientIpError::InvalidHeader(name))
}

fn parse_ip(value: &axum::http::HeaderValue, name: &'static str) -> Result<IpAddr, ClientIpError> {
    value
        .to_str()
        .map_err(|_| ClientIpError::InvalidHeader(name))?
        .trim()
        .parse()
        .map_err(|_| ClientIpError::InvalidHeader(name))
}

fn cloudfront_viewer(headers: &HeaderMap) -> Result<IpAddr, ClientIpError> {
    let name = "cloudfront-viewer-address";
    last_header(headers, name)?
        .rsplit_once(':')
        .map(|(ip, _)| ip)
        .ok_or(ClientIpError::InvalidHeader(name))?
        .trim()
        .parse()
        .map_err(|_| ClientIpError::InvalidHeader(name))
}

fn rightmost_x_forwarded_for(headers: &HeaderMap) -> Result<IpAddr, ClientIpError> {
    let name = "x-forwarded-for";
    last_header(headers, name)?
        .rsplit(',')
        .next()
        .ok_or(ClientIpError::InvalidHeader(name))?
        .trim()
        .parse()
        .map_err(|_| ClientIpError::InvalidHeader(name))
}


