//! Windows.UI.Composition visual tree for the island (design.md D1, D4, D5).
//! Retained-mode: springs run on the compositor; the app has no frame loop.
use crate::{island::Visual, log, media::Snapshot, theme};
use windows::{
    Foundation::{IReference, PropertyValue, TimeSpan},
    Graphics::{
        DirectX::{DirectXAlphaMode, DirectXPixelFormat},
        SizeInt32,
    },
    System::DispatcherQueueController,
    UI::Color,
    UI::Composition::{
        AnimationIterationBehavior, CompositionDrawingSurface, CompositionGraphicsDevice, CompositionVisualSurface, ContainerVisual, DropShadow, ShapeVisual,
        CompositionRoundedRectangleGeometry, CompositionSpriteShape, Compositor, Desktop::DesktopWindowTarget, SpriteVisual, Visual as CVisual,
    },
    Win32::{
        Foundation::{HANDLE, HMODULE, HWND, POINT},
        Globalization::{GetDateFormatEx, GetTimeFormatEx, DATE_LONGDATE, TIME_NOSECONDS},
        System::LibraryLoader::{GetProcAddress, LoadLibraryW},
        Graphics::{
            Direct2D::{
                Common::{D2D1_ALPHA_MODE_PREMULTIPLIED, D2D1_COLOR_F, D2D1_PIXEL_FORMAT, D2D_RECT_F, D2D_SIZE_U},
                D2D1CreateDevice, ID2D1Device, ID2D1DeviceContext, D2D1_BITMAP_OPTIONS_NONE, D2D1_BITMAP_PROPERTIES1, D2D1_DRAW_TEXT_OPTIONS_NONE, D2D1_ELLIPSE,
                D2D1_ROUNDED_RECT,
            },
            Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP, D3D_FEATURE_LEVEL},
            Direct3D11::{ID3D11Device, ID3D11Device4, D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION},
            DirectWrite::{
                DWriteCreateFactory, IDWriteFactory, IDWriteTextFormat, DWRITE_FACTORY_TYPE_SHARED, DWRITE_FONT_STRETCH_NORMAL,
                DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_WEIGHT, DWRITE_FONT_WEIGHT_REGULAR, DWRITE_FONT_WEIGHT_SEMI_BOLD,
                DWRITE_MEASURING_MODE_NATURAL, DWRITE_PARAGRAPH_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_CENTER, DWRITE_TEXT_ALIGNMENT_TRAILING,
                DWRITE_TRIMMING, DWRITE_TRIMMING_GRANULARITY_CHARACTER, DWRITE_WORD_WRAPPING_NO_WRAP,
            },
            Dxgi::{Common::DXGI_FORMAT_B8G8R8A8_UNORM, IDXGIDevice},
        },
        System::WinRT::{
            Composition::{ICompositionDrawingSurfaceInterop, ICompositorDesktopInterop, ICompositorInterop},
            CreateDispatcherQueueController, DispatcherQueueOptions, DQTAT_COM_STA, DQTYPE_THREAD_CURRENT,
        },
    },
    core::{h, s, w, Interface, Result, HRESULT, HSTRING, PCWSTR},
};
use std::time::Duration;
use windows_numerics::{Matrix3x2, Vector2, Vector3};

/// Target frame of the shape, in visual-window pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub radius: f32,
}

/// One content layer; two of them swap on every content change so the old one can fade out
/// while the new one (already drawn) fades in.
struct Content {
    visual: SpriteVisual,
    surface: CompositionDrawingSurface,
}

/// Lazily created text stack (design.md D5): Idle-only use never loads D3D/D2D.
struct Gpu {
    _d3d: ID3D11Device,
    graphics: CompositionGraphicsDevice,
    dwrite: IDWriteFactory,
    layers: [Content; 2],
    front: usize,
}

pub struct Renderer {
    _queue: DispatcherQueueController,
    _target: DesktopWindowTarget,
    compositor: Compositor,
    root: ContainerVisual,
    shadow_visual: SpriteVisual,
    shadow_source: CompositionVisualSurface,
    shape_visual: ShapeVisual,
    shape: CompositionSpriteShape,
    geometry: CompositionRoundedRectangleGeometry,
    gpu: Option<Gpu>,
    frame: Option<Frame>,
    content: Option<ContentKey>,
    /// Equalizer bars of the media Compact and the media progress fill (media-now-playing D4):
    /// composition visuals animated by the compositor, so playing music costs the app nothing.
    bars: ContainerVisual,
    progress: SpriteVisual,
    dpi: u32,
    media: Option<Snapshot>,
    /// Signalled by D3D when the device is removed; window waits on it (no polling).
    pub device_removed: Option<HANDLE>,
}

