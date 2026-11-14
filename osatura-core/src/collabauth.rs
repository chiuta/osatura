// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! Per-participant collaboration tokens (R-111 hardening, wave 7).
//!
//! Instead of one bearer value that everybody shares, the host keeps one SECRET (a file only the host can read) and
//! hands each participant a token that is derived from it:
//!
//! ```text
//! ap1.<name>.<expires>.<mac>      mac = HMAC-SHA-256(secret, "osatura-collab-token/1\n<name>\n<expires>")  (hex)
//! ```
//!
//! * `name` is 1 to 32 characters of `A-Z a-z 0-9 _ -` (it becomes the participant's display name, so one person
//!   cannot join as another);
//! * `expires` is the last second (Unix time) at which the token is accepted;
//! * the token needs no server-side table: the server recomputes the MAC from the secret, compares in constant time and
//!   checks the clock. A leaked token is therefore limited to one name and one deadline; to revoke it earlier the host
//!   lists the name (`--collab-revoke`) or changes the secret.
//!
//! HMAC is RFC 2104 over the project's own SHA-256 ([`crate::vcs::sha256`], tested against the FIPS 180 vectors); the
//! tests below run the RFC 4231 HMAC-SHA-256 vectors. Nothing here uses randomness or a clock by itself: `mint_token`
//! and `verify_token` take the time as an argument.

use crate::vcs::sha256;

const BLOCK: usize = 64;

/// HMAC-SHA-256 (RFC 2104) of `msg` under `key`.
pub fn hmac_sha256(key: &[u8], msg: &[u8]) -> [u8; 32] {
    let mut k = [0u8; BLOCK];
    if key.len() > BLOCK {
        k[..32].copy_from_slice(&sha256(key));
    } else {
        k[..key.len()].copy_from_slice(key);
    }
    let mut inner = Vec::with_capacity(BLOCK + msg.len());
    inner.extend(k.iter().map(|b| b ^ 0x36));
    inner.extend_from_slice(msg);
    let ih = sha256(&inner);
    let mut outer = Vec::with_capacity(BLOCK + 32);
    outer.extend(k.iter().map(|b| b ^ 0x5c));
    outer.extend_from_slice(&ih);
    sha256(&outer)
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Constant-time equality of two byte strings of any length (a length difference is not hidden, the content is).
fn ct_eq_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.iter().zip(b).fold(0u8, |d, (x, y)| d | (x ^ y)) == 0
}

/// Shortest accepted server secret.
pub const MIN_SECRET_LEN: usize = 16;
/// Longest lifetime `mint_token` allows (30 days).
pub const MAX_TTL_SECS: u64 = 30 * 24 * 3600;
/// Most characters of a participant name.
pub const MAX_NAME_LEN: usize = 32;

/// What a verified token says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Claims {
    pub name: String,
    /// Last second (Unix time) at which the token is valid.
    pub expires: u64,
}

/// Why a token was refused.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenError {
    /// Not of the form `ap1.<name>.<expires>.<mac>`.
    Malformed,
    /// The MAC does not match (wrong secret or a changed name / deadline).
    BadSignature,
    /// Valid signature, but the deadline has passed.
    Expired { expires: u64 },
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::Malformed => write!(f, "not a participant token"),
            TokenError::BadSignature => write!(f, "wrong signature"),
            TokenError::Expired { expires } => write!(f, "expired (valid until Unix time {expires})"),
        }
    }
}

/// Checks a participant name.
pub fn check_name(name: &str) -> Result<(), String> {
    if name.is_empty() || name.chars().count() > MAX_NAME_LEN || !name.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_' || c == b'-') {
        return Err(format!("a participant name is 1 to {MAX_NAME_LEN} characters of letters, digits, _ and - (got '{name}')"));
    }
    Ok(())
}

/// Checks a server secret (length only: it should be random, e.g. `openssl rand -hex 32`).
pub fn check_secret(secret: &str) -> Result<(), String> {
    if secret.trim().len() < MIN_SECRET_LEN {
        return Err(format!("the collaboration secret must be at least {MIN_SECRET_LEN} characters (e.g. `openssl rand -hex 32`)"));
    }
    Ok(())
}

fn mac(secret: &str, name: &str, expires: u64) -> String {
    hex(&hmac_sha256(secret.trim().as_bytes(), format!("osatura-collab-token/1\n{name}\n{expires}").as_bytes()))
}

/// A token for `name`, valid for `ttl_secs` from `now` (Unix seconds). TTL 1 second to [`MAX_TTL_SECS`].
pub fn mint_token(secret: &str, name: &str, now: u64, ttl_secs: u64) -> Result<String, String> {
    check_secret(secret)?;
    check_name(name)?;
    if ttl_secs == 0 || ttl_secs > MAX_TTL_SECS {
        return Err(format!("the lifetime must be between 1 second and {MAX_TTL_SECS} seconds (30 days)"));
    }
    let expires = now.checked_add(ttl_secs).ok_or("lifetime overflow")?;
    Ok(format!("ap1.{name}.{expires}.{}", mac(secret, name, expires)))
}

