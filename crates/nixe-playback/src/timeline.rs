//! Join a single media version's files into mpv's continuous seekable timeline.
//! Sources may contain credentials: never persist or log the returned string.

/// Keep a single file unchanged. Escape every multi-file entry using its byte
/// length, so commas, semicolons, Unicode and query values remain URL contents.
/// See https://github.com/mpv-player/mpv/blob/master/DOCS/edl-mpv.rst.
pub fn concatenate<'a>(sources: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let sources: Vec<_> = sources.into_iter().collect();
    if sources.is_empty() || sources.iter().any(|source| source.is_empty() || source.contains('\0')) {
        return None;
    }
    if sources.len() == 1 {
        return Some(sources[0].to_owned());
    }
    let entries: Vec<_> = sources.iter().map(|source| format!("%{}%{source}", source.len())).collect();
    Some(format!("edl://{}", entries.join(";")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mpv::{Event, Mpv};
    use std::time::{Duration, Instant};

    #[test]
    fn mpv_joins_parts_and_seeks_across_the_boundary() {
        let directory = std::env::temp_dir().join(format!("nixe-timeline-{}", std::process::id()));
        std::fs::create_dir_all(&directory).unwrap();
        // Two seconds of mono PCM per part; no display, audio device or server.
        let mut wav = b"RIFF".to_vec();
        wav.extend_from_slice(&(36u32 + 32000).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes());
        wav.extend_from_slice(&8000u32.to_le_bytes());
        wav.extend_from_slice(&16000u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&32000u32.to_le_bytes());
        wav.resize(wav.len() + 32000, 0);
        let first = directory.join("first,;é.wav");
        let second = directory.join("second.wav");
        std::fs::write(&first, &wav).unwrap();
        std::fs::write(&second, &wav).unwrap();
        let first = first.to_str().unwrap();
        let second = second.to_str().unwrap();
        let timeline = concatenate([first, second]).unwrap();
        let (mpv, _) =
            Mpv::new(&[("vo", "null"), ("ao", "null"), ("pause", "yes"), ("idle", "yes"), ("config", "no"), ("terminal", "no")])
                .unwrap();
        mpv.command(&["loadfile", &timeline, "replace"]).unwrap();
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match mpv.next_event() {
                Some(Event::FileLoaded) => break,
                Some(Event::EndFile { error: Some(_), .. }) => panic!("The joined fixture failed to load"),
                _ => {}
            }
            assert!(Instant::now() < deadline, "Timed out loading the joined fixture");
            std::thread::sleep(Duration::from_millis(5));
        }
        assert!((mpv.get_f64("duration").unwrap() - 4.0).abs() < 0.05);
        for target in [2.5, 0.5] {
            mpv.command(&["seek", &target.to_string(), "absolute+exact"]).unwrap();
            let deadline = Instant::now() + Duration::from_secs(3);
            while mpv.get_f64("time-pos").is_none_or(|position| (position - target).abs() > 0.1) {
                assert!(Instant::now() < deadline, "Timed out seeking across the part boundary");
                while mpv.next_event().is_some() {}
                std::thread::sleep(Duration::from_millis(5));
            }
        }
        mpv.command(&["sub-add", "memory://1\n00:00:00,000 --> 00:00:01,000\nSubtitle fixture\n", "select", "English", "eng"])
            .unwrap();
        assert!(mpv.get_i64("sid").is_some_and(|id| id > 0));
        mpv.set_str("sid", "no").unwrap();
        assert_eq!(mpv.get_str("sid").as_deref(), Some("no"));
        drop(mpv);
        std::fs::remove_dir_all(directory).unwrap();
    }
}