type ContentKey = (Visual, bool, u32, Option<(String, String, bool, usize, [bool; 3], bool)>);

fn color(rgb: u32) -> Color {
    Color { A: 255, R: (rgb >> 16) as u8, G: (rgb >> 8) as u8, B: rgb as u8 }
}

fn color_f(rgb: u32) -> D2D1_COLOR_F {
    let c = color(rgb);
    D2D1_COLOR_F { r: c.R as f32 / 255.0, g: c.G as f32 / 255.0, b: c.B as f32 / 255.0, a: 1.0 }
}

fn ms(v: i64) -> TimeSpan {
    TimeSpan { Duration: v * 10_000 }
}

fn v2(x: f32, y: f32) -> Vector2 {
    Vector2 { X: x, Y: y }
}

impl Renderer {
    pub fn new(hwnd: HWND) -> Result<Self> {
        let options = DispatcherQueueOptions {
            dwSize: size_of::<DispatcherQueueOptions>() as u32,
            threadType: DQTYPE_THREAD_CURRENT,
            apartmentType: DQTAT_COM_STA,
        };
        let queue = unsafe { CreateDispatcherQueueController(options)? };
        let compositor = Compositor::new()?;
        let target = unsafe { compositor.cast::<ICompositorDesktopInterop>()?.CreateDesktopWindowTarget(hwnd, true)? };

        let geometry = compositor.CreateRoundedRectangleGeometry()?;
        let shape = compositor.CreateSpriteShapeWithGeometry(&geometry)?;
        shape.SetFillBrush(&compositor.CreateColorBrushWithColor(color(theme::FILL))?)?;
        shape.SetStrokeBrush(&compositor.CreateColorBrushWithColor(color(theme::STROKE))?)?;
        let shape_visual = compositor.CreateShapeVisual()?;
        shape_visual.Shapes()?.Append(&shape)?;

        // Shadow masked by a live VisualSurface of the shape, so it follows the springs with no redraw.
        // (LayerVisual.Shadow paints the whole DesktopWindowTarget black — tried and discarded.)
        let root = compositor.CreateContainerVisual()?;
        let shadow_visual = compositor.CreateSpriteVisual()?;
        let shadow_source = compositor.CreateVisualSurface()?;
        let shadowed = (|| -> Result<()> {
            shadow_source.SetSourceVisual(&shape_visual)?;
            let shadow = compositor.CreateDropShadow()?;
            shadow.SetColor(color(0))?;
            shadow.SetOpacity(theme::SHADOW_OPACITY)?;
            shadow.SetMask(&compositor.CreateSurfaceBrushWithSurface(&shadow_source)?)?;
            shadow_visual.SetShadow(&shadow)
        })();
        if let Err(e) = shadowed {
            log::warn(&format!("shadow unavailable: {e}"));
        }
        root.Children()?.InsertAtTop(&shadow_visual)?;
        root.Children()?.InsertAtTop(&shape_visual)?;
        let bars = compositor.CreateContainerVisual()?;
        let bar_brush = compositor.CreateColorBrushWithColor(color(theme::BAR_COLOR))?;
        for _ in 0..theme::BAR_COUNT {
            let bar = compositor.CreateSpriteVisual()?;
            bar.SetBrush(&bar_brush)?;
            bars.Children()?.InsertAtTop(&bar)?;
        }
        bars.SetOpacity(0.0)?;
        let progress = compositor.CreateSpriteVisual()?;
        progress.SetBrush(&compositor.CreateColorBrushWithColor(color(theme::TEXT_PRIMARY))?)?;
        progress.SetOpacity(0.0)?;
        target.SetRoot(&root)?;
        Ok(Self {
            _queue: queue,
            _target: target,
            compositor,
            root,
            shadow_visual,
            shadow_source,
            shape_visual,
            shape,
            geometry,
            gpu: None,
            frame: None,
            content: None,
            bars,
            progress,
            dpi: 96,
            media: None,
            device_removed: None,
        })
    }

