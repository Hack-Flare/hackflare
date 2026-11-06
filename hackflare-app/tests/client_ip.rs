use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
};

use axum::{
    extract::{ConnectInfo, FromRequestParts},
    http::HeaderValue,
    http::{Request, request::Parts},
};

use hackflare_app::client_ip::{ClientIp, ClientIpError, ClientIpSource};

fn parts(source: ClientIpSource) -> Parts {
    let (mut parts, ()) = Request::builder()
        .body(())
        .expect("request should build")
        .into_parts();
    parts.extensions.insert(source);
    parts
}

#[test]
fn source_round_trips() {
    for source in [
        ClientIpSource::CfConnectingIp,
        ClientIpSource::CloudFrontViewerAddress,
        ClientIpSource::ConnectInfo,
        ClientIpSource::FlyClientIp,
        ClientIpSource::RightmostXForwardedFor,
        ClientIpSource::TrueClientIp,
        ClientIpSource::XEnvoyExternalAddress,
        ClientIpSource::XRealIp,
    ] {
        assert_eq!(ClientIpSource::from_str(&source.to_string()), Ok(source));
    }
}

#[tokio::test]
async fn extracts_from_connect_info() {
    let mut parts = parts(ClientIpSource::ConnectInfo);
    parts.extensions.insert(ConnectInfo(SocketAddr::from((
        Ipv4Addr::new(192, 0, 2, 1),
        8080,
    ))));

    let ClientIp(ip) = ClientIp::from_request_parts(&mut parts, &())
        .await
        .expect("connect info should extract");

    assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)));
}

#[tokio::test]
async fn extracts_rightmost_x_forwarded_for() {
    let mut parts = parts(ClientIpSource::RightmostXForwardedFor);
    parts.headers.append(
        "x-forwarded-for",
        "1.1.1.1, 2.2.2.2".parse::<HeaderValue>().expect("header"),
    );
    parts.headers.append(
        "x-forwarded-for",
        "3.3.3.3, 4.4.4.4".parse::<HeaderValue>().expect("header"),
    );

    let ClientIp(ip) = ClientIp::from_request_parts(&mut parts, &())
        .await
        .expect("header should extract");

    assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(4, 4, 4, 4)));
}

#[tokio::test]
async fn extracts_cloudfront_viewer_address() {
    let mut parts = parts(ClientIpSource::CloudFrontViewerAddress);
    parts.headers.insert(
        "cloudfront-viewer-address",
        "1:23:4567:89ab:c:d:e:f:8000"
            .parse::<HeaderValue>()
            .expect("header"),
    );

    let ClientIp(ip) = ClientIp::from_request_parts(&mut parts, &())
        .await
        .expect("header should extract");

    let ip_v6: Ipv6Addr = "1:23:4567:89ab:c:d:e:f".parse().expect("ip");
    assert_eq!(ip, IpAddr::V6(ip_v6));
}

#[tokio::test]
async fn rejects_repeated_single_headers() {
    let mut parts = parts(ClientIpSource::CfConnectingIp);
    parts.headers.append(
        "cf-connecting-ip",
        "1.1.1.1".parse::<HeaderValue>().expect("header"),
    );
    parts.headers.append(
        "cf-connecting-ip",
        "2.2.2.2".parse::<HeaderValue>().expect("header"),
    );

    let error = ClientIp::from_request_parts(&mut parts, &())
        .await
        .expect_err("duplicate header should fail");

    assert!(matches!(
        error,
        ClientIpError::MultipleHeaders("cf-connecting-ip")
    ));
}
