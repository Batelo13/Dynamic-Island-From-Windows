//! Now-playing source (media-now-playing D1/D2): the Windows "current" media session, event-driven.
//! A dedicated MTA thread owns the WinRT objects; event handlers only nudge a channel, the thread
//! rebuilds a `Snapshot` and posts `notify` to the UI window, which reads `latest`.
use crate::{log, theme};
use std::{
    sync::{mpsc, Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use windows::{
    Foundation::TypedEventHandler,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session, GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    },
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        Graphics::Imaging::{
            CLSID_WICImagingFactory, GUID_WICPixelFormat32bppPBGRA, IWICImagingFactory, WICBitmapDitherTypeNone, WICBitmapInterpolationModeHighQualityCubic,
            WICBitmapPaletteTypeCustom, WICDecodeMetadataCacheOnDemand,
        },
        System::{
            Com::{CoCreateInstance, CoInitializeEx, IStream, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED},
            WinRT::CreateStreamOverRandomAccessStream,
        },
        UI::WindowsAndMessaging::PostMessageW,
    },
    core::Result,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Previous,
    TogglePlayPause,
    Next,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub title: String,
    pub artist: String,
    pub playing: bool,
    /// Enabled commands: previous, play/pause, next.
    pub can: [bool; 3],
    /// Playback position when `at` was taken; zero `duration` = unknown (live streams).
    pub position: Duration,
    pub duration: Duration,
    pub at: Instant,
    /// `theme::ART_PX`² BGRA premultiplied; shared so redraws don't copy it.
    pub art: Option<Arc<Vec<u8>>>,
}

impl Snapshot {
    /// Position extrapolated to now while playing, clamped to the duration.
    pub fn position_now(&self) -> Duration {
        let p = if self.playing { self.position + self.at.elapsed() } else { self.position };
        if self.duration.is_zero() { p } else { p.min(self.duration) }
    }

    /// What the drawn surfaces depend on (bars/progress live in their own visuals and don't count).
    pub fn content_key(&self) -> (String, String, bool, usize, [bool; 3], bool) {
        let art = self.art.as_ref().map_or(0, |a| Arc::as_ptr(a) as usize);
        (self.title.clone(), self.artist.clone(), self.playing, art, self.can, self.duration.is_zero())
    }
}

/// Last decoded art, keyed by (title, artist): decoded again only when the track changes.
type ArtCache = Option<((String, String), Option<Arc<Vec<u8>>>)>;

/// A WinRT event handler that only nudges the media thread (all work happens there).
fn nudge<S: windows::core::RuntimeType + 'static, A: windows::core::RuntimeType + 'static>(tx: &mpsc::Sender<Msg>, m: fn() -> Msg) -> TypedEventHandler<S, A> {
    let tx = tx.clone();
    TypedEventHandler::new(move |_, _| {
        let _ = tx.send(m());
        Ok(())
    })
}

enum Msg {
    /// Current session (or the session list) changed: resubscribe, then refresh.
    Session,
    Refresh,
    Command(Command),
}

pub struct Media {
    tx: mpsc::Sender<Msg>,
    pub latest: Arc<Mutex<Option<Snapshot>>>,
}

impl Media {
    /// Fire-and-forget; the resulting state arrives through `PlaybackInfoChanged`.
    pub fn command(&self, c: Command) {
        let _ = self.tx.send(Msg::Command(c));
    }
}

/// Starts the media thread. `notify` is posted to `hwnd` whenever `latest` changes.
pub fn start(hwnd: HWND, notify: u32) -> Media {
    let (tx, rx) = mpsc::channel();
    let latest = Arc::new(Mutex::new(None));
    let (thread_tx, thread_latest) = (tx.clone(), latest.clone());
    let hwnd = hwnd.0 as isize; // HWND isn't Send; it's just a handle value
    std::thread::spawn(move || {
        if let Err(e) = run(rx, thread_tx, thread_latest, hwnd, notify) {
            log::warn(&format!("media unavailable: {e}")); // spec: app keeps working without music
        }
    });
    Media { tx, latest }
}

