// SPDX-License-Identifier: MPL-2.0
// Copyright (c) 2026 Alexandru-Ionuț Chiuță (Alexio)
//! MIDI 1.0 message decoding and Standard MIDI File (SMF) reading.
//!
//! Written from the public MIDI 1.0 Detailed Specification (message table: status byte high
//! nibble 8..E, data bytes below 0x80, pitch bend = 14 bit little-endian pair, running status,
//! system real-time bytes F8..FF may appear anywhere) and the public Standard MIDI Files
//! specification (MThd/MTrk chunks, variable-length quantities, meta event FF 51 tempo,
//! sysex F0/F7, ticks per quarter note or SMPTE division). Nothing here talks to a device; the
//! studio hands raw bytes from the browser to [`decode`] / [`decode_stream`].
//!
//! Every function returns `Err`/warnings on malformed input and never panics.

use crate::liveinput::{InputEvent, InputSource};

/// A decoded channel or system message. `channel` is 0..15.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiMessage {
    NoteOff { channel: u8, note: u8, velocity: u8 },
    NoteOn { channel: u8, note: u8, velocity: u8 },
    PolyPressure { channel: u8, note: u8, pressure: u8 },
    ControlChange { channel: u8, controller: u8, value: u8 },
    ProgramChange { channel: u8, program: u8 },
    ChannelPressure { channel: u8, pressure: u8 },
    /// 14 bit value 0..16383, centre 8192.
    PitchBend { channel: u8, value: u16 },
    /// Any system message (status 0xF0..0xFF); carries no mapped data.
    System { status: u8 },
}

/// Number of data bytes that follow a status byte (`None` = variable / not a message).
fn data_len(status: u8) -> Option<usize> {
    match status {
        0x80..=0xBF | 0xE0..=0xEF => Some(2),
        0xC0..=0xDF => Some(1),
        0xF1 | 0xF3 => Some(1),
        0xF2 => Some(2),
        0xF6 | 0xF8..=0xFF => Some(0),
        _ => None,
    }
}

fn build(status: u8, d1: u8, d2: u8) -> MidiMessage {
    let channel = status & 0x0F;
    match status & 0xF0 {
        0x80 => MidiMessage::NoteOff { channel, note: d1, velocity: d2 },
        0x90 => MidiMessage::NoteOn { channel, note: d1, velocity: d2 },
        0xA0 => MidiMessage::PolyPressure { channel, note: d1, pressure: d2 },
        0xB0 => MidiMessage::ControlChange { channel, controller: d1, value: d2 },
        0xC0 => MidiMessage::ProgramChange { channel, program: d1 },
        0xD0 => MidiMessage::ChannelPressure { channel, pressure: d1 },
        0xE0 => MidiMessage::PitchBend { channel, value: (d1 as u16) | ((d2 as u16) << 7) },
        _ => MidiMessage::System { status },
    }
}

/// Decode exactly one complete message (as delivered by Web MIDI): status byte plus its data
/// bytes, nothing else. A note-on with velocity 0 is returned as `NoteOn` (see [`to_event`],
/// which treats it as a release).
pub fn decode(bytes: &[u8]) -> Result<MidiMessage, String> {
    let status = *bytes.first().ok_or("empty MIDI message")?;
    if status < 0x80 {
        return Err(format!("first byte {status:#04x} is not a status byte"));
    }
    let n = data_len(status).ok_or_else(|| format!("unsupported status {status:#04x}"))?;
    if bytes.len() != n + 1 {
        return Err(format!("status {status:#04x} needs {n} data byte(s), got {}", bytes.len() - 1));
    }
    if let Some(b) = bytes[1..].iter().find(|b| **b >= 0x80) {
        return Err(format!("data byte {b:#04x} has the high bit set"));
    }
    Ok(build(status, bytes.get(1).copied().unwrap_or(0), bytes.get(2).copied().unwrap_or(0)))
}

