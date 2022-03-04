// Based on the Vulkano triangle example.

// Triangle example Copyright (c) 2016 The vulkano developers
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>,
// at your option. All files in the project carrying such
// notice may not be copied, modified, or distributed except
// according to those terms.

pub mod animations;
pub mod sprite;
pub mod tiles;
pub mod types;
use tiles::{Tile, Tilemap, Tileset};

use animations::{Animation, AnimationSet, AnimationState};
// use png;
use sprite::{Action, Character, Sprite};
use std::collections::hash_map::HashMap;
use std::io::Cursor;
use std::rc::Rc;
use std::sync::Arc;
use types::{Color, Image, Rect, Vec2i};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, SubpassContents};
use vulkano::descriptor_set::PersistentDescriptorSet;
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceExtensions, Features, Queue, QueuesIter};
use vulkano::format::Format;

use vulkano::image::ImageCreateFlags;

use vulkano::image::{
    view::ImageView, ImageAccess, ImageDimensions, ImageUsage, StorageImage, SwapchainImage,
};
use vulkano::instance::Instance;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint};
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
use vulkano::sampler::{Filter, MipmapMode, Sampler, SamplerAddressMode};
use vulkano::shader::ShaderModule;
use vulkano::swapchain::{self, AcquireError, Surface, Swapchain, SwapchainCreationError};
use vulkano::sync::{self, FlushError, GpuFuture};
use vulkano::Version;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

const WIDTH: usize = 320;
const HEIGHT: usize = 320;

const PLAYER_WIDTH: i32 = 20;
const PLAYER_HEIGHT: i32 = 32;

const TILE_SZ: i32 = 16;

#[derive(Default, Debug, Clone)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}
vulkano::impl_vertex!(Vertex, position, uv);

pub struct Vk {
    instance: Arc<Instance>,
    event_loop: EventLoop<()>,
    surface: Arc<Surface<Window>>,
    device: Arc<Device>,
    queues: QueuesIter,
    queue: Arc<Queue>,
    swapchain: Arc<Swapchain<Window>>,
    images: Vec<Arc<SwapchainImage<Window>>>,
    vs: Arc<ShaderModule>,
    fs: Arc<ShaderModule>,
}

impl Vk {
    pub fn new() -> Self {
        let required_extensions = vulkano_win::required_extensions();
        let instance = Instance::new(None, Version::V1_1, &required_extensions, None).unwrap();
        let event_loop = EventLoop::new();
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::none()
        };
        let (physical_device, queue_family) = PhysicalDevice::enumerate(&instance)
            .filter(|&p| p.supported_extensions().is_superset_of(&device_extensions))
            .filter_map(|p| {
                p.queue_families()
                    .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
                    .map(|q| (p, q))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
            })
            .unwrap();
        let (device, mut queues) = Device::new(
            physical_device,
            &Features::none(),
            &physical_device
                .required_extensions()
                .union(&device_extensions),
            [(queue_family, 0.5)].iter().cloned(),
        )
        .unwrap();
        let queue = queues.next().unwrap();
        let (mut swapchain, images) = {
            let caps = surface.capabilities(physical_device).unwrap();
            let composite_alpha = caps.supported_composite_alpha.iter().next().unwrap();
            let format = caps.supported_formats[0].0;
            let dimensions: [u32; 2] = surface.window().inner_size().into();
            Swapchain::start(device.clone(), surface.clone())
                .num_images(caps.min_image_count)
                .format(format)
                .dimensions(dimensions)
                .usage(ImageUsage::color_attachment())
                .sharing_mode(&queue)
                .composite_alpha(composite_alpha)
                .build()
                .unwrap()
        };

        mod vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: "
                        #version 450
        