    /// Visual-window size and DPI scale; resets the shadow/stroke metrics that depend on scale.
    pub fn set_window(&mut self, w: f32, h: f32, scale: f32) -> Result<()> {
        // Every child is sized from the root so they cover the whole visual window.
        let size = v2(w, h);
        self.root.SetSize(size)?;
        self.shadow_visual.SetSize(size)?;
        self.shadow_source.SetSourceSize(size)?;
        self.shape_visual.SetSize(size)?;
        if let Ok(shadow) = self.shadow_visual.Shadow().and_then(|s| s.cast::<DropShadow>()) {
            shadow.SetBlurRadius(theme::SHADOW_BLUR_DIP * scale)?;
            shadow.SetOffset(Vector3 { X: 0.0, Y: theme::SHADOW_OFFSET_Y_DIP * scale, Z: 0.0 })?;
        }
        self.shape.SetStrokeThickness((theme::STROKE_DIP * scale).round().max(1.0))?;
        Ok(())
    }

    /// Move the shape to `frame` (spring when `animate`), swap content if what it shows changed,
    /// and keep the live media visuals (bars, progress) in step with `media`.
    pub fn apply(&mut self, frame: Frame, visual: Visual, activity: bool, media: Option<&Snapshot>, dpi: u32, animate: bool) -> Result<()> {
        if self.frame != Some(frame) {
            let animate = animate && self.frame.is_some();
            self.frame = Some(frame);
            self.move_shape(frame, animate)?;
        }
        self.dpi = dpi;
        let key = (visual, activity, dpi, media.map(Snapshot::content_key));
        let swapped = self.content.as_ref() != Some(&key);
        if swapped {
            self.content = Some(key);
            self.swap_content(frame, visual, activity, media, dpi as f32 / 96.0, animate)?;
        }
        if swapped || self.media.as_ref() != media {
            self.media = media.cloned();
            self.update_live(frame, visual, dpi as f32 / 96.0, animate)?;
        }
        Ok(())
    }

    /// Redraw the visible content in place (no fade): the media times, once a second, while Expanded.
    pub fn redraw_front(&mut self, visual: Visual, activity: bool) -> Result<()> {
        let (Some(frame), Some(gpu)) = (self.frame, self.gpu.as_ref()) else { return Ok(()) };
        let size = SizeInt32 { Width: frame.w.ceil() as i32, Height: frame.h.ceil() as i32 };
        draw(gpu, &gpu.layers[gpu.front].surface, size, visual, activity, self.media.as_ref(), self.dpi as f32 / 96.0)
    }

    /// Bars (media Compact) and progress fill (media Expanded with a known duration).
    fn update_live(&self, f: Frame, visual: Visual, scale: f32, animate: bool) -> Result<()> {
        let m = self.media.as_ref();
        let show_bars = visual == Visual::Compact && m.is_some();
        let show_progress = visual == Visual::Expanded && m.is_some_and(|m| !m.duration.is_zero());
        // Live visuals appear and disappear in step with the drawn content.
        let show = |v: &CVisual, on: bool| -> Result<()> {
            if on {
                self.fade(v, 1.0, theme::CONTENT_FADE_IN_DELAY_MS, theme::CONTENT_FADE_IN_MS, animate)
            } else {
                self.fade(v, 0.0, 0, theme::CONTENT_FADE_OUT_MS, animate)
            }
        };
        let px = |dip: f32| dip * scale;

        // Bars: right slot of the Compact, vertically centered, each pulsing at its own pace.
        let (bw, gap, bh) = (px(theme::BAR_W_DIP).round().max(2.0), px(theme::BAR_GAP_DIP).round(), px(theme::BAR_H_DIP).round());
        let total = theme::BAR_COUNT as f32 * bw + (theme::BAR_COUNT - 1) as f32 * gap;
        self.bars.SetOffset(Vector3 { X: f.x + f.w - px(theme::CONTENT_INSET_DIP) - total, Y: f.y + (f.h - bh) / 2.0, Z: 0.0 })?;
        let playing = m.is_some_and(|m| m.playing);
        for (i, bar) in self.bars.Children()?.into_iter().enumerate() {
            bar.SetSize(v2(bw, bh))?;
            bar.SetOffset(Vector3 { X: i as f32 * (bw + gap), Y: 0.0, Z: 0.0 })?;
            bar.SetCenterPoint(Vector3 { X: 0.0, Y: bh / 2.0, Z: 0.0 })?;
            bar.StopAnimation(h!("Scale.Y"))?;
            if playing && show_bars && animate {
                const PERIODS_MS: [i64; 4] = [520, 680, 450, 610];
                let a = self.compositor.CreateScalarKeyFrameAnimation()?;
                a.InsertKeyFrame(0.0, 0.35)?;
                a.InsertKeyFrame(0.5, 1.0)?;
                a.InsertKeyFrame(1.0, 0.35)?;
                a.SetDuration(ms(PERIODS_MS[i % PERIODS_MS.len()]))?;
                a.SetIterationBehavior(AnimationIterationBehavior::Forever)?;
                bar.StartAnimation(h!("Scale.Y"), &a)?;
            } else {
                // Paused: still, low and even. Animations off while playing: still, mid height.
                let level = if playing { 0.7 } else { theme::BAR_PAUSED_SCALE };
                bar.SetScale(Vector3 { X: 1.0, Y: level, Z: 1.0 })?;
            }
        }
        show(&self.bars.cast()?, show_bars)?;

        // Progress fill: Scale.X runs linearly from the current fraction to 1 over the remaining time.
        let side = px(theme::PROGRESS_SIDE_DIP);
        let track_w = (f.w - 2.0 * px(theme::EXPANDED_INSET_DIP) - 2.0 * side).max(1.0);
        self.progress.SetOffset(Vector3 { X: f.x + px(theme::EXPANDED_INSET_DIP) + side, Y: f.y + px(theme::PROGRESS_Y_DIP), Z: 0.0 })?;
        self.progress.SetSize(v2(track_w, px(theme::PROGRESS_H_DIP)))?;
        self.progress.StopAnimation(h!("Scale.X"))?;
        if let Some(m) = m.filter(|_| show_progress) {
            let pos = m.position_now();
            let frac = (pos.as_secs_f32() / m.duration.as_secs_f32()).clamp(0.0, 1.0);
            self.progress.SetScale(Vector3 { X: frac, Y: 1.0, Z: 1.0 })?;
            let remaining = m.duration.saturating_sub(pos);
            if m.playing && !remaining.is_zero() {
                let a = self.compositor.CreateScalarKeyFrameAnimation()?;
                let linear = self.compositor.CreateLinearEasingFunction()?;
                a.InsertKeyFrame(0.0, frac)?;
                a.InsertKeyFrameWithEasingFunction(1.0, 1.0, &linear)?;
                a.SetDuration(TimeSpan { Duration: (remaining.as_nanos() / 100) as i64 })?;
                self.progress.StartAnimation(h!("Scale.X"), &a)?;
            }
        }
        show(&self.progress.cast()?, show_progress)
    }

