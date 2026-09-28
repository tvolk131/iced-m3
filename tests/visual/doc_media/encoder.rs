use super::super::reference::Image;
use std::io::Cursor;

// SOURCE replacement with NONE disposal lets each frame store only the changed
// bounding rectangle, while retaining full RGBA precision (no palette reduction).
fn changed_bounds(before: &Image, after: &Image) -> (u32, u32, u32, u32) {
    let (mut x0, mut y0, mut x1, mut y1) = (after.width, after.height, 0, 0);
    for (i, (a, b)) in before
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(after.pixels.as_chunks::<4>().0)
        .enumerate()
    {
        if a != b {
            let (x, y) = (i as u32 % after.width, i as u32 / after.width);
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    assert!(x0 <= x1 && y0 <= y1);
    (x0, y0, x1 - x0 + 1, y1 - y0 + 1)
}

pub(super) fn encode_sparse(frames: &[&Image], fps: u16) -> (Vec<u8>, usize) {
    encode_impl(frames, fps, true, true)
}

fn encode_impl(frames: &[&Image], fps: u16, optimized: bool, sparse: bool) -> (Vec<u8>, usize) {
    let mut runs: Vec<(usize, u16)> = Vec::new();
    for (i, frame) in frames.iter().enumerate() {
        if optimized && i > 0 && *frame == frames[i - 1] {
            runs.last_mut().unwrap().1 += 1;
        } else {
            runs.push((i, 1));
        }
    }
    let first = frames[0];
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, first.width, first.height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Balanced);
        encoder.set_animated(runs.len() as u32, 0).unwrap();
        encoder.set_blend_op(png::BlendOp::Source).unwrap();
        encoder.set_dispose_op(png::DisposeOp::None).unwrap();
        let mut writer = encoder.write_header().unwrap();
        for &(i, duration) in &runs {
            let (x, y, w, h) = if optimized && i > 0 {
                changed_bounds(frames[i - 1], frames[i])
            } else {
                (0, 0, first.width, first.height)
            };
            writer.set_frame_position(0, 0).unwrap();
            writer.set_frame_dimension(w, h).unwrap();
            writer.set_frame_position(x, y).unwrap();
            writer.set_frame_delay(duration, fps).unwrap();
            let mut patch = frames[i].crop(x, y, w, h);
            let over =
                sparse && i > 0 && patch.pixels.as_chunks::<4>().0.iter().all(|p| p[3] == 255);
            if over {
                let previous = frames[i - 1].crop(x, y, w, h);
                for (pixel, old) in patch
                    .pixels
                    .as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .zip(previous.pixels.as_chunks::<4>().0)
                {
                    if pixel == old {
                        *pixel = [0; 4];
                    }
                }
            }
            writer
                .set_blend_op(if over {
                    png::BlendOp::Over
                } else {
                    png::BlendOp::Source
                })
                .unwrap();
            writer.write_image_data(&patch.pixels).unwrap();
        }
        writer.finish().unwrap();
    }
    // Independently decode/composite all stored frames and expand their delays:
    // compression may change neither pixels nor the original frame timeline.
    let mut reader = png::Decoder::new(Cursor::new(&bytes)).read_info().unwrap();
    assert_eq!(
        reader.info().animation_control.unwrap().num_frames,
        runs.len() as u32
    );
    let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
    let mut canvas = vec![0; first.pixels.len()];
    let mut sample = 0;
    for _ in &runs {
        let info = reader.next_frame(&mut buffer).unwrap();
        let fc = reader.info().frame_control.unwrap();

        assert_eq!(fc.dispose_op, png::DisposeOp::None);
        assert_eq!(fc.delay_den, fps);
        for y in 0..info.height {
            let start = (((fc.y_offset + y) * first.width + fc.x_offset) * 4) as usize;
            let input = (y * info.width * 4) as usize;
            let len = (info.width * 4) as usize;
            for (destination, source) in canvas[start..start + len]
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(buffer[input..input + len].as_chunks::<4>().0)
            {
                if fc.blend_op == png::BlendOp::Source || source[3] == 255 {
                    *destination = *source;
                } else {
                    assert_eq!(
                        source[3], 0,
                        "sparse patches contain only opaque replacements or transparent unchanged pixels"
                    );
                }
            }
        }
        for _ in 0..fc.delay_num {
            assert_eq!(
                canvas, frames[sample].pixels,
                "APNG pixel mismatch at sample {sample}"
            );
            sample += 1;
        }
    }
    assert_eq!(sample, frames.len());
    (bytes, runs.len())
}

#[test]
fn sparse_encoding_preserves_holes_alpha_and_held_frame_timing() {
    let image = |pixels: &[[u8; 4]]| Image {
        width: pixels.len() as u32,
        height: 1,
        pixels: pixels.iter().flatten().copied().collect(),
    };
    let a = image(&[[255, 255, 255, 255]; 3]);
    // The middle pixel is unchanged inside a bounding rectangle spanning both ends.
    let b = image(&[[0, 0, 0, 255], [255, 255, 255, 255], [0, 0, 0, 255]]);
    // Partial alpha must use SOURCE replacement instead of sparse OVER blending.
    let c = image(&[[0, 0, 0, 128], [255, 255, 255, 255], [0, 0, 0, 0]]);
    let (_, count) = encode_sparse(&[&a, &b, &b, &c], 60);
    assert_eq!(count, 3);
    let (_, count) = encode_impl(&[&a, &b, &b, &c], 60, false, false);
    assert_eq!(count, 4);
}
