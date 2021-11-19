/*
 * Created on Sat Sep 11 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod logo;

use std::{iter, sync::Arc, time::Instant};

use futures::executor::block_on;
use logo::build_logo_path;
use storyboard_box_2d::{compositor::BoxCompositor, BoxStyle};
use storyboard_graphics::component::extent::ExtentSize2D;
use storyboard_graphics::component::layout::ComponentLayout;
use storyboard_graphics::wgpu::{Backends, BlendState, BufferUsages, Color, ColorTargetState, ColorWrites, CommandEncoderDescriptor, Features, LoadOp, Operations, PresentMode, RenderPassColorAttachment, RenderPassDepthStencilAttachment, RenderPassDescriptor, TextureFormat};
use storyboard_graphics::{
    backend::{BackendOptions, StoryboardBackend},
    buffer::stream::StreamBufferAllocator,
    component::{
        color::ShapeColor,
        extent::{Extent2D, ExtentStandard, ExtentUnit},
        texture::{ComponentTexture, TextureLayout},
        DrawSpace,
    },
    context::DrawContext,
    math::{Point2D, Rect, Size2D},
    pipeline::PipelineTargetDescriptor,
    renderer::StoryboardRenderer,
    texture::depth::DepthStencilTexture,
    texture::resources::TextureResources,
    unit::PixelUnit,
    wgpu::{CompareFunction, DepthBiasState, DepthStencilState, StencilState},
};
use storyboard_path::lyon::{
    lyon_tessellation::{BuffersBuilder, StrokeOptions, StrokeTessellator, VertexBuffers},
    path::Path,
};
use storyboard_path::{
    compositor::{PathCompositor, PathFiller},
    PathVertex, ScalablePath,
};
use storyboard_primitive::{compositor::PrimitiveCompositor, PrimitiveStyle};
use storyboard_text::allsorts::font::MatchingPresentation;
use storyboard_text::allsorts::glyph_position::TextDirection;
use storyboard_text::font::DrawFont;
use storyboard_text::layout::TextLayout;
use storyboard_text::TextStyle;
use storyboard_text::{
    compositor::GlyphCompositor, font_kit::source::SystemSource, store::GlyphStore,
};
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub fn main() {
    // simple_logger::SimpleLogger::new().init().unwrap();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(800, 800))
        .build(&event_loop)
        .unwrap();

    let (mut backend, surface) = block_on(StoryboardBackend::init_surface(
        &window,
        BackendOptions {
            backends: Backends::all(),
            ..Default::default()
        },
    ))
    .unwrap();
    let size: (u32, u32) = window.inner_size().into();
    backend.update_present_mode(PresentMode::Mailbox);
    backend.update_view(size.into());

    let textures = TextureResources::init(backend.device().clone(), backend.queue().clone());

    let mut stream_allocator = StreamBufferAllocator::new(
        BufferUsages::VERTEX | BufferUsages::INDEX | BufferUsages::UNIFORM,
    );

    let path_style = {
        let mut builder = Path::builder().with_svg();
        build_logo_path(&mut builder);
        let path = builder.build();

        let mut geometry: VertexBuffers<PathVertex, u16> = VertexBuffers::new();
        let mut tessellator = StrokeTessellator::new();
        {
            tessellator
                .tessellate_path(
                    &path,
                    &StrokeOptions::default().with_line_width(2.0),
                    &mut BuffersBuilder::new(
                        &mut geometry,
                        PathFiller {
                            color: (1.0, 1.0, 1.0, 1.0).into(),
                        },
                    ),
                )
                .unwrap();
        }

        ScalablePath {
            path: geometry,
            rect: Rect {
                origin: (0.0, 0.0).into(),
                size: (120.0, 120.0).into(),
            },
        }
    };

    let mut glyph_brush = GlyphStore::init(
        &textures,
        Arc::new(DrawFont::new(
            SystemSource::new()
                .select_by_postscript_name("NotoSansCJKkr-Regular")
                .unwrap()
                .load()
                .unwrap(),
        )),
        96.0,
    );

    let mut rect = BoxStyle::default();

    rect.border_radius = ExtentUnit::Percent(0.5);
    rect.border_thickness = 5.0;

    rect.texture = Some(ComponentTexture {
        texture: glyph_brush.texture().clone(),
        layout: TextureLayout::FitY,
    });

    rect.fill_color = ShapeColor::Gradient([
        (1.0, 0.0, 1.0, 1.0).into(),
        (0.0, 1.0, 1.0, 1.0).into(),
        (1.0, 1.0, 0.0, 1.0).into(),
        (1.0, 1.0, 1.0, 1.0).into(),
    ]);

    rect.border_color = ShapeColor::Single((1.0, 0.0, 1.0, 1.0).into());

    let mut rect_node = ComponentLayout::new();
    rect_node.set_position(Extent2D {
        standard: ExtentStandard::Parent,
        x: ExtentUnit::Percent(0.5),
        y: ExtentUnit::Percent(0.5),
    });

    rect_node.set_anchor(Extent2D {
        standard: ExtentStandard::Current,
        x: ExtentUnit::Percent(0.5),
        y: ExtentUnit::Percent(0.5),
    });

    rect_node.set_size(ExtentSize2D {
        standard: ExtentStandard::Parent,
        width: ExtentUnit::Percent(0.5),
        height: ExtentUnit::Percent(0.5),
    });

    rect_node.transform_mut().origin = Extent2D {
        standard: ExtentStandard::Current,
        x: ExtentUnit::Percent(0.5),
        y: ExtentUnit::Percent(0.5),
    };

    let mut triangle_trans = PrimitiveStyle::default();
    triangle_trans.opacity = 0.5;

    let pipeline_desc = PipelineTargetDescriptor {
        fragments_targets: &[ColorTargetState {
            format: backend.surface_config().format,
            blend: Some(BlendState::ALPHA_BLENDING),
            write_mask: ColorWrites::ALL,
        }],

        depth_stencil: Some(DepthStencilState {
            format: TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: CompareFunction::LessEqual,
            stencil: StencilState {
                read_mask: !0,
                write_mask: !0,
                ..StencilState::default()
            },
            bias: DepthBiasState::default(),
        }),

        ..PipelineTargetDescriptor::default()
    };

    let primitive_compositor = PrimitiveCompositor::init(
        backend.device(),
        textures.texture2d_bind_group_layout(),
        pipeline_desc.clone(),
    );
    let box_compositor = BoxCompositor::init(
        backend.device(),
        textures.texture2d_bind_group_layout(),
        pipeline_desc.clone(),
    );
    let path_compositor = PathCompositor::init(backend.device(), pipeline_desc.clone());

    let mut depth_stencil = DepthStencilTexture::init(backend.device(), size.into());

    let mut elapsed = 0;
    let mut counter = 0;

    let text_compositor = GlyphCompositor::init(
        backend.device(),
        textures.texture2d_bind_group_layout(),
        pipeline_desc.clone(),
    );

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } if window_id == window.id() => {
                let screen_size = (size.width, size.height).into();
                backend.update_view(screen_size);
                depth_stencil = DepthStencilTexture::init(backend.device(), screen_size);
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,

            Event::RedrawRequested(_) => {
                let mut renderer = StoryboardRenderer::new();

                let start = Instant::now();

                if let Ok(current_texture) = backend.get_current_texture(&surface) {
                    let size: Size2D<f32, PixelUnit> = (
                        window.inner_size().width as f32,
                        window.inner_size().height as f32,
                    )
                        .into();

                    let space = DrawSpace::new_screen(Rect {
                        origin: Point2D::default(),
                        size,
                    });

                    let view = current_texture
                        .texture
                        .create_view(&storyboard_graphics::wgpu::TextureViewDescriptor::default());

                    let rotation = &mut rect_node.transform_mut().rotation;

                    rotation.x = ExtentUnit::Percent(rotation.x.value() + 0.00003);
                    rotation.y = ExtentUnit::Percent(rotation.y.value() + 0.00001);
                    rotation.z = ExtentUnit::Percent(rotation.z.value() + 0.00002);

                    rect_node.update(&space);

                    renderer.append(box_compositor.box_2d(&rect, &rect_node.get_draw_box(&space)));

                    let text: String = format!(
                        "텍스트 테스트.\nRender took {} ms, {} fps",
                        elapsed / 1000,
                        1000000.0 / elapsed as f64
                    )
                    .into();

                    let font = glyph_brush.draw_font().clone();

                    /*
                    let mut glyphs = {
                        let mut vec = Vec::new();

                        for text in TextLayout::new(
                            &mut font,
                            &text,
                            TextDirection::LeftToRight,
                            MatchingPresentation::Required,
                            0,
                            None,
                            None,
                            true,
                        ) {
                            for glyph in text.list {
                                vec.push(glyph);
                            }
                        }

                        vec
                    };

                    renderer.append(text_compositor.text(
                        backend.queue(),
                        &mut glyph_brush,
                        &glyphs,
                        &TextStyle {
                            size: 96.0,
                            color: ShapeColor::default(),
                        },
                        &space,
                        (150.0, 150.0).into(),
                    ));
                    */

                    renderer.append(path_compositor.path_scalable(
                        &path_style,
                        &space.inner_box(
                            Rect {
                                origin: (0.0, 0.0).into(),
                                size: (900.0, 900.0).into(),
                            },
                            None,
                        ),
                    ));

                    let mut encoder =
                        backend
                            .device()
                            .create_command_encoder(&CommandEncoderDescriptor {
                                label: Some("Example encoder"),
                            });

                    let mut draw_context = DrawContext {
                        device: backend.device(),
                        queue: backend.queue(),
                        textures: &textures,
                        stream_allocator: &mut stream_allocator,
                    };

                    renderer.prepare(&mut draw_context, &mut encoder);

                    let render_context = draw_context.into_render_context();
                    let pass = encoder.begin_render_pass(&RenderPassDescriptor {
                        label: Some("Test render pass"),
                        color_attachments: &[RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: Operations {
                                load: LoadOp::Clear(Color::BLACK),
                                store: true,
                            },
                        }],
                        depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                            view: depth_stencil.view(),
                            depth_ops: Some(Operations {
                                load: LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: Some(Operations {
                                load: LoadOp::Clear(0),
                                store: true,
                            }),
                        }),
                    });

                    renderer.render(&render_context, pass);

                    renderer.finish();

                    backend.queue().submit(iter::once(encoder.finish()));
                    elapsed = start.elapsed().as_micros();

                    counter += elapsed;

                    if counter > 1_000_000 {
                        counter = 0;

                        println!(
                            "Render took {} ms, {} fps",
                            elapsed / 1000,
                            1000000.0 / elapsed as f64
                        );
                    }

                    current_texture.present();
                }
            }

            _ => {}
        }

        window.request_redraw();
    });
}