    fn move_shape(&self, f: Frame, animate: bool) -> Result<()> {
        let g = &self.geometry;
        if !animate {
            for name in ["Offset", "Size", "CornerRadius"] {
                g.StopAnimation(&HSTRING::from(name))?;
            }
            g.SetOffset(v2(f.x, f.y))?;
            g.SetSize(v2(f.w, f.h))?;
            return g.SetCornerRadius(v2(f.radius, f.radius));
        }
        // One scalar spring per component: windows-rs can box f32 (not Vector2) into the IReference FinalValue needs.
        // Same spring for all, so size, radius and position arrive together.
        let channels = [("Offset.X", f.x), ("Offset.Y", f.y), ("Size.X", f.w), ("Size.Y", f.h), ("CornerRadius.X", f.radius), ("CornerRadius.Y", f.radius)];
        for (name, value) in channels {
            let spring = self.compositor.CreateSpringScalarAnimation()?;
            spring.SetDampingRatio(theme::SPRING_DAMPING)?;
            spring.SetPeriod(ms(theme::SPRING_PERIOD_MS))?;
            spring.SetFinalValue(&PropertyValue::CreateSingle(value)?.cast::<IReference<f32>>()?)?;
            // No InitialValue: NaturalMotion starts from the current animated value, so interruptions don't jump.
            g.StartAnimation(&HSTRING::from(name), &spring)?;
        }
        Ok(())
    }

    fn fade(&self, visual: &CVisual, to: f32, delay_ms: i64, dur_ms: i64, animate: bool) -> Result<()> {
        if !animate {
            visual.StopAnimation(h!("Opacity"))?;
            return visual.SetOpacity(to);
        }
        let a = self.compositor.CreateScalarKeyFrameAnimation()?;
        a.InsertKeyFrame(1.0, to)?;
        a.SetDuration(ms(dur_ms))?;
        a.SetDelayTime(ms(delay_ms))?;
        visual.StartAnimation(h!("Opacity"), &a)
    }

