//! Watching a region of the screen.

mod dxgi;
mod staging;

use std::time::{Duration, Instant};

use imperifex_bench_domain::desktop::Rect;
use imperifex_bench_domain::ports::{ScreenWatch, Watch};
use imperifex_bench_domain::qpc::qpc_to_duration;
use imperifex_bench_domain::screen::ScreenFrame;
use windows::Win32::Graphics::Direct3D11::ID3D11Device;
use windows::Win32::Graphics::Dxgi::{
    IDXGIOutputDuplication, DXGI_ERROR_ACCESS_LOST, DXGI_ERROR_WAIT_TIMEOUT,
};

use crate::qpc_clock::frequency;
use staging::Staging;

fn failed(what: &str) -> impl Fn(windows::core::Error) -> String + '_ {
    move |error| format!("{what} falló: {error}")
}

/// The real screen.
#[derive(Debug, Default, Clone, Copy)]
pub struct DxgiScreen;

impl ScreenWatch for DxgiScreen {
    fn watch(&self, region: Rect) -> Result<Box<dyn Watch>, String> {
        Ok(Box::new(DxgiWatch::new(region)?))
    }
}

struct DxgiWatch {
    region: Rect,
    device: ID3D11Device,
    /// `None` only while it is being made again.
    duplication: Option<IDXGIOutputDuplication>,
    staging: Staging,
    found: dxgi::Found,
    frequency: i64,
    /// A lost duplication is made again this once.
    recreated: bool,
}

impl DxgiWatch {
    fn new(region: Rect) -> Result<Self, String> {
        let found = dxgi::monitor_with(region)?;
        let (device, context) = dxgi::device_for(&found)?;
        let duplication = dxgi::duplicate(&found, &device)?;
        // SAFETY: the duplication is alive; the description is returned by value.
        let format = unsafe { duplication.GetDesc() }.ModeDesc.Format;
        // `monitor_with` proved the region has a positive size inside the monitor.
        let size = (region.w.unsigned_abs(), region.h.unsigned_abs());
        let staging = Staging::new(&device, context, format, found.corner, size)?;
        Ok(Self {
            region,
            device,
            duplication: Some(duplication),
            staging,
            found,
            frequency: frequency(),
            recreated: false,
        })
    }

    /// A monitor allows one duplication per program, so the lost one goes first.
    fn recreate(&mut self) -> Result<(), String> {
        if self.recreated {
            return Err(format!(
                "se perdió dos veces la duplicación de la región {:?}",
                self.region
            ));
        }
        self.recreated = true;
        self.duplication = None;
        self.duplication = Some(dxgi::duplicate(&self.found, &self.device)?);
        Ok(())
    }

    /// One update: `Ok(None)` for one that is not a picture, or for none.
    fn take(&mut self, wait_ms: u32) -> Result<Option<ScreenFrame>, String> {
        let Some(duplication) = &self.duplication else {
            return Err("no hay duplicación de la pantalla".into());
        };
        // The frame is turned into data, and released, before anything else.
        let acquired = dxgi::acquire(duplication, wait_ms)
            .map(|held| picture(&held, &self.staging, self.frequency));
        match acquired {
            Ok(read) => read,
            Err(error) if error.code() == DXGI_ERROR_WAIT_TIMEOUT => Ok(None),
            Err(error) if error.code() == DXGI_ERROR_ACCESS_LOST => self.recreate().map(|()| None),
            Err(error) => Err(failed("AcquireNextFrame")(error)),
        }
    }
}

/// `None` for an update that is not a picture.
fn picture(
    held: &dxgi::Held<'_>,
    staging: &Staging,
    frequency: i64,
) -> Result<Option<ScreenFrame>, String> {
    if held.info.LastPresentTime == 0 {
        return Ok(None);
    }
    Ok(Some(ScreenFrame {
        at: qpc_to_duration(held.info.LastPresentTime, frequency),
        accumulated: held.info.AccumulatedFrames,
        width: staging.width as usize,
        height: staging.height as usize,
        bgra: staging.read(&held.picture()?)?,
    }))
}

