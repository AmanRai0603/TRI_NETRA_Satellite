//! The smallest HTTP/1.1 the page needs: one request per connection, heads and bodies
//! capped, the Host checked, a change allowed only with the page's own header.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

pub const MAX_HEAD: usize = 16 * 1024;
pub const MAX_BODY: usize = 64 * 1024;

pub struct Request { pub method: String, pub path: String, pub query: String, pub headers: Vec<(String, String)>, pub body: Vec<u8> }

impl Request {
    pub fn header(&self, k: &str) -> Option<&str> {
        self.headers.iter().find(|(n, _)| n.eq_ignore_ascii_case(k)).map(|(_, v)| v.as_str())
    }
    /// A query parameter, percent-decoded.
    pub fn param(&self, k: &str) -> Option<String> {
        self.query.split('&').filter_map(|p| p.split_once('=')).find(|(n, _)| *n == k).map(|(_, v)| decode(v))
    }
}

pub struct Response { pub status: u16, pub kind: &'static str, pub body: Vec<u8>, pub extra: Vec<(String, String)> }

impl Response {
    pub fn json(status: u16, v: &serde_json::Value) -> Self { Response { status, kind: "application/json", body: (v.to_string() + "\n").into_bytes(), extra: vec![] } }
    pub fn error(status: u16, why: &str) -> Self { Self::json(status, &serde_json::json!({"ok": false, "error": why})) }
}

pub fn esc(s: &str) -> String { s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;") }

fn decode(s: &str) -> String {
    let b = s.as_bytes();
    let (mut out, mut i) = (Vec::with_capacity(b.len()), 0);
    while i < b.len() {
        if b[i] == b'+' {
            out.push(b' ');
            i += 1;
        } else if b[i] == b'%' && i + 2 < b.len() {
            match std::str::from_utf8(&b[i + 1..i + 3]).ok().and_then(|h| u8::from_str_radix(h, 16).ok()) {
                Some(x) => { out.push(x); i += 3; }
                None => { out.push(b'%'); i += 1; }
            }
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn read_request(s: &mut TcpStream) -> Result<Request, (u16, &'static str)> {
    let _ = s.set_read_timeout(Some(Duration::from_secs(10)));
    // the whole request within 15 s: a client sending a byte at a time cannot hold a connection
    let deadline = std::time::Instant::now() + Duration::from_secs(15);
    let late = || std::time::Instant::now() > deadline;
    let mut buf = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    let head_end = loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") { break i; }
        if buf.len() > MAX_HEAD { return Err((431, "the request head is too large")); }
        if late() { return Err((408, "the request took too long to arrive")); }
        let n = s.read(&mut chunk).map_err(|_| (408, "the request did not arrive"))?;
        if n == 0 { return Err((400, "the connection closed early")); }
        buf.extend_from_slice(&chunk[..n]);
    };
    let head = std::str::from_utf8(&buf[..head_end]).map_err(|_| (400, "the request head is not text"))?;
    let mut lines = head.split("\r\n");
    let first: Vec<&str> = lines.next().unwrap_or("").split(' ').collect();
    if first.len() != 3 { return Err((400, "not an HTTP request")); }
    let headers: Vec<(String, String)> = lines.filter_map(|l| l.split_once(':')).map(|(k, v)| (k.trim().to_string(), v.trim().to_string())).collect();
    let (path, query) = first[1].split_once('?').map(|(p, q)| (p.to_string(), q.to_string())).unwrap_or((first[1].to_string(), String::new()));
    let len = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-length")).map(|(_, v)| v.parse::<usize>()).transpose().map_err(|_| (400, "a bad Content-Length"))?.unwrap_or(0);
    if len > MAX_BODY { return Err((413, "the request body is too large")); }
    let mut body = buf[head_end + 4..].to_vec();
    while body.len() < len {
        if late() { return Err((408, "the request took too long to arrive")); }
        let n = s.read(&mut chunk).map_err(|_| (408, "the request body did not arrive"))?;
        if n == 0 { break; }
        body.extend_from_slice(&chunk[..n]);
    }
    body.truncate(len);
    Ok(Request { method: first[0].to_string(), path, query, headers, body })
}

fn write_response(s: &mut TcpStream, r: &Response) {
    let reason = match r.status { 200 => "OK", 400 => "Bad Request", 403 => "Forbidden", 404 => "Not Found", 405 => "Method Not Allowed",
        408 => "Request Timeout", 409 => "Conflict", 413 => "Payload Too Large", 421 => "Misdirected Request", 431 => "Request Header Fields Too Large", _ => "Error" };
    let mut head = format!("HTTP/1.1 {} {reason}\r\nContent-Type: {}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\n\
        Referrer-Policy: no-referrer\r\nContent-Security-Policy: default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; script-src 'self' 'unsafe-inline'\r\nConnection: close\r\n",
        r.status, r.kind, r.body.len());
    for (k, v) in &r.extra { head += &format!("{k}: {v}\r\n"); }
    head += "\r\n";
    let _ = s.write_all(head.as_bytes());
    let _ = s.write_all(&r.body);
}

/// Is the Host this app on this machine? (A page on another site that resolves its own name to
/// 127.0.0.1 sends its own name as Host, and is refused.)
pub fn host_ok(host: Option<&str>, port: u16) -> bool {
    matches!(host, Some(h) if h == format!("127.0.0.1:{port}") || h == format!("localhost:{port}"))
}

/// A change (POST) comes from this app's page: its own header, and no other site's Origin.
pub fn change_ok(r: &Request, port: u16) -> bool {
    let origin_ok = match r.header("origin") { None => true, Some(o) => o == format!("http://127.0.0.1:{port}") || o == format!("http://localhost:{port}") };
    r.header("x-trinetra") == Some("1") && origin_ok
}

pub fn serve(mut s: TcpStream, port: u16) {
    let resp = match read_request(&mut s) {
        Err((code, why)) => Response::error(code, why),
        Ok(r) if !host_ok(r.header("host"), port) => Response::error(421, "this app answers only to 127.0.0.1 and localhost"),
        Ok(r) if r.method == "POST" && !change_ok(&r, port) => Response::error(403, "a change must come from this app's own page"),
        Ok(r) if r.method != "GET" && r.method != "POST" => Response::error(405, "GET or POST only"),
        Ok(r) => match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| crate::routes::route(&r, port))) {
            Ok(x) => x,
            Err(_) => Response::error(500, "an internal error: a crash report was written to the log folder; the app keeps running"),
        },
    };
    write_response(&mut s, &resp);
}

#[cfg(test)]
mod t {
    use super::*;
    #[test]
    fn only_this_machine_is_a_host() {
        assert!(host_ok(Some("127.0.0.1:7788"), 7788) && host_ok(Some("localhost:7788"), 7788));
        for h in [None, Some("evil.example:7788"), Some("127.0.0.1:7789"), Some("127.0.0.1")] { assert!(!host_ok(h, 7788), "{h:?}"); }
    }
    #[test]
    fn a_change_needs_the_pages_header_and_no_foreign_origin() {
        let r = |h: Vec<(&str, &str)>| Request { method: "POST".into(), path: "/".into(), query: String::new(), headers: h.into_iter().map(|(a, b)| (a.into(), b.into())).collect(), body: vec![] };
        assert!(change_ok(&r(vec![("X-Trinetra", "1")]), 7788));
        assert!(change_ok(&r(vec![("X-Trinetra", "1"), ("Origin", "http://127.0.0.1:7788")]), 7788));
        assert!(!change_ok(&r(vec![]), 7788));
        assert!(!change_ok(&r(vec![("X-Trinetra", "1"), ("Origin", "https://evil.example")]), 7788));
    }
    #[test]
    fn query_values_are_decoded() {
        let r = Request { method: "GET".into(), path: "/".into(), query: "run=app%2Fnadir+hold&x=1".into(), headers: vec![], body: vec![] };
        assert_eq!(r.param("run").as_deref(), Some("app/nadir hold"));
        assert_eq!(r.param("y"), None);
    }
    #[test]
    fn text_is_escaped_for_a_page() { assert_eq!(esc("<b a=\"1\">&"), "&lt;b a=&quot;1&quot;&gt;&amp;"); }
}