/// Verifies `token` against `secret` at time `now` (Unix seconds).
pub fn verify_token(secret: &str, token: &str, now: u64) -> Result<Claims, TokenError> {
    let mut parts = token.split('.');
    let (Some("ap1"), Some(name), Some(exp), Some(sig), None) = (parts.next(), parts.next(), parts.next(), parts.next(), parts.next()) else {
        return Err(TokenError::Malformed);
    };
    if check_name(name).is_err() || exp.is_empty() || exp.len() > 20 || !exp.bytes().all(|c| c.is_ascii_digit()) || sig.len() != 64 {
        return Err(TokenError::Malformed);
    }
    let expires: u64 = exp.parse().map_err(|_| TokenError::Malformed)?;
    // one spelling per token: "0100" and "100" would otherwise carry the same signature
    if exp != expires.to_string() {
        return Err(TokenError::Malformed);
    }
    if !ct_eq_bytes(sig.as_bytes(), mac(secret, name, expires).as_bytes()) {
        return Err(TokenError::BadSignature);
    }
    if now > expires {
        return Err(TokenError::Expired { expires });
    }
    Ok(Claims { name: name.to_string(), expires })
}

/// A token bucket: `rate` tokens per second, at most `burst` stored. Time is passed in (milliseconds), so it is testable.
#[derive(Clone, Copy, Debug)]
pub struct Bucket {
    tokens: f64,
    last_ms: u64,
}

impl Bucket {
    pub fn new(burst: f64, now_ms: u64) -> Bucket {
        Bucket { tokens: burst, last_ms: now_ms }
    }

    /// Time of the last update (milliseconds, the caller's clock).
    pub fn last_ms(&self) -> u64 {
        self.last_ms
    }