    fn swap_content(&mut self, frame: Frame, visual: Visual, activity: bool, media: Option<&Snapshot>, scale: f32, animate: bool) -> Result<()> {
        let has_content = matches!(visual, Visual::Compact | Visual::Expanded);
        if self.gpu.is_none() {
            if !has_content {
                return Ok(()); // Hidden/Idle have no content: keep D3D unloaded.
            }
            self.gpu = Some(self.create_gpu()?);
        }
        let Some(gpu) = self.gpu.as_ref() else { return Ok(()) };
        let old = &gpu.layers[gpu.front];
        let new = &gpu.layers[1 - gpu.front];
        self.fade(&old.visual.cast()?, 0.0, 0, theme::CONTENT_FADE_OUT_MS, animate)?;
        new.visual.SetOffset(Vector3 { X: frame.x, Y: frame.y, Z: 0.0 })?;
        new.visual.SetSize(v2(frame.w, frame.h))?;
        new.visual.SetOpacity(0.0)?;
        if has_content {
            let size = SizeInt32 { Width: frame.w.ceil() as i32, Height: frame.h.ceil() as i32 };
            draw(gpu, &new.surface, size, visual, activity, media, scale)?;
            self.fade(&new.visual.cast()?, 1.0, theme::CONTENT_FADE_IN_DELAY_MS, theme::CONTENT_FADE_IN_MS, animate)?;
        }
        if let Some(gpu) = self.gpu.as_mut() {
            gpu.front = 1 - gpu.front;
        }
        Ok(())
    }

    fn create_gpu(&mut self) -> Result<Gpu> {
        let (d3d, d2d) = create_device()?;
        let graphics = unsafe { self.compositor.cast::<ICompositorInterop>()?.CreateGraphicsDevice(&d2d)? };
        let dwrite: IDWriteFactory = unsafe { DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED)? };
        let layer = |_| -> Result<Content> {
            let surface = graphics.CreateDrawingSurface2(SizeInt32 { Width: 1, Height: 1 }, DirectXPixelFormat::B8G8R8A8UIntNormalized, DirectXAlphaMode::Premultiplied)?;
            let brush = self.compositor.CreateSurfaceBrushWithSurface(&surface)?;
            let visual = self.compositor.CreateSpriteVisual()?;
            visual.SetBrush(&brush)?;
            visual.SetOpacity(0.0)?;
            self.root.Children()?.InsertAtTop(&visual)?;
            Ok(Content { visual, surface })
        };
        let layers = [layer(0)?, layer(1)?];
        // Live media visuals sit above the drawn content (the progress fill covers the drawn track).
        self.root.Children()?.InsertAtTop(&self.bars)?;
        self.root.Children()?.InsertAtTop(&self.progress)?;
        self.device_removed = watch_device(&d3d);
        log::info("gpu text device created");
        Ok(Gpu { _d3d: d3d, graphics, dwrite, layers, front: 0 })
    }

    /// Device removed (driver reset, GPU switch): new D3D/D2D on the same CompositionGraphicsDevice, then redraw.
    pub fn recover_device(&mut self) -> Result<()> {
        let Some(gpu) = self.gpu.as_mut() else { return Ok(()) };
        let (d3d, d2d) = create_device()?;
        unsafe { gpu.graphics.cast::<windows::Win32::System::WinRT::Composition::ICompositionGraphicsDeviceInterop>()?.SetRenderingDevice(&d2d)? };
        self.device_removed = watch_device(&d3d);
        gpu._d3d = d3d;
        self.content = None; // force redraw on next apply
        log::warn("gpu device recovered");
        Ok(())
    }
}

/// `D3D11CreateDevice` resolved at runtime: a static import loads dxgi.dll at startup, which reserves
/// ~54 MB of private commit as soon as the thread creates a window — even if no device is ever made.
/// ponytail: once a device exists the reservation stays for the process lifetime (unloading doesn't return it);
/// it is untouched commit, not RAM — the private working set stays small.
type D3D11CreateDeviceFn = unsafe extern "system" fn(
    adapter: *mut core::ffi::c_void,
    driver: D3D_DRIVER_TYPE,
    software: HMODULE,
    flags: D3D11_CREATE_DEVICE_FLAG,
    levels: *const D3D_FEATURE_LEVEL,
    level_count: u32,
    sdk: u32,
    device: *mut Option<ID3D11Device>,
    level: *mut D3D_FEATURE_LEVEL,
    context: *mut *mut core::ffi::c_void,
) -> HRESULT;