impl Watch for DxgiWatch {
    fn next(&mut self, timeout: Duration) -> Result<Option<ScreenFrame>, String> {
        let start = Instant::now();
        loop {
            let left = timeout.saturating_sub(start.elapsed());
            let wait_ms = u32::try_from(left.as_millis()).unwrap_or(u32::MAX);
            if let Some(frame) = self.take(wait_ms)? {
                return Ok(Some(frame));
            }
            if start.elapsed() >= timeout {
                return Ok(None);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hosted runner has no desktop to duplicate.
    fn hardware_or_skip() -> bool {
        std::env::var_os("IMPERIFEX_NO_HARDWARE").is_none()
    }

    const CORNER: Rect = Rect {
        x: 0,
        y: 0,
        w: 64,
        h: 32,
    };

    #[test]
    fn a_region_of_the_primary_monitor_can_be_watched() {
        if !hardware_or_skip() {
            return;
        }
        let mut watch = DxgiScreen.watch(CORNER).unwrap();
        // The screen may be still: then there is no picture, and that is fine.
        if let Some(frame) = watch.next(Duration::from_millis(500)).unwrap() {
            assert_eq!((frame.width, frame.height), (64, 32));
            assert_eq!(frame.bgra.len(), 64 * 32 * 4);
            assert!(frame.at > Duration::ZERO);
        }
    }

    #[test]
    fn a_region_off_every_monitor_is_refused_with_the_region() {
        if !hardware_or_skip() {
            return;
        }
        let nowhere = Rect {
            x: -100_000,
            ..CORNER
        };
        let error = DxgiScreen.watch(nowhere).map(drop).unwrap_err();
        assert!(error.contains("-100000"), "{error}");
    }

    /// What a new duplication hands over before anything is filtered out, on
    /// each monitor: whether a still screen gives a first picture to compare
    /// against.
    /// `cargo test --release -p imperifex-bench-platform measure_the_first -- --ignored --nocapture`
    #[test]
    #[ignore = "a measurement, run by hand on a real screen"]
    fn measure_the_first_updates_of_a_new_duplication() {
        use imperifex_domain::ports::Clock;

        let clock = crate::QpcClock::new();
        for (monitor, x) in [("primary", 0), ("secondary", 1920)] {
            let region = Rect {
                x,
                y: 0,
                w: 1920,
                h: 1080,
            };
            for round in 1..=3 {
                let asked = clock.monotonic();
                let watch = DxgiWatch::new(region).unwrap();
                let made = clock.monotonic();
                let duplication = watch.duplication.as_ref().unwrap();
                let start = Instant::now();
                let mut seen = 0;
                while start.elapsed() < Duration::from_secs(2) {
                    let Ok(held) = dxgi::acquire(duplication, 200) else {
                        continue;
                    };
                    seen += 1;
                    if seen > 4 {
                        continue;
                    }
                    let presented = held.info.LastPresentTime;
                    let at = qpc_to_duration(presented, watch.frequency);
                    let pixels = watch.staging.read(&held.picture().unwrap()).unwrap();
                    let lit = pixels.chunks(4).filter(|p| p[..3] != [0, 0, 0]).count();
                    println!(
                        "{monitor} {round} · update {seen} after {:?}: {}, joined {}, {lit} pixels not black",
                        start.elapsed(),
                        if presented == 0 {
                            "LastPresentTime 0".to_string()
                        } else {
                            format!(
                                "presented {:.1} ms from the duplication",
                                (at.as_secs_f64() - made.as_secs_f64()) * 1000.0
                            )
                        },
                        held.info.AccumulatedFrames
                    );
                }
                println!(
                    "{monitor} {round} · {seen} updates in 2 s; making the duplication took {:?}",
                    made - asked
                );
            }
        }
    }

    /// `cargo test --release -p imperifex-bench-platform measure_watching -- --ignored --nocapture`
    #[test]
    #[ignore = "a measurement, run by hand on a real screen"]
    fn measure_watching_a_window_sized_region() {
        use imperifex_domain::ports::Clock;

        let region = Rect {
            x: 100,
            y: 140,
            w: 1280,
            h: 680,
        };
        let clock = crate::QpcClock::new();
        let mut watch = DxgiScreen.watch(region).unwrap();
        let (mut frames, mut joined, mut late) = (0u32, 0u32, Vec::new());
        let start = Instant::now();
        while start.elapsed() < Duration::from_secs(5) {
            if let Some(frame) = watch.next(Duration::from_millis(100)).unwrap() {
                frames += 1;
                joined += u32::from(frame.accumulated > 1);
                late.push(clock.monotonic().saturating_sub(frame.at));
            }
        }
        late.sort();
        let at = |share: usize| late.get(late.len() * share / 100).copied();
        println!(
            "{frames} pictures in 5 s ({joined} joined); from presented to read: p50 {:?}, p95 {:?}",
            at(50),
            at(95)
        );
    }
}