    /// Takes one token; false (and nothing taken) when the bucket is empty. `rate`: tokens per second.
    pub fn take(&mut self, rate: f64, burst: f64, now_ms: u64) -> bool {
        let dt = now_ms.saturating_sub(self.last_ms) as f64 / 1000.0;
        self.last_ms = self.last_ms.max(now_ms);
        self.tokens = (self.tokens + dt * rate).min(burst);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(s: &str) -> Vec<u8> {
        (0..s.len() / 2).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).unwrap()).collect()
    }

    /// RFC 4231 test cases 1, 2, 3, 6 (long key) and 7 (long key and data).
    #[test]
    fn hmac_sha256_matches_rfc_4231() {
        assert_eq!(hex(&hmac_sha256(&[0x0b; 20], b"Hi There")), "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7");
        assert_eq!(hex(&hmac_sha256(b"Jefe", b"what do ya want for nothing?")), "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843");
        assert_eq!(hex(&hmac_sha256(&[0xaa; 20], &[0xdd; 50])), "773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe");
        assert_eq!(
            hex(&hmac_sha256(&[0xaa; 131], b"Test Using Larger Than Block-Size Key - Hash Key First")),
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54"
        );
        assert_eq!(
            hex(&hmac_sha256(&[0xaa; 131], b"This is a test using a larger than block-size key and a larger than block-size data. The key needs to be hashed before being used by the HMAC algorithm.")),
            "9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2"
        );
        let k = h("0102030405060708090a0b0c0d0e0f10111213141516171819");
        assert_eq!(hex(&hmac_sha256(&k, &[0xcd; 50])), "82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b");
    }

    const SECRET: &str = "0123456789abcdef0123456789abcdef";

    #[test]
    fn tokens_round_trip_and_expire() {
        let t = mint_token(SECRET, "ana-2", 1_000_000, 3600).unwrap();
        assert!(t.starts_with("ap1.ana-2.1003600."), "{t}");
        assert!(t.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'.' || c == b'-'), "cookie-safe: {t}");
        assert_eq!(verify_token(SECRET, &t, 1_000_000).unwrap(), Claims { name: "ana-2".into(), expires: 1_003_600 });
        assert!(verify_token(SECRET, &t, 1_003_600).is_ok(), "valid up to and including the deadline");
        assert_eq!(verify_token(SECRET, &t, 1_003_601), Err(TokenError::Expired { expires: 1_003_600 }));
        // tokens are deterministic for the same inputs
        assert_eq!(t, mint_token(SECRET, "ana-2", 1_000_000, 3600).unwrap());
        // a surrounding newline in the secret file does not matter
        assert!(verify_token(&format!("{SECRET}\n"), &t, 5).is_ok());
    }

    #[test]
    fn forged_or_changed_tokens_are_refused() {
        let t = mint_token(SECRET, "ana", 1000, 600).unwrap();
        // another secret
        assert_eq!(verify_token("ffffffffffffffffffffffffffffffff", &t, 1001), Err(TokenError::BadSignature));
        // another name or deadline with the old signature
        assert_eq!(verify_token(SECRET, &t.replacen("ana", "bob", 1), 1001), Err(TokenError::BadSignature));
        assert_eq!(verify_token(SECRET, &t.replacen("1600", "9999999", 1), 1001), Err(TokenError::BadSignature));
        // a flipped signature character
        let mut bad = t.clone();
        let last = bad.pop().unwrap();
        bad.push(if last == '0' { '1' } else { '0' });
        assert_eq!(verify_token(SECRET, &bad, 1001), Err(TokenError::BadSignature));
        // shape errors
        for m in ["", "ana", "ap1.ana.1600", "ap2.ana.1600.00", "ap1..1600.0", "ap1.ana.x.0", "ap1.a b.1600.0", "ap1.ana.1600.short", &format!("{t}.extra")] {
            assert_eq!(verify_token(SECRET, m, 1001), Err(TokenError::Malformed), "{m}");
        }
        // the shared bearer of the old scheme is not a participant token
        assert_eq!(verify_token(SECRET, SECRET, 1), Err(TokenError::Malformed));
    }

    /// The signature is checked before the deadline or any revocation list is consulted: whoever does not hold a
    /// correctly signed token learns nothing about names, deadlines or revocations from the answer.
    #[test]
    fn the_signature_is_judged_before_the_deadline() {
        let t = mint_token(SECRET, "ana", 1000, 600).unwrap();
        // long past its deadline AND forged: the answer is "bad signature", never "expired"
        let forged = t.replacen("ana", "bob", 1);
        assert_eq!(verify_token(SECRET, &forged, 9_999_999), Err(TokenError::BadSignature));
        assert_eq!(verify_token("ffffffffffffffffffffffffffffffff", &t, 9_999_999), Err(TokenError::BadSignature));
        // the genuine token past its deadline is the only "expired" answer
        assert_eq!(verify_token(SECRET, &t, 9_999_999), Err(TokenError::Expired { expires: 1600 }));
    }

    /// `.` separates the fields and cannot occur inside a name, so no token can be read in two ways.
    #[test]
    fn separators_cannot_hide_in_a_name_and_the_deadline_has_one_spelling() {
        let mac_of = |name: &str, exp: u64| mac(SECRET, name, exp);
        // a name with a dot is refused when minting, and a token built by hand with one is malformed (6 fields)
        assert!(mint_token(SECRET, "j.doe", 0, 60).is_err());
        assert_eq!(verify_token(SECRET, &format!("ap1.j.doe.1600.{}", mac_of("j.doe", 1600)), 1001), Err(TokenError::Malformed));
        assert_eq!(verify_token(SECRET, "ap1.x.9999999999.deadbeef", 1001), Err(TokenError::Malformed));
        // the same deadline written with a leading zero is a different string and is refused, although the MAC would match
        let t = mint_token(SECRET, "ana", 1000, 600).unwrap();
        assert!(verify_token(SECRET, &t, 1001).is_ok());
        let padded = t.replacen(".1600.", ".01600.", 1);
        assert_eq!(verify_token(SECRET, &padded, 1001), Err(TokenError::Malformed));
        // a newline in the name cannot shift the fields of the signed text (name and deadline are separated by "\n")
        assert_eq!(verify_token(SECRET, &format!("ap1.ana\n1600.1600.{}", mac_of("ana", 1600)), 1001), Err(TokenError::Malformed));
    }

    #[test]
    fn minting_validates_its_arguments() {
        assert!(mint_token("short", "ana", 0, 60).unwrap_err().contains("at least 16"));
        for n in ["", "a.b", "a b", "ä", &"x".repeat(33)] {
            assert!(mint_token(SECRET, n, 0, 60).is_err(), "{n}");
        }
        assert!(mint_token(SECRET, "ana", 0, 0).is_err());
        assert!(mint_token(SECRET, "ana", 0, MAX_TTL_SECS + 1).is_err());
        assert!(mint_token(SECRET, "ana", 0, MAX_TTL_SECS).is_ok());
        assert!(mint_token(SECRET, "ana", u64::MAX, 60).is_err(), "overflow is an error, not a wrap");
    }

    #[test]
    fn bucket_allows_a_burst_then_the_sustained_rate() {
        let (rate, burst) = (10.0, 20.0);
        let mut b = Bucket::new(burst, 0);
        let allowed = (0..50).filter(|_| b.take(rate, burst, 0)).count();
        assert_eq!(allowed, 20, "the burst, then nothing at the same instant");
        assert!(!b.take(rate, burst, 50), "50 ms later: half a token");
        assert!(b.take(rate, burst, 100), "100 ms later: one token");
        // after a long pause the bucket is full again but never beyond the burst
        let mut n = 0;
        while b.take(rate, burst, 60_000) {
            n += 1;
        }
        assert_eq!(n, 20);
        // a clock that goes back does not mint tokens
        assert!(!b.take(rate, burst, 10));
    }
}