fn create_device() -> Result<(ID3D11Device, ID2D1Device)> {
    let d3d11_create_device: D3D11CreateDeviceFn = unsafe {
        let module = LoadLibraryW(w!("d3d11.dll"))?;
        let proc = GetProcAddress(module, s!("D3D11CreateDevice")).ok_or_else(windows::core::Error::from_thread)?;
        std::mem::transmute::<unsafe extern "system" fn() -> isize, D3D11CreateDeviceFn>(proc)
    };
    let make = |driver: D3D_DRIVER_TYPE| -> Result<ID3D11Device> {
        let mut device = None;
        let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
        let null = std::ptr::null_mut();
        unsafe { d3d11_create_device(null, driver, HMODULE::default(), flags, std::ptr::null(), 0, D3D11_SDK_VERSION, &mut device, null.cast(), null.cast()).ok()? };
        device.ok_or_else(|| windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))
    };
    let d3d = make(D3D_DRIVER_TYPE_HARDWARE).or_else(|_| make(D3D_DRIVER_TYPE_WARP))?;
    let d2d = unsafe { D2D1CreateDevice(&d3d.cast::<IDXGIDevice>()?, None)? };
    Ok((d3d, d2d))
}

fn watch_device(d3d: &ID3D11Device) -> Option<HANDLE> {
    let device4 = d3d.cast::<ID3D11Device4>().ok()?;
    let event = unsafe { windows::Win32::System::Threading::CreateEventW(None, false, false, None) }.ok()?;
    unsafe { device4.RegisterDeviceRemovedEvent(event) }.ok()?;
    Some(event)
}

/// Current time (no seconds) and long date in the user's locale, e.g. "20:45" / "quinta-feira, 26 de novembro de 2026".
fn now_strings() -> (String, String) {
    let fmt = |f: &dyn Fn(&mut [u16]) -> i32| {
        let mut buf = [0u16; 128];
        let n = f(&mut buf).max(1) as usize;
        String::from_utf16_lossy(&buf[..n - 1])
    };
    let time = fmt(&|b| unsafe { GetTimeFormatEx(PCWSTR::null(), TIME_NOSECONDS, None, PCWSTR::null(), Some(b)) });
    let date = fmt(&|b| unsafe { GetDateFormatEx(PCWSTR::null(), DATE_LONGDATE, None, PCWSTR::null(), Some(b), PCWSTR::null()) });
    (time, date)
}

/// "m:ss" for the media times.
fn mmss(d: Duration) -> String {
    let s = d.as_secs();
    format!("{}:{:02}", s / 60, s % 60)
}