fn run(rx: mpsc::Receiver<Msg>, tx: mpsc::Sender<Msg>, latest: Arc<Mutex<Option<Snapshot>>>, hwnd: isize, notify: u32) -> Result<()> {
    unsafe { CoInitializeEx(None, COINIT_MULTITHREADED).ok()? };
    let manager = Manager::RequestAsync()?.join()?;
    let wic: IWICImagingFactory = unsafe { CoCreateInstance(&CLSID_WICImagingFactory, None, CLSCTX_INPROC_SERVER)? };
    manager.CurrentSessionChanged(&nudge(&tx, || Msg::Session))?;
    manager.SessionsChanged(&nudge(&tx, || Msg::Session))?;

    let mut session: Option<(Session, [i64; 3])> = None;
    let mut art_cache: ArtCache = None;
    let mut msg = Msg::Session; // initial state
    loop {
        match msg {
            Msg::Session => {
                if let Some((old, [a, b, c])) = session.take() {
                    let _ = old.RemoveMediaPropertiesChanged(a);
                    let _ = old.RemovePlaybackInfoChanged(b);
                    let _ = old.RemoveTimelinePropertiesChanged(c);
                }
                if let Ok(s) = manager.GetCurrentSession() {
                    let tokens = [
                        s.MediaPropertiesChanged(&nudge(&tx, || Msg::Refresh))?,
                        s.PlaybackInfoChanged(&nudge(&tx, || Msg::Refresh))?,
                        s.TimelinePropertiesChanged(&nudge(&tx, || Msg::Refresh))?,
                    ];
                    session = Some((s, tokens));
                }
                publish(&latest, snapshot(session.as_ref().map(|s| &s.0), &wic, &mut art_cache), hwnd, notify);
            }
            Msg::Refresh => publish(&latest, snapshot(session.as_ref().map(|s| &s.0), &wic, &mut art_cache), hwnd, notify),
            Msg::Command(c) => {
                if let Some((s, _)) = &session {
                    let _ = match c {
                        Command::Previous => s.TrySkipPreviousAsync(),
                        Command::TogglePlayPause => s.TryTogglePlayPauseAsync(),
                        Command::Next => s.TrySkipNextAsync(),
                    };
                }
            }
        }
        // Blocks until the next event or command: no polling.
        match rx.recv() {
            Ok(m) => msg = m,
            Err(_) => return Ok(()),
        }
    }
}

/// `None` = no session (or it couldn't be read: players briefly report errors while switching tracks).
fn snapshot(session: Option<&Session>, wic: &IWICImagingFactory, art_cache: &mut ArtCache) -> Option<Snapshot> {
    let s = session?;
    let props = s.TryGetMediaPropertiesAsync().ok()?.join().ok()?;
    let info = s.GetPlaybackInfo().ok()?;
    let status = info.PlaybackStatus().ok()?;
    if status == Status::Closed {
        return None;
    }
    let (title, artist) = (props.Title().ok()?.to_string(), props.Artist().map(|a| a.to_string()).unwrap_or_default());
    let can = info.Controls().map_or([false; 3], |c| {
        [c.IsPreviousEnabled().unwrap_or(false), c.IsPlayPauseToggleEnabled().unwrap_or(false), c.IsNextEnabled().unwrap_or(false)]
    });
    let playing = status == Status::Playing;
    let (position, duration, at) = timeline(s, playing);
    // Decode the art only when the track changes (D2).
    let key = (title.clone(), artist.clone());
    let art = match art_cache {
        Some((k, art)) if *k == key => art.clone(),
        _ => {
            let art = props.Thumbnail().ok().and_then(|t| decode_art(&t, wic).map_err(|e| log::warn(&format!("album art: {e}"))).ok()).map(Arc::new);
            *art_cache = Some((key, art.clone()));
            art
        }
    };
    Some(Snapshot { title, artist, playing, can, position, duration, at, art })
}