                        layout(location = 0) in vec2 position;
                        layout(location = 1) in vec2 uv;
                        layout(location = 0) out vec2 out_uv;
                        void main() {
                            gl_Position = vec4(position, 0.0, 1.0);
                            out_uv = uv;
                        }
                    "
            }
        }

        mod fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: "
                        #version 450
        
                        layout(set = 0, binding = 0) uniform sampler2D tex;
                        layout(location = 0) in vec2 uv;
                        layout(location = 0) out vec4 f_color;
        
                        void main() {
                            f_color = texture(tex, uv);
                        }
                    "
            }
        }

        let vs = vs::load(device.clone()).unwrap();
        let fs = fs::load(device.clone()).unwrap();
        Vk {
            instance,
            event_loop,
            surface,
            device,
            queues,
            queue,
            swapchain,
            images,
            vs,
            fs,
        }
    }
}

pub struct VkState {
    render_pass: Arc<RenderPass>,
    viewport: Viewport,
    framebuffers: Vec<Arc<Framebuffer>>,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl VkState {
    pub fn new(vk: &Vk) -> Self {
        let render_pass = vulkano::single_pass_renderpass!(
            vk.device.clone(),
            attachments: {
                color: {
                    // Pro move: We're going to cover the screen completely. Trust us!
                    load: DontCare,
                    store: Store,
                    format: vk.swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                color: [color],
                depth_stencil: {}
            }
        )
        .unwrap();

        let mut viewport = Viewport {
            origin: [0.0, 0.0],
            dimensions: [0.0, 0.0],
            depth_range: 0.0..1.0,
        };

        let mut framebuffers =
            window_size_dependent_setup(&vk.images, render_pass.clone(), &mut viewport);

        let mut recreate_swapchain = false;
        let mut previous_frame_end = Some(sync::now(vk.device.clone()).boxed());

        VkState {
            render_pass,
            viewport,
            framebuffers,
            recreate_swapchain,
            previous_frame_end,
        }
    }
}

pub struct FBState {
    pipeline: Arc<GraphicsPipeline>,
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    fb2d_buffer: Arc<CpuAccessibleBuffer<[Color]>>,
    fb2d_image: Arc<StorageImage>,
    set: Arc<PersistentDescriptorSet>,
    fb2d: Image,
}

impl FBState {
    pub fn new(vk: &Vk, vk_state: &VkState, fb2d: Image) -> Self {
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            vk.device.clone(),
            BufferUsage::all(),
            false,
            [
                Vertex {
                    position: [-1.0, -1.0],
                    uv: [0.0, 0.0],
                },
                Vertex {
                    position: [3.0, -1.0],
                    uv: [2.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 3.0],
                    uv: [0.0, 2.0],
                },
            ]
            .iter()
            .cloned(),
        )
        .unwrap();

        let fb2d_buffer = CpuAccessibleBuffer::from_iter(
            vk.device.clone(),
            BufferUsage::transfer_source(),
            false,
            (0..WIDTH * HEIGHT).map(|_| (255_u8, 0_u8, 0_u8, 0_u8)),
        )
        .unwrap();
        // Let's set up the Image we'll copy into:
        let dimensions = ImageDimensions::Dim2d {
            width: WIDTH as u32,
            height: HEIGHT as u32,
            array_layers: 1,
        };
        let fb2d_image = StorageImage::with_usage(
            vk.device.clone(),
            dimensions,
            Format::R8G8B8A8_UNORM,
            ImageUsage {
                // This part is key!
                transfer_destination: true,
                sampled: true,
                storage: true,
                transfer_source: false,
                color_attachment: false,
                depth_stencil_attachment: false,
                transient_attachment: false,
                input_attachment: false,
            },
            ImageCreateFlags::default(),
            std::iter::once(vk.queue.family()),
        )
        .unwrap();
        // Get a view on it to use as a texture:
        let fb2d_texture = ImageView::new(fb2d_image.clone()).unwrap();
        let fb2d_sampler = Sampler::new(
            vk.device.clone(),
            Filter::Linear,
            Filter::Linear,
            MipmapMode::Nearest,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            SamplerAddressMode::Repeat,
            0.0,
            1.0,
            0.0,
            0.0,
        )
        .unwrap();

        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vk.vs.entry_point("main").unwrap(), ())
            .input_assembly_state(InputAssemblyState::new())
            .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
            .fragment_shader(vk.fs.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(vk_state.render_pass.clone(), 0).unwrap())
            .build(vk.device.clone())
            .unwrap();

        let layout = pipeline.layout().descriptor_set_layouts().get(0).unwrap();
        let mut set_builder = PersistentDescriptorSet::start(layout.clone());

        set_builder
            .add_sampled_image(fb2d_texture, fb2d_sampler)
            .unwrap();

        let set = set_builder.build().unwrap();

        FBState {
            pipeline,
            vertex_buffer,
            fb2d_buffer,
            fb2d_image,
            set,
            fb2d,
        }
    }
}

