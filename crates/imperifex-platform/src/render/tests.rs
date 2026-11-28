//! A frame drawn on a texture and read back: the shaders, the atlas and the
//! blending, with no window. It needs a graphics card.

use std::rc::Rc;

use imperifex_domain::draw::{DrawList, Rect, TextRun, UiFont};
use imperifex_domain::theme::Rgb;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11Texture2D, D3D11_BIND_RENDER_TARGET, D3D11_CPU_ACCESS_READ,
    D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ, D3D11_TEXTURE2D_DESC, D3D11_USAGE,
    D3D11_USAGE_DEFAULT, D3D11_USAGE_STAGING,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_SAMPLE_DESC};

use super::painter::Painter;
use crate::text::DirectWriteText;

const SIZE: (u32, u32) = (64, 32);

fn texture(device: &ID3D11Device, usage: D3D11_USAGE, bind: u32, cpu: u32) -> ID3D11Texture2D {
    let description = D3D11_TEXTURE2D_DESC {
        Width: SIZE.0,
        Height: SIZE.1,
        MipLevels: 1,
        ArraySize: 1,
        Format: DXGI_FORMAT_B8G8R8A8_UNORM,
        SampleDesc: DXGI_SAMPLE_DESC {
            Count: 1,
            Quality: 0,
        },
        Usage: usage,
        BindFlags: bind,
        CPUAccessFlags: cpu,
        ..Default::default()
    };
    let mut texture = None;
    // SAFETY: the description and the output live through the call.
    unsafe { device.CreateTexture2D(&description, None, Some(&mut texture)) }.expect("a texture");
    texture.expect("a texture")
}

/// The colours of the pixels of `list` drawn, row by row.
fn pixels(painter: &mut Painter, list: &DrawList) -> Vec<Rgb> {
    let bind = D3D11_BIND_RENDER_TARGET.0 as u32;
    let drawn = texture(painter.device(), D3D11_USAGE_DEFAULT, bind, 0);
    let cpu = D3D11_CPU_ACCESS_READ.0 as u32;
    let copy = texture(painter.device(), D3D11_USAGE_STAGING, 0, cpu);
    let target = painter.target(&drawn).expect("a target");
    painter.paint(list, &target, SIZE).expect("a frame");
    // SAFETY: both are of one device, with the same size and format.
    unsafe { painter.context().CopyResource(&copy, &drawn) };
    let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
    // SAFETY: a staging texture mapped for reading; `mapped` lives through it.
    unsafe {
        painter
            .context()
            .Map(&copy, 0, D3D11_MAP_READ, 0, Some(&mut mapped))
    }
    .expect("the pixels");
    let mut found = Vec::new();
    for row in 0..SIZE.1 as usize {
        let first = mapped.pData.cast::<u8>();
        // SAFETY: the mapped texture has `SIZE.1` rows `RowPitch` bytes apart.
        let start = unsafe { first.add(row * mapped.RowPitch as usize) };
        // SAFETY: each with at least four bytes per pixel of its width.
        let bytes = unsafe { std::slice::from_raw_parts(start, SIZE.0 as usize * 4) };
        let pixels = bytes.as_chunks::<4>().0.iter();
        found.extend(pixels.map(|p| Rgb(p[2], p[1], p[0])));
    }
    // SAFETY: mapped above.
    unsafe { painter.context().Unmap(&copy, 0) };
    found
}

#[test]
fn rectangles_and_letters_reach_the_pixels_in_their_colours() {
    // A hosted runner has no graphics card: skipped there on purpose.
    if std::env::var_os("IMPERIFEX_NO_HARDWARE").is_some() {
        return;
    }
    let text = Rc::new(DirectWriteText::new("Consolas").expect("DirectWrite"));
    let mut painter = Painter::new(text).expect("a graphics card");
    let rect = |x, width| Rect {
        x,
        y: 0,
        width,
        height: 32,
    };
    let (red, blue) = (Rgb(200, 10, 20), Rgb(10, 20, 200));
    let list = DrawList {
        rects: vec![(rect(0, 64), red), (rect(32, 32), blue)],
        texts: vec![TextRun {
            x: 4,
            y: 2,
            text: "H".into(),
            color: Rgb::WHITE,
            font: UiFont::Text,
            size: 24,
        }],
    };
    let found = pixels(&mut painter, &list);
    let at = |x: usize, y: usize| found[y * SIZE.0 as usize + x];
    assert_eq!((at(1, 1), at(63, 31), at(31, 31)), (red, blue, red));
    let left = || (0..32).flat_map(|y| (4..32).map(move |x| (x, y)));
    let white = left().filter(|&(x, y)| at(x, y) == Rgb::WHITE).count();
    assert!(white > 20, "{white} white pixels");
    // Smoothed edges: between the letter's colour and what is under it.
    let between = left().filter(|&(x, y)| ![red, Rgb::WHITE].contains(&at(x, y)));
    assert!(between.count() > 0);
    // Nothing of the letter went past it.
    assert!((32..64).all(|x| (0..32).all(|y| at(x, y) == blue)));
    // The second time the glyph comes from the atlas, and it looks the same.
    assert_eq!(pixels(&mut painter, &list), found);
    painter.forget_glyphs();
    assert_eq!(pixels(&mut painter, &list), found);
}