#[allow(clippy::too_many_arguments)]
fn draw(gpu: &Gpu, surface: &CompositionDrawingSurface, size: SizeInt32, visual: Visual, activity: bool, media: Option<&Snapshot>, scale: f32) -> Result<()> {
    surface.Resize(size)?;
    let interop = surface.cast::<ICompositionDrawingSurfaceInterop>()?;
    let mut offset = POINT::default();
    let dc: ID2D1DeviceContext = unsafe { interop.BeginDraw(None, &mut offset)? };
    let result = (|| -> Result<()> {
        let (ox, oy) = (offset.x as f32, offset.y as f32);
        let (w, h) = (size.Width as f32, size.Height as f32);
        let px = |dip: f32| dip * scale;
        let format_in = |family: &str, size_dip: f32, weight: DWRITE_FONT_WEIGHT| -> Result<IDWriteTextFormat> {
            let family: Vec<u16> = family.encode_utf16().chain([0]).collect();
            let f = unsafe {
                gpu.dwrite.CreateTextFormat(PCWSTR(family.as_ptr()), None, weight, DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, px(size_dip), windows::core::w!("pt-BR"))?
            };
            unsafe {
                f.SetParagraphAlignment(DWRITE_PARAGRAPH_ALIGNMENT_CENTER)?;
                f.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP)?;
            }
            Ok(f)
        };
        let format = |size_dip: f32, weight: DWRITE_FONT_WEIGHT| format_in(theme::FONT_FAMILY, size_dip, weight);
        // Text that doesn't fit ends in "…" instead of spilling over (spec: Texto longo).
        let ellipsis = |f: &IDWriteTextFormat| -> Result<()> {
            let sign = unsafe { gpu.dwrite.CreateEllipsisTrimmingSign(f)? };
            let trimming = DWRITE_TRIMMING { granularity: DWRITE_TRIMMING_GRANULARITY_CHARACTER, delimiter: 0, delimiterCount: 0 };
            unsafe { f.SetTrimming(&trimming, &sign) }
        };
        let text = |s: &str, f: &IDWriteTextFormat, rect: D2D_RECT_F, rgb: u32| -> Result<()> {
            let brush = unsafe { dc.CreateSolidColorBrush(&color_f(rgb), None)? };
            let s: Vec<u16> = s.encode_utf16().collect();
            unsafe { dc.DrawText(&s, f, &rect, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE, DWRITE_MEASURING_MODE_NATURAL) };
            Ok(())
        };
        unsafe {
            dc.Clear(Some(&D2D1_COLOR_F::default()));
        }
        // Album art (or a note glyph on gray) in a rounded square at (x, y), `side` px.
        let art = |m: &Snapshot, x: f32, y: f32, side: f32| -> Result<()> {
            let rr = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: x, top: y, right: x + side, bottom: y + side }, radiusX: px(theme::ART_RADIUS_DIP) * side / px(theme::ART_EXPANDED_DIP), radiusY: px(theme::ART_RADIUS_DIP) * side / px(theme::ART_EXPANDED_DIP) };
            if let Some(pixels) = &m.art {
                let n = theme::ART_PX;
                let props = D2D1_BITMAP_PROPERTIES1 {
                    pixelFormat: D2D1_PIXEL_FORMAT { format: DXGI_FORMAT_B8G8R8A8_UNORM, alphaMode: D2D1_ALPHA_MODE_PREMULTIPLIED },
                    dpiX: 96.0,
                    dpiY: 96.0,
                    bitmapOptions: D2D1_BITMAP_OPTIONS_NONE,
                    colorContext: std::mem::ManuallyDrop::new(None),
                };
                let bmp = unsafe { dc.CreateBitmap(D2D_SIZE_U { width: n, height: n }, Some(pixels.as_ptr().cast()), n * 4, &props)? };
                let brush = unsafe { dc.CreateBitmapBrush(&bmp, None, None)? };
                let k = side / n as f32;
                unsafe {
                    brush.SetTransform(&Matrix3x2 { M11: k, M12: 0.0, M21: 0.0, M22: k, M31: x, M32: y });
                    dc.FillRoundedRectangle(&rr, &brush);
                }
            } else {
                let bg = unsafe { dc.CreateSolidColorBrush(&color_f(theme::ART_PLACEHOLDER), None)? };
                unsafe { dc.FillRoundedRectangle(&rr, &bg) };
                let f = format_in(theme::ICON_FONT, side / scale * 0.45, DWRITE_FONT_WEIGHT_REGULAR)?;
                unsafe { f.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)? };
                text("\u{E8D6}", &f, rr.rect, theme::TEXT_SECONDARY)?;
            }
            Ok(())
        };
        match (visual, media) {
            (Visual::Compact, Some(m)) => {
                // Art in the left slot; the bars on the right are live visuals (update_live).
                let side = px(theme::ART_COMPACT_DIP);
                let margin = (h - side) / 2.0;
                art(m, ox + margin, oy + margin, side)?;
                return Ok(());
            }
            (Visual::Expanded, Some(m)) => {
                let inset = px(theme::EXPANDED_INSET_DIP);
                let side = px(theme::ART_EXPANDED_DIP);
                art(m, ox + inset, oy + inset, side)?;
                let text_left = ox + inset + side + px(14.0);
                let title = format(theme::FONT_TITLE_DIP, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
                let artist = format(theme::FONT_SUBTITLE_DIP, DWRITE_FONT_WEIGHT_REGULAR)?;
                ellipsis(&title)?;
                ellipsis(&artist)?;
                let row = |top: f32, height: f32| D2D_RECT_F { left: text_left, top: oy + px(top), right: ox + w - inset, bottom: oy + px(top + height) };
                text(&m.title, &title, row(26.0, 22.0), theme::TEXT_PRIMARY)?;
                text(&m.artist, &artist, row(48.0, 18.0), theme::TEXT_SECONDARY)?;
                if !m.duration.is_zero() {
                    // Track (the fill is a live visual on top) and the times at its ends.
                    let side_w = px(theme::PROGRESS_SIDE_DIP);
                    let (y, th) = (oy + px(theme::PROGRESS_Y_DIP), px(theme::PROGRESS_H_DIP));
                    let track = unsafe { dc.CreateSolidColorBrush(&color_f(theme::PROGRESS_TRACK), None)? };
                    let tr = D2D1_ROUNDED_RECT { rect: D2D_RECT_F { left: ox + inset + side_w, top: y, right: ox + w - inset - side_w, bottom: y + th }, radiusX: th / 2.0, radiusY: th / 2.0 };
                    unsafe { dc.FillRoundedRectangle(&tr, &track) };
                    let pos = m.position_now();
                    let time = format(theme::FONT_TIME_DIP, DWRITE_FONT_WEIGHT_REGULAR)?;
                    let band = |l: f32, r: f32| D2D_RECT_F { left: l, top: y + th / 2.0 - px(9.0), right: r, bottom: y + th / 2.0 + px(9.0) };
                    text(&mmss(pos), &time, band(ox + inset, ox + inset + side_w - px(6.0)), theme::TEXT_SECONDARY)?;
                    unsafe { time.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING)? };
                    text(&format!("-{}", mmss(m.duration.saturating_sub(pos))), &time, band(ox + w - inset - side_w + px(6.0), ox + w - inset), theme::TEXT_SECONDARY)?;
                }
                // Buttons: Segoe Fluent Icons glyphs, centered on the hit boxes from layout::media_buttons.
                let icons = format_in(theme::ICON_FONT, theme::FONT_BUTTON_DIP, DWRITE_FONT_WEIGHT_REGULAR)?;
                unsafe { icons.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_CENTER)? };
                let glyphs = ["\u{E892}", if m.playing { "\u{E769}" } else { "\u{E768}" }, "\u{E893}"];
                for (i, b) in crate::layout::media_buttons(w.round() as i32, scale).iter().enumerate() {
                    let rect = D2D_RECT_F { left: ox + b.left as f32, top: oy + b.top as f32, right: ox + b.right as f32, bottom: oy + b.bottom as f32 };
                    text(glyphs[i], &icons, rect, if m.can[i] { theme::TEXT_PRIMARY } else { theme::DISABLED })?;
                }
                return Ok(());
            }
            _ => {}
        }
        // ponytail: fixed placeholder content (debug Demo only); real activities bring their own payloads.
        match visual {
            Visual::Hidden | Visual::Idle => {}
            Visual::Expanded if !activity => {
                // ponytail: drawn once when expanding, not a live clock (content phase brings real widgets).
                let inset = px(theme::EXPANDED_INSET_DIP);
                let line = |top: f32, height: f32| D2D_RECT_F { left: ox + inset, top: oy + px(top), right: ox + w - inset, bottom: oy + px(top + height) };
                let clock = format(theme::FONT_CLOCK_DIP, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
                let date = format(theme::FONT_SUBTITLE_DIP, DWRITE_FONT_WEIGHT_REGULAR)?;
                let (time_s, date_s) = now_strings();
                text(&time_s, &clock, line(30.0, 56.0), theme::TEXT_PRIMARY)?;
                text(&date_s, &date, line(90.0, 20.0), theme::TEXT_SECONDARY)?;
            }
            Visual::Compact => {
                let inset = px(theme::CONTENT_INSET_DIP);
                let r = px(4.0);
                let accent = unsafe { dc.CreateSolidColorBrush(&color_f(theme::ACCENT), None)? };
                let dot = D2D1_ELLIPSE { point: v2(ox + inset + r, oy + h / 2.0), radiusX: r, radiusY: r };
                unsafe { dc.FillEllipse(&dot, &accent) };
                let f = format(theme::FONT_COMPACT_DIP, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
                unsafe { f.SetTextAlignment(DWRITE_TEXT_ALIGNMENT_TRAILING)? };
                text("0:08", &f, D2D_RECT_F { left: ox + inset, top: oy, right: ox + w - inset, bottom: oy + h }, theme::ACCENT)?;
            }
            Visual::Expanded => {
                let inset = px(theme::EXPANDED_INSET_DIP);
                let title = format(theme::FONT_TITLE_DIP, DWRITE_FONT_WEIGHT_SEMI_BOLD)?;
                let sub = format(theme::FONT_SUBTITLE_DIP, DWRITE_FONT_WEIGHT_REGULAR)?;
                let line = |top: f32, height: f32| D2D_RECT_F { left: ox + inset, top: oy + px(top), right: ox + w - inset, bottom: oy + px(top + height) };
                text("Atividade de exemplo", &title, line(26.0, 22.0), theme::TEXT_PRIMARY)?;
                text("Demonstração da Fase 2", &sub, line(48.0, 18.0), theme::TEXT_SECONDARY)?;
                // Placeholder "controls" row: a progress track.
                let track = unsafe { dc.CreateSolidColorBrush(&color_f(0x3A3A3C), None)? };
                let fill = unsafe { dc.CreateSolidColorBrush(&color_f(theme::TEXT_PRIMARY), None)? };
                let bar = |right: f32| D2D1_ROUNDED_RECT {
                    rect: D2D_RECT_F { left: ox + inset, top: oy + px(110.0), right, bottom: oy + px(116.0) },
                    radiusX: px(3.0),
                    radiusY: px(3.0),
                };
                unsafe {
                    dc.FillRoundedRectangle(&bar(ox + w - inset), &track);
                    dc.FillRoundedRectangle(&bar(ox + inset + (w - 2.0 * inset) * 0.6), &fill);
                }
            }
        }
        Ok(())
    })();
    unsafe { interop.EndDraw()? };
    result
}