struct GameState {
    sprite: Sprite,
    animation_set: AnimationSet,
    speedup_factor: usize,
}

impl GameState {
    pub fn new(character: Character) -> Self {
        let animation_set = AnimationSet::new(character);
        let sprite = Sprite {
            character: character,
            action: Action::Walk,
            animation_state: animation_set.play_animation(Action::Walk),
            shape: Rect {
                pos: Vec2i { x: 20, y: 20 },
                sz: Vec2i {
                    x: PLAYER_WIDTH as i32,
                    y: PLAYER_HEIGHT as i32,
                },
            },
        };
        let speedup_factor = 5; // this acts more like a slow down factor.
        GameState {
            sprite: sprite,
            animation_set: animation_set,
            speedup_factor: speedup_factor,
        }
    }
}
#[derive(Copy, Clone)]
struct Rect2 {
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    color: Color,
}

impl Rect2 {
    fn change_color_to(&mut self, new: Color) {
        self.color = new;
    }

    #[allow(dead_code)]
    fn left(self) -> usize {
        self.x
    }

    #[allow(dead_code)]
    fn right(self) -> usize {
        self.x + self.width
    }

    #[allow(dead_code)]
    fn top(self) -> usize {
        self.y
    }

    #[allow(dead_code)]
    fn bottom(self) -> usize {
        self.y + self.height
    }

    #[allow(dead_code)]
    fn change_x_by(&mut self, change: f32) {
        let mut x = self.x as f32;
        x = x + change;

        if x < 0.0 {
            self.x = 0;
        } else if x as usize + self.width > WIDTH {
            self.x = WIDTH - self.width;
        } else {
            self.x = x as usize;
        }
    }

    #[allow(dead_code)]
    fn change_y_by(&mut self, change: f32) {
        let mut y = self.y as f32;
        y = y + change;

        if y < 0.0 {
            self.y = 0;
        } else if y as usize + self.height > HEIGHT {
            self.y = HEIGHT - self.height;
        } else {
            self.y = y as usize;
        }
    }
}