/// Decode a byte stream with running status. Real-time bytes (F8..FF) and system exclusive
/// blocks are skipped; broken pieces are skipped and reported in the second result.
pub fn decode_stream(bytes: &[u8]) -> (Vec<MidiMessage>, Vec<String>) {
    let (mut out, mut warn) = (Vec::new(), Vec::new());
    let (mut running, mut i) = (None::<u8>, 0usize);
    while i < bytes.len() {
        let b = bytes[i];
        if b >= 0xF8 {
            i += 1; // real time: allowed anywhere, does not disturb running status
            continue;
        }
        if b == 0xF0 {
            running = None;
            match bytes[i..].iter().position(|x| *x == 0xF7) {
                Some(k) => i += k + 1,
                None => {
                    warn.push("unterminated system exclusive message skipped".into());
                    break;
                }
            }
            continue;
        }
        let (status, mut at) = if b >= 0x80 { (b, i + 1) } else { match running { Some(s) => (s, i), None => { warn.push(format!("data byte {b:#04x} without status skipped")); i += 1; continue; } } };
        let Some(n) = data_len(status) else {
            warn.push(format!("status {status:#04x} skipped"));
            running = None;
            i = at;
            continue;
        };
        if status < 0xF0 { running = Some(status); } else { running = None; }
        let mut d = [0u8; 2];
        let mut ok = true;
        for slot in d.iter_mut().take(n) {
            // real-time bytes may be interleaved inside a message
            while at < bytes.len() && bytes[at] >= 0xF8 { at += 1; }
            match bytes.get(at) {
                Some(x) if *x < 0x80 => { *slot = *x; at += 1; }
                _ => { ok = false; break; }
            }
        }
        if ok {
            out.push(build(status, d[0], d[1]));
        } else {
            warn.push(format!("truncated message with status {status:#04x} skipped"));
        }
        i = at.max(i + 1);
    }
    (out, warn)
}

/// Convert a message to a device-independent input event (`None` for messages that have no
/// mapped meaning: program change, poly pressure, channel pressure, system messages).
/// ids: control change `"<channel 1..16>:<controller>"`, note `"<channel>:<note>"`, pitch bend `"<channel>"`.
pub fn to_event(m: &MidiMessage, time: f32) -> Option<InputEvent> {
    let ev = |source, id: String, value: f32| Some(InputEvent { source, id, value, time });
    match *m {
        MidiMessage::ControlChange { channel, controller, value } => ev(InputSource::MidiCc, format!("{}:{}", channel + 1, controller), value as f32 / 127.0),
        MidiMessage::NoteOn { channel, note, velocity } => ev(InputSource::MidiNote, format!("{}:{}", channel + 1, note), velocity as f32 / 127.0),
        MidiMessage::NoteOff { channel, note, .. } => ev(InputSource::MidiNote, format!("{}:{}", channel + 1, note), 0.0),
        MidiMessage::PitchBend { channel, value } => ev(InputSource::MidiPitchBend, format!("{}", channel + 1), value as f32 / 16383.0),
        _ => None,
    }
}

// ---- Standard MIDI File ----------------------------------------------------------------------

/// One channel message from a file with its absolute position.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SmfEvent {
    pub track: usize,
    pub tick: u64,
    pub seconds: f64,
    pub msg: MidiMessage,
}

struct Rd<'a> {
    b: &'a [u8],
    i: usize,
}

impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], String> {
        let end = self.i.checked_add(n).filter(|e| *e <= self.b.len()).ok_or("unexpected end of MIDI file")?;
        let s = &self.b[self.i..end];
        self.i = end;
        Ok(s)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u16(&mut self) -> Result<u16, String> {
        let s = self.take(2)?;
        Ok(u16::from_be_bytes([s[0], s[1]]))
    }
    fn u32(&mut self) -> Result<u32, String> {
        let s = self.take(4)?;
        Ok(u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
    }
    /// Variable-length quantity: 7 bits per byte, high bit = more, at most 4 bytes.
    fn vlq(&mut self) -> Result<u32, String> {
        let mut v = 0u32;
        for _ in 0..4 {
            let b = self.u8()?;
            v = (v << 7) | (b & 0x7F) as u32;
            if b & 0x80 == 0 {
                return Ok(v);
            }
        }
        Err("variable-length quantity longer than 4 bytes".into())
    }
}

const MAX_SMF_EVENTS: usize = 2_000_000;

/// Parse a format 0 or 1 Standard MIDI File into channel messages with absolute times
/// (tempo changes from any track are honoured; SMPTE divisions are supported).
pub fn parse_smf(bytes: &[u8]) -> Result<Vec<SmfEvent>, String> {
    if bytes.len() > 64 << 20 {
        return Err("MIDI file larger than 64 MB".into());
    }
    let mut r = Rd { b: bytes, i: 0 };
    if r.take(4)? != b"MThd" {
        return Err("not a Standard MIDI File (no MThd header)".into());
    }
    let hlen = r.u32()? as usize;
    if hlen < 6 {
        return Err("MThd chunk shorter than 6 bytes".into());
    }
    let (format, ntrks, division) = (r.u16()?, r.u16()?, r.u16()?);
    r.take(hlen - 6)?;
    if format > 1 {
        return Err(format!("SMF format {format} (independent sequences) is not supported; use format 0 or 1"));
    }
    // seconds per tick unless tempo-based
    let (tpq, smpte) = if division & 0x8000 != 0 {
        let fps = match (division >> 8) as u8 as i8 { -24 => 24.0, -25 => 25.0, -29 => 30000.0 / 1001.0, -30 => 30.0, o => return Err(format!("bad SMPTE frame rate {o}")) };
        let sub = (division & 0xFF) as f64;
        if sub == 0.0 {
            return Err("SMPTE division with 0 ticks per frame".into());
        }
        (0.0, Some(1.0 / (fps * sub)))
    } else {
        if division == 0 {
            return Err("division is 0 ticks per quarter note".into());
        }
        (division as f64, None)
    };

    let mut raw: Vec<(usize, u64, MidiMessage)> = Vec::new();
    let mut tempos: Vec<(u64, u32)> = Vec::new();
    let mut tracks = 0usize;
    while r.i < bytes.len() && tracks < ntrks as usize {
        if bytes.len() - r.i < 8 {
            break; // trailing junk
        }
        let id = r.take(4)?;
        let len = r.u32()? as usize;
        let body = r.take(len)?;
        if id != b"MTrk" {
            continue; // unknown chunk types are skipped, per the specification
        }
        let t = tracks;
        tracks += 1;
        let mut c = Rd { b: body, i: 0 };
        let (mut tick, mut running) = (0u64, None::<u8>);
        while c.i < body.len() {
            tick += c.vlq()? as u64;
            let first = c.u8()?;
            let status = if first >= 0x80 { first } else { c.i -= 1; running.ok_or("running status used before any status byte")? };
            match status {
                0xFF => {
                    running = None;
                    let ty = c.u8()?;
                    let n = c.vlq()? as usize;
                    let data = c.take(n)?;
                    if ty == 0x51 {
                        if n != 3 {
                            return Err("tempo meta event must carry 3 bytes".into());
                        }
                        let us = ((data[0] as u32) << 16) | ((data[1] as u32) << 8) | data[2] as u32;
                        if us == 0 {
                            return Err("tempo of 0 microseconds per quarter note".into());
                        }
                        tempos.push((tick, us));
                    }
                    if ty == 0x2F {
                        break;
                    }
                }
                0xF0 | 0xF7 => {
                    running = None;
                    let n = c.vlq()? as usize;
                    c.take(n)?;
                }
                0x80..=0xEF => {
                    running = Some(status);
                    let n = data_len(status).unwrap_or(2);
                    let d1 = c.u8()?;
                    let d2 = if n == 2 { c.u8()? } else { 0 };
                    if d1 >= 0x80 || d2 >= 0x80 {
                        return Err("data byte with the high bit set in a track".into());
                    }
                    if raw.len() >= MAX_SMF_EVENTS {
                        return Err("more than 2 million events in the MIDI file".into());
                    }
                    raw.push((t, tick, build(status, d1, d2)));
                }
                other => return Err(format!("unexpected status byte {other:#04x} in a track")),
            }
        }
    }
    if tracks == 0 {
        return Err("MIDI file contains no tracks".into());
    }
    tempos.sort_by_key(|t| t.0); // stable: same-tick changes keep file order, the last one wins
    // cumulative seconds at each tempo change
    let mut segs: Vec<(u64, f64, u32)> = vec![(0, 0.0, 500_000)]; // (tick, seconds at tick, tempo)
    for &(tk, us) in &tempos {
        let (lt, ls, lus) = *segs.last().expect("seeded");
        let secs = ls + match smpte { Some(s) => (tk - lt) as f64 * s, None => (tk - lt) as f64 * lus as f64 / 1e6 / tpq };
        segs.push((tk, secs, us));
    }
    let to_sec = |tk: u64| -> f64 {
        let k = segs.partition_point(|s| s.0 <= tk).saturating_sub(1);
        let (st, ss, us) = segs[k];
        ss + match smpte { Some(s) => (tk - st) as f64 * s, None => (tk - st) as f64 * us as f64 / 1e6 / tpq }
    };
    let mut out: Vec<SmfEvent> = raw.into_iter().map(|(track, tick, msg)| SmfEvent { track, tick, seconds: to_sec(tick), msg }).collect();
    out.sort_by_key(|e| e.tick); // stable
    Ok(out)
}

/// A `.mid` file as an offline "live" source: control change, note and pitch-bend events.
pub fn smf_to_events(bytes: &[u8]) -> Result<Vec<InputEvent>, String> {
    Ok(parse_smf(bytes)?.iter().filter_map(|e| to_event(&e.msg, e.seconds as f32)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_each_message_kind() {
        assert_eq!(decode(&[0x93, 60, 100]).unwrap(), MidiMessage::NoteOn { channel: 3, note: 60, velocity: 100 });
        assert_eq!(decode(&[0x83, 60, 5]).unwrap(), MidiMessage::NoteOff { channel: 3, note: 60, velocity: 5 });
        assert_eq!(decode(&[0xB0, 7, 127]).unwrap(), MidiMessage::ControlChange { channel: 0, controller: 7, value: 127 });
        assert_eq!(decode(&[0xC5, 9]).unwrap(), MidiMessage::ProgramChange { channel: 5, program: 9 });
        assert_eq!(decode(&[0xD1, 40]).unwrap(), MidiMessage::ChannelPressure { channel: 1, pressure: 40 });
        assert_eq!(decode(&[0xA2, 10, 11]).unwrap(), MidiMessage::PolyPressure { channel: 2, note: 10, pressure: 11 });
        // pitch bend: LSB first, 14 bit
        assert_eq!(decode(&[0xE0, 0x00, 0x40]).unwrap(), MidiMessage::PitchBend { channel: 0, value: 8192 });
        assert_eq!(decode(&[0xE2, 0x7F, 0x7F]).unwrap(), MidiMessage::PitchBend { channel: 2, value: 16383 });
        assert_eq!(decode(&[0xE0, 0x01, 0x00]).unwrap(), MidiMessage::PitchBend { channel: 0, value: 1 });
        assert_eq!(decode(&[0xF8]).unwrap(), MidiMessage::System { status: 0xF8 });
    }

    #[test]
    fn decode_rejects_malformed_input() {
        for bad in [&[][..], &[0x40, 1][..], &[0x90, 60][..], &[0x90, 60, 100, 1][..], &[0x90, 0x80, 1][..], &[0xB0, 1, 0xFF][..], &[0xF0][..], &[0xF4][..]] {
            assert!(decode(bad).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn stream_running_status_realtime_and_sysex() {
        // note on, running-status note on, a clock byte in the middle, sysex, CC
        let s = [0x90, 60, 100, 62, 90, 0xF8, 64, 0xF8, 80, 0xF0, 1, 2, 3, 0xF7, 0xB1, 1, 64];
        let (m, w) = decode_stream(&s);
        assert!(w.is_empty(), "{w:?}");
        assert_eq!(m, vec![
            MidiMessage::NoteOn { channel: 0, note: 60, velocity: 100 },
            MidiMessage::NoteOn { channel: 0, note: 62, velocity: 90 },
            MidiMessage::NoteOn { channel: 0, note: 64, velocity: 80 },
            MidiMessage::ControlChange { channel: 1, controller: 1, value: 64 },
        ]);
        // garbage: data byte first, truncated message, unterminated sysex
        let (m, w) = decode_stream(&[5, 0x90, 60, 0xB0, 1]);
        assert_eq!(m.len(), 0);
        assert_eq!(w.len(), 3, "{w:?}");
        let (_, w) = decode_stream(&[0xF0, 1, 2]);
        assert_eq!(w.len(), 1);
    }

    #[test]
    fn event_conversion() {
        let e = to_event(&decode(&[0xB2, 1, 127]).unwrap(), 1.5).unwrap();
        assert_eq!((e.source, e.id.as_str(), e.value, e.time), (InputSource::MidiCc, "3:1", 1.0, 1.5));
        let e = to_event(&decode(&[0x90, 60, 0]).unwrap(), 0.0).unwrap();
        assert_eq!(e.source, InputSource::MidiNote);
        assert!(e.value == 0.0, "velocity 0 note-on is a release");
        let e = to_event(&decode(&[0xE0, 0, 0x40]).unwrap(), 0.0).unwrap();
        assert!((e.value - 8192.0 / 16383.0).abs() < 1e-6);
        assert!(to_event(&decode(&[0xC0, 3]).unwrap(), 0.0).is_none());
    }

    fn vlq(mut v: u32) -> Vec<u8> {
        let mut out = vec![(v & 0x7F) as u8];
        v >>= 7;
        while v > 0 {
            out.push(((v & 0x7F) as u8) | 0x80);
            v >>= 7;
        }
        out.reverse();
        out
    }

    fn smf(format: u16, division: u16, tracks: &[Vec<u8>]) -> Vec<u8> {
        let mut f = b"MThd".to_vec();
        f.extend(6u32.to_be_bytes());
        f.extend(format.to_be_bytes());
        f.extend((tracks.len() as u16).to_be_bytes());
        f.extend(division.to_be_bytes());
        for t in tracks {
            f.extend(b"MTrk");
            f.extend((t.len() as u32).to_be_bytes());
            f.extend(t);
        }
        f
    }

    #[test]
    fn vlq_roundtrip_examples_from_the_specification() {
        for (v, b) in [(0u32, vec![0x00u8]), (0x7F, vec![0x7F]), (0x80, vec![0x81, 0x00]), (0x2000, vec![0xC0, 0x00]), (0x3FFF, vec![0xFF, 0x7F]), (0x4000, vec![0x81, 0x80, 0x00]), (0x0FFFFFFF, vec![0xFF, 0xFF, 0xFF, 0x7F])] {
            assert_eq!(vlq(v), b);
            assert_eq!(Rd { b: &b, i: 0 }.vlq().unwrap(), v);
        }
        assert!(Rd { b: &[0x80, 0x80, 0x80, 0x80, 0x01], i: 0 }.vlq().is_err());
    }

    #[test]
    fn smf_times_follow_tempo_changes_and_running_status() {
        let mut t = vec![];
        t.extend([0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20]); // 500000 us/qn = 120 bpm
        t.extend([0x00, 0xB0, 1, 0]); // CC1 = 0 at tick 0
        t.extend(vlq(480)); t.extend([1, 127]); // running status, 1 beat later: 0.5 s
        t.extend([0x00, 0xFF, 0x51, 0x03, 0x0F, 0x42, 0x40]); // tempo 1_000_000 from tick 480
        t.extend(vlq(480)); t.extend([0xB0, 1, 64]); // meta cancelled running status; one beat later at the slower tempo: 0.5 + 1.0
        t.extend(vlq(0)); t.extend([0x90, 60, 100]);
        t.extend(vlq(0)); t.extend([0xFF, 0x2F, 0x00]);
        let ev = parse_smf(&smf(0, 480, &[t])).unwrap();
        assert_eq!(ev.len(), 4);
        let s: Vec<f64> = ev.iter().map(|e| e.seconds).collect();
        assert!((s[0] - 0.0).abs() < 1e-9 && (s[1] - 0.5).abs() < 1e-9 && (s[2] - 1.5).abs() < 1e-9 && (s[3] - 1.5).abs() < 1e-9, "{s:?}");
        let ie = smf_to_events(&smf(0, 480, &[{
            let mut t = vec![0x00, 0xB0, 1, 127];
            t.extend([0x00, 0xFF, 0x2F, 0x00]);
            t
        }])).unwrap();
        assert_eq!(ie.len(), 1);
        assert_eq!(ie[0].id, "1:1");
        assert!((ie[0].value - 1.0).abs() < 1e-6);
    }

    #[test]
    fn smf_format1_merges_tracks_by_time_and_tempo_track_applies_to_all() {
        let mut tempo = vec![0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20];
        tempo.extend([0x00, 0xFF, 0x2F, 0x00]);
        let mut a = vec![]; // a note at tick 96 (quarter = 96)
        a.extend(vlq(96)); a.extend([0x90, 60, 100]);
        a.extend([0x00, 0xFF, 0x2F, 0x00]);
        let mut b = vec![];
        b.extend(vlq(48)); b.extend([0xB0, 7, 10]);
        b.extend([0x00, 0xFF, 0x2F, 0x00]);
        let ev = parse_smf(&smf(1, 96, &[tempo, a, b])).unwrap();
        assert_eq!(ev.len(), 2);
        assert_eq!(ev[0].track, 2);
        assert!((ev[0].seconds - 0.25).abs() < 1e-9 && (ev[1].seconds - 0.5).abs() < 1e-9);
    }

    #[test]
    fn smf_smpte_division_and_unknown_chunks() {
        let mut t = vec![];
        t.extend(vlq(25)); t.extend([0xB0, 1, 5]); // 25 ticks at 25 fps * 1 tick/frame = 1 s
        let mut f = smf(0, ((-25i8 as u8 as u16) << 8) | 1, &[t]);
        // an unknown chunk before the track must be skipped
        let mut junk = b"XXXX".to_vec();
        junk.extend(3u32.to_be_bytes());
        junk.extend([1, 2, 3]);
        f.splice(14..14, junk);
        let ev = parse_smf(&f).unwrap();
        assert!((ev[0].seconds - 1.0).abs() < 1e-9);
    }

    #[test]
    fn smf_errors_never_panic() {
        assert!(parse_smf(&[]).is_err());
        assert!(parse_smf(b"RIFFxxxxxxxxxxxxxxxxxxxx").is_err());
        assert!(parse_smf(&smf(2, 96, &[vec![0x00, 0xFF, 0x2F, 0x00]])).is_err(), "format 2");
        assert!(parse_smf(&smf(0, 0, &[vec![0x00, 0xFF, 0x2F, 0x00]])).is_err(), "zero division");
        assert!(parse_smf(&smf(0, 96, &[vec![0x00, 0x40, 1]])).is_err(), "running status without status");
        assert!(parse_smf(&smf(0, 96, &[vec![0x00, 0xB0, 1]])).is_err(), "truncated event");
        assert!(parse_smf(&smf(0, 96, &[vec![0x00, 0xFF, 0x51, 0x03, 0, 0, 0]])).is_err(), "zero tempo");
        assert!(parse_smf(&smf(0, 96, &[vec![0x00, 0xFF, 0x51, 0x02, 1, 1]])).is_err(), "short tempo");
        assert!(parse_smf(&smf(0, 96, &[vec![0x80, 0x80, 0x80, 0x80, 0x01, 0xB0, 1, 1]])).is_err(), "5 byte delta");
        assert!(parse_smf(&smf(0, 96, &[vec![0x00, 0xF0, 0x7F, 1]])).is_err(), "sysex longer than data");
        assert!(parse_smf(&smf(0, 96, &[])).is_err(), "no tracks");
        // truncation at every length of a valid file: Ok or Err, never a panic
        let mut t = vec![0x00, 0xFF, 0x51, 0x03, 0x07, 0xA1, 0x20, 0x00, 0xB0, 1, 9];
        t.extend([0x00, 0xFF, 0x2F, 0x00]);
        let f = smf(0, 96, &[t]);
        for n in 0..f.len() {
            let _ = parse_smf(&f[..n]);
        }
        // arbitrary bytes
        for seed in 0..200u32 {
            let mut x = seed.wrapping_mul(2654435761).wrapping_add(1);
            let junk: Vec<u8> = (0..64).map(|_| { x = x.wrapping_mul(1664525).wrapping_add(1013904223); (x >> 24) as u8 }).collect();
            let _ = parse_smf(&junk);
            let _ = decode_stream(&junk);
        }
    }
}