/// Position/duration, with the position moved forward from the player's last update to now.
fn timeline(s: &Session, playing: bool) -> (Duration, Duration, Instant) {
    let now = Instant::now();
    let Ok(t) = s.GetTimelineProperties() else { return (Duration::ZERO, Duration::ZERO, now) };
    let ticks = |v: i64| Duration::from_nanos(v.max(0) as u64 * 100);
    let start = t.StartTime().map_or(0, |v| v.Duration);
    let duration = ticks(t.EndTime().map_or(0, |v| v.Duration) - start);
    let mut position = ticks(t.Position().map_or(0, |v| v.Duration) - start);
    if playing && let Ok(updated) = t.LastUpdatedTime() {
        // DateTime: 100 ns ticks since 1601-01-01 UTC.
        const EPOCH_1601_TO_1970: i64 = 116_444_736_000_000_000;
        let now_ticks = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| (d.as_nanos() / 100) as i64) + EPOCH_1601_TO_1970;
        let since = now_ticks - updated.UniversalTime;
        if (0..10 * 60 * 10_000_000).contains(&since) {
            position += ticks(since); // ignore absurd gaps (stale/zero timestamps)
        }
    }
    (position, duration, now)
}

/// Thumbnail stream → WIC → `ART_PX`² premultiplied BGRA.
/// ponytail: non-square art is stretched to the square; crop-to-fill if a player sends wide images.
fn decode_art(thumb: &windows::Storage::Streams::IRandomAccessStreamReference, wic: &IWICImagingFactory) -> Result<Vec<u8>> {
    let stream = thumb.OpenReadAsync()?.join()?;
    let n = theme::ART_PX;
    let mut pixels = vec![0u8; (n * n * 4) as usize];
    unsafe {
        let istream: IStream = CreateStreamOverRandomAccessStream(&stream)?;
        let decoder = wic.CreateDecoderFromStream(&istream, std::ptr::null(), WICDecodeMetadataCacheOnDemand)?;
        let scaler = wic.CreateBitmapScaler()?;
        scaler.Initialize(&decoder.GetFrame(0)?, n, n, WICBitmapInterpolationModeHighQualityCubic)?;
        let converter = wic.CreateFormatConverter()?;
        converter.Initialize(&scaler, &GUID_WICPixelFormat32bppPBGRA, WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeCustom)?;
        converter.CopyPixels(std::ptr::null(), n * 4, &mut pixels)?;
    }
    Ok(pixels)
}

fn publish(latest: &Mutex<Option<Snapshot>>, snap: Option<Snapshot>, hwnd: isize, notify: u32) {
    if let Ok(mut l) = latest.lock() {
        *l = snap;
    }
    let _ = unsafe { PostMessageW(Some(HWND(hwnd as *mut _)), notify, WPARAM(0), LPARAM(0)) };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(playing: bool, position: u64, duration: u64) -> Snapshot {
        Snapshot {
            title: "t".into(),
            artist: "a".into(),
            playing,
            can: [true; 3],
            position: Duration::from_secs(position),
            duration: Duration::from_secs(duration),
            at: Instant::now() - Duration::from_secs(5),
            art: None,
        }
    }

    #[test]
    fn position_advances_only_while_playing_and_clamps() {
        assert!(snap(true, 10, 200).position_now() >= Duration::from_secs(15));
        assert_eq!(snap(false, 10, 200).position_now(), Duration::from_secs(10));
        assert_eq!(snap(true, 198, 200).position_now(), Duration::from_secs(200));
        assert!(snap(true, 10, 0).position_now() >= Duration::from_secs(15)); // unknown duration: no clamp
    }

    #[test]
    fn content_key_ignores_position() {
        let (a, mut b) = (snap(true, 10, 200), snap(true, 99, 200));
        assert_eq!(a.content_key(), b.content_key());
        b.playing = false;
        assert_ne!(a.content_key(), b.content_key());
    }
}