fn render3d(vk: &mut Vk, vk_state: &mut VkState, fb_state: &FBState) {
    {
        // We need to synchronize here to send new data to the GPU.
        // We can't send the new framebuffer until the previous frame is done being drawn.
        // Dropping the future will block until it's done.
        if let Some(mut fut) = vk_state.previous_frame_end.take() {
            fut.cleanup_finished();
        }
    }
    // Now we can copy into our buffer.
    {
        let writable_fb = &mut *fb_state.fb2d_buffer.write().unwrap();
        writable_fb.copy_from_slice(&fb_state.fb2d.buffer);
    }

    if vk_state.recreate_swapchain {
        let dimensions: [u32; 2] = vk.surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match vk.swapchain.recreate().dimensions(dimensions).build() {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };

        vk.swapchain = new_swapchain;
        vk_state.framebuffers = window_size_dependent_setup(
            &new_images,
            vk_state.render_pass.clone(),
            &mut vk_state.viewport,
        );
        vk_state.recreate_swapchain = false;
    }
    let (image_num, suboptimal, acquire_future) =
        match swapchain::acquire_next_image(vk.swapchain.clone(), None) {
            Ok(r) => r,
            Err(AcquireError::OutOfDate) => {
                vk_state.recreate_swapchain = true;
                return;
            }
            Err(e) => panic!("Failed to acquire next image: {:?}", e),
        };
    if suboptimal {
        vk_state.recreate_swapchain = true;
    }

    let mut builder = AutoCommandBufferBuilder::primary(
        vk.device.clone(),
        vk.queue.family(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    builder
        // Now copy that framebuffer buffer into the framebuffer image
        .copy_buffer_to_image(fb_state.fb2d_buffer.clone(), fb_state.fb2d_image.clone())
        .unwrap()
        // And resume our regularly scheduled programming
        .begin_render_pass(
            vk_state.framebuffers[image_num].clone(),
            SubpassContents::Inline,
            std::iter::once(vulkano::format::ClearValue::None),
        )
        .unwrap()
        .set_viewport(0, [vk_state.viewport.clone()])
        .bind_pipeline_graphics(fb_state.pipeline.clone())
        .bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            fb_state.pipeline.layout().clone(),
            0,
            fb_state.set.clone(),
        )
        .bind_vertex_buffers(0, fb_state.vertex_buffer.clone())
        .draw(fb_state.vertex_buffer.len() as u32, 1, 0, 0)
        .unwrap()
        .end_render_pass()
        .unwrap();

    let command_buffer = builder.build().unwrap();

    let future = acquire_future
        .then_execute(vk.queue.clone(), command_buffer)
        .unwrap()
        .then_swapchain_present(vk.queue.clone(), vk.swapchain.clone(), image_num)
        .then_signal_fence_and_flush();

    match future {
        Ok(future) => {
            vk_state.previous_frame_end = Some(future.boxed());
        }
        Err(FlushError::OutOfDate) => {
            vk_state.recreate_swapchain = true;
            vk_state.previous_frame_end = Some(sync::now(vk.device.clone()).boxed());
        }
        Err(e) => {
            println!("Failed to flush future: {:?}", e);
            vk_state.previous_frame_end = Some(sync::now(vk.device.clone()).boxed());
        }
    }
}

pub fn main() {
    let mut vk = Vk::new();
    let mut vk_state = VkState::new(&vk);
    let event_loop = EventLoop::new();
    let mut game_state = GameState::new(Character::Luigi);

    // ---------------------------------------------------------------------------------
    // Beginning of Game Stuff
    let fb2d = Image {
        buffer: vec![(0, 0, 0, 255); (HEIGHT * WIDTH) as usize].into_boxed_slice(),
        sz: Vec2i {
            x: WIDTH as i32,
            y: HEIGHT as i32,
        },
    };
    let mut to = Vec2i { x: 0, y: 0 };

    let mut fb_state = FBState::new(&vk, &vk_state, fb2d);
    let mut now_keys = [false; 255];
    let mut prev_keys = now_keys.clone();
    let mut now_lmouse = false;
    let mut prev_lmouse = false;
    let event_loop = EventLoop::new();

    // PLAYER
    let mut player = Rect {
        pos: Vec2i {
            x: (WIDTH as i32) / 4 - PLAYER_WIDTH / 2,
            y: (HEIGHT as i32) - 48 - PLAYER_HEIGHT,
        },
        sz: Vec2i {
            x: PLAYER_WIDTH,
            y: PLAYER_HEIGHT,
        },
    };

    let mut vx = 0.0;
    let mut vy = 0.0;
    let mut ax = 0.0;
    let ay = 0.2;

    // BALL
    let mut ball = Rect {
        pos: Vec2i {
            x: 0,
            y: 0,
        },
        sz: Vec2i {
            x: 8,
            y: 8,
        },
    };

    let mut bvx = 0.0;
    let mut bvy = 0.0;
    let bay = 0.2;

    let mut ball_shot = false;

    // POWER METER
    let mut meter = Rect {
        pos: Vec2i {
            x: 0,
            y: 0,
        },
        sz: Vec2i {
            x: 4,
            y: 0,
        },
    };

    // End of Game stuff
    // ---------------------------------------------------------------

    let mut jumping = false;

    let img = Rc::new(Image::from_file(std::path::Path::new(
        "content/tilesheet.png",
    )));
    let tileset = Rc::new(Tileset::new(
        vec![
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: true },
            Tile { solid: false },
            Tile { solid: false },
            Tile { solid: false },
            Tile { solid: false },
            Tile { solid: false },
        ],
        img.clone(),
    ));
    let map = Tilemap::new(
        Vec2i { x: 0, y: 0 },
        (20, 20),
        tileset.clone(),
        vec![
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 6,
            6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
            6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 9, 6, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8, 5, 5, 5, 5, 5, 5, 6,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2, 2, 2, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4,
            4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
        ],
    );

    map.draw(&mut fb_state.fb2d);

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                vk_state.recreate_swapchain = true;
            }
            Event::NewEvents(_) => {
                // Leave now_keys alone, but copy over all changed keys
                prev_keys.copy_from_slice(&now_keys);
                prev_lmouse = now_lmouse;
            }
            Event::WindowEvent {
                // Note this deeply nested pattern match
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            winit::event::KeyboardInput {
                                // Which serves to filter out only events we actually want
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                // It also binds these handy variable names!
                match state {
                    winit::event::ElementState::Pressed => {
                        // VirtualKeycode is an enum with a defined representation
                        now_keys[keycode as usize] = true;
                    }
                    winit::event::ElementState::Released => {
                        now_keys[keycode as usize] = false;
                    }
                }
            }
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        button: btn, state, ..
                    },
                ..
            } => {
                match btn {
                    winit::event::MouseButton::Left => {
                        match state {
                            winit::event::ElementState::Pressed => {
                                // VirtualKeycode is an enum with a defined representation
                                now_lmouse = true;
                            }
                            winit::event::ElementState::Released => {
                                now_lmouse = false;
                            }
                        }
                    }
                    _ => {}
                }
            }
            Event::MainEventsCleared => {
                {
                    // We need to synchronize here to send new data to the GPU.
                    // We can't send the new framebuffer until the previous frame is done being drawn.
                    // Dropping the future will block until it's done.
                    if let Some(mut fut) = vk_state.previous_frame_end.take() {
                        fut.cleanup_finished();
                    }
                }

                // Mouse Events
                if now_lmouse && !prev_lmouse {}

                // Keyboard Events
                if now_keys[VirtualKeyCode::Escape as usize] {
                    *control_flow = ControlFlow::Exit;
                }
                if now_keys[VirtualKeyCode::LShift as usize]
                    || now_keys[VirtualKeyCode::RShift as usize]
                {
                } else {
                }

                if !ball_shot {
                    if now_keys[VirtualKeyCode::Space as usize]
                    && !prev_keys[VirtualKeyCode::Space as usize] {

                    } else if !now_keys[VirtualKeyCode::Space as usize]
                        && prev_keys[VirtualKeyCode::Space as usize] {

                    }
                    
                }
                if now_keys[VirtualKeyCode::Space as usize]
                    && !prev_keys[VirtualKeyCode::Space as usize]
                    && !ball_shot
                {
                    ball_shot = true;
                    ball.pos = player.pos;
                    bvx = -5.0;
                    bvy = -3.0;
                }

                if now_keys[VirtualKeyCode::Up as usize]
                    && !prev_keys[VirtualKeyCode::Up as usize]
                    && !jumping
                {
                    vy = -5.0;
                    jumping = true;
                }

                if now_keys[VirtualKeyCode::Down as usize] {}
                if now_keys[VirtualKeyCode::Left as usize] {
                    if vx > -2.0 {
                        ax = -0.2;
                    } else {
                        ax = 0.0
                    }
                } else if now_keys[VirtualKeyCode::Right as usize] {
                    if vx < 2.0 {
                        ax = 0.2;
                    } else {
                        ax = 0.0
                    }
                } else {
                    if vx > 0.1 {
                        ax = -0.1
                    } else if vx < -0.1 {
                        ax = 0.1
                    } else {
                        ax = 0.0
                    }
                }
                // It's debatable whether the following code should live here or in the drawing section.
                // First clear the framebuffer...

                // clear the framebuffer back to the static level
                map.draw(&mut fb_state.fb2d);
                fb_state.fb2d.bitblt(
                    game_state.animation_set.get_image(),
                    game_state.sprite.play_animation(&game_state.speedup_factor),
                    player.pos,
                );

                vx += ax;
                vy += ay;
                player.move_by(vx as i32, vy as i32);

                let mut ovs = vec![];
                let cpts = vec![(0,0),(0,1),(0,2),(1,0),(1,2),(2,0),(2,1),(2,2)];
                for (i,j) in cpts {
                    let p = Vec2i {
                        x: player.pos.x + i * (player.sz.x / 2),
                        y: player.pos.y + j * (player.sz.y / 2),
                    };
                    let r = map.tile_at(p);
                    if r.1.solid {
                        let mut ov = Vec2i { x: 0, y: 0 };
                        if vx > 0.0 {
                            ov.x = r.0.x - (player.pos.x + PLAYER_WIDTH);
                        } else {
                            ov.x = (r.0.x + TILE_SZ) - player.pos.x;
                        }

                        if vy > 0.0 {
                            ov.y = r.0.y - (player.pos.y + PLAYER_HEIGHT);
                        } else {
                            ov.y = (r.0.y + TILE_SZ) - player.pos.y;
                        }

                        ovs.push(ov);
                    }
                }

                let mut disps = Vec2i { x: 0, y: 0 };
                let mut resolved = false;
                for ov in ovs.iter() {
                    // Touching but not overlapping
                    if ov.x == 0 && ov.y == 0 {
                        resolved = true;
                        // Maybe track "I'm touching it on this side or that side"
                        break;
                    }
                    // Is this more of a horizontal collision... (and we are allowed to displace horizontally)
                    if ov.x.abs() <= ov.y.abs() && ov.x.signum() != -disps.x.signum() {
                        // Record that we moved by o.x, to avoid contradictory moves later
                        disps.x += ov.x;
                        // Actually move player pos
                        player.pos.x += ov.x;
                        vx = 0.0;
                        // Mark collision for the player as resolved.
                        resolved = true;
                        break;
                        // or is it more of a vertical collision (and we are allowed to displace vertically)
                    } else if ov.y.abs() <= ov.x.abs() && ov.y.signum() != -disps.y.signum() {
                        disps.y += ov.y;
                        player.pos.y += ov.y;
                        vy = 0.0;
                        jumping = false;
                        resolved = true;
                        break;
                    }
                }
                // Couldn't resolve collision, player must be squashed or trapped (e.g. by a moving platform)
                if !resolved {
                    // In your game, this might mean killing the player character or moving them somewhere else
                }

                // check to make sure player is in screen bounds
                if player.pos.x < 0 {
                    player.pos.x = 0
                }
                if player.pos.x > WIDTH as i32 - player.sz.x {
                    player.pos.x = WIDTH as i32 - player.sz.x
                }
                if player.pos.y < 0 {
                    player.pos.y = 0;
                }
                if player.pos.y > HEIGHT as i32 - player.sz.y {
                    player.pos.y = HEIGHT as i32 - player.sz.y;
                }

                if ball_shot {
                    if ball.pos.x + ball.sz.x > 0 && ball.pos.y < HEIGHT as i32 {
                        bvy += bay;

                        ball.move_by(bvx as i32, bvy as i32);

                        fb_state.fb2d.draw_rect(&ball, (255,255,255,255));
                    } else {
                        ball_shot = false;
                        bvx = 0.0;
                        bvy = 0.0;
                    }
                }

                render3d(&mut vk, &mut vk_state, &fb_state);
            }
            _ => (),
        }
    });
}

fn window_size_dependent_setup(
    images: &[Arc<SwapchainImage<Window>>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let dimensions = images[0].dimensions().width_height();
    viewport.dimensions = [dimensions[0] as f32, dimensions[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new(image.clone()).unwrap();
            Framebuffer::start(render_pass.clone())
                .add(view)
                .unwrap()
                .build()
                .unwrap()
        })
        .collect::<Vec<_>>()
}

// fn get_animation_state(animation_states: &Vec<AnimationState>, index: usize) -> AnimationState {
//     animation_states[index].clone()
// }
