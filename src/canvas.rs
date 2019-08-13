use gfx::{texture as tex, memory as mem, format as fmt, Factory, handle};
use imgui::{Ui, TextureId, im_str, Condition, ImGuiWindowFlags, StyleVar};
use rand::Rng;

use crate::{
    support::{FrameData, RenderSystem, types}, 
    simulation::{Simulation, ErasedSimulation}, 
    cells::Cell,
};

pub struct Canvas(Option<SimTexture>);

impl Canvas {
    pub fn new() -> Self {
        Canvas(None)
    }

    pub fn draw(&mut self, frame_data: &mut FrameData, simulation: &mut dyn ErasedSimulation) {
        self.update_textures(frame_data, simulation);
        self.draw_simulation(frame_data.ui, simulation);
    }

    fn update_textures(&mut self, frame_data: &mut FrameData, simulation: &dyn ErasedSimulation) {
        let mut tex = self.0.take()
            .and_then(|x| match x.size() == simulation.size() {
                true => Some(x),
                false => { x.cleanup(frame_data.render_sys); None }
            })
            .unwrap_or_else(|| SimTexture::new(frame_data.render_sys, simulation.size()));

        simulation.render_to(frame_data.encoder, &mut tex);
        self.0 = Some(tex);
    }

    fn draw_simulation(&mut self, ui: &Ui, simulation: &mut dyn ErasedSimulation) {
        if let Some(texture_id) = self.0.as_ref().map(SimTexture::id) {
            let display_size = ui.io().display_size;

            let scale_factor = f32::min(
                display_size[0] / simulation.size().0 as f32,
                display_size[1] / simulation.size().1 as f32,
            );

            let size = [
                scale_factor * simulation.size().0 as f32, 
                scale_factor * simulation.size().1 as f32
            ];

            let pos = [
                (display_size[0] - size[0]) / 2.0,
                (display_size[1] - size[1]) / 2.0,
            ];

            let _square_win = ui.push_style_var(StyleVar::WindowRounding(0.0));
            let _no_border = ui.push_style_var(StyleVar::WindowBorderSize(0.0));
            let _no_padding = ui.push_style_var(StyleVar::WindowPadding([0.0, 0.0]));

            ui.window(im_str!("Simulation view"))
                .position(pos, Condition::Always)
                .size(size, Condition::Always)
                .flags(
                    ImGuiWindowFlags::NoTitleBar | 
                    ImGuiWindowFlags::NoResize |
                    ImGuiWindowFlags::NoMove |
                    ImGuiWindowFlags::NoCollapse |
                    ImGuiWindowFlags::NoBackground |
                    ImGuiWindowFlags::NoBringToFrontOnFocus |
                    ImGuiWindowFlags::NoDecoration
                ).build(|| {
                    ui.image(texture_id, size).build()
                })
        }
    }
}

type TextureFormat = fmt::Srgba8;

pub struct SimTexture {
    size: (u16, u16),
    texture: handle::Texture<types::Resources, fmt::R8_G8_B8_A8>,
    id: TextureId,
}

impl SimTexture {
    pub fn new(render_sys: &mut RenderSystem, size: (u16, u16)) -> Self {
        let factory = &mut render_sys.factory;
        let texture = factory.create_texture(
            tex::Kind::D2(size.0, size.1, tex::AaMode::Single), 
            1, 
            mem::Bind::TRANSFER_DST | mem::Bind::SHADER_RESOURCE,
            mem::Usage::Dynamic,
            Some(fmt::ChannelType::Unorm),
        ).expect("Failed to create simulation texture");

        let srv = factory.view_texture_as_shader_resource::<TextureFormat>(&texture, (0, 0), fmt::Swizzle::new())
            .expect("Failed to create shader resource view");
        
        let sampler = factory.create_sampler(
            tex::SamplerInfo::new(tex::FilterMethod::Scale, tex::WrapMode::Tile)
        ); // do we need to destroy this stuff somewhere...?

        let textures = render_sys.renderer.textures();
        SimTexture { size, texture, id: textures.insert((srv, sampler)) }
    }

    pub(crate) fn update<C: Cell, R: Rng>(&mut self, encoder: &mut types::Encoder, simulation: &Simulation<C, R>) {
        assert_eq!(simulation.cells.size(), self.size);
        let image_info = tex::ImageInfoCommon {
            xoffset: 0, yoffset: 0, zoffset: 0, 
            width: self.size.0, height: self.size.1, depth: 1,
            format: (), mipmap: 0,
        };

        let data: Vec<[u8; 4]> = simulation.cells.iter().map(
            |c| c.to_color().correct_gamma().to_rgba()
        ).collect();

        encoder.update_texture::<_, TextureFormat>(&self.texture, None, image_info, &data)
            .expect("Failed to update texture data");
    }

    pub fn cleanup(self, render_sys: &mut RenderSystem) {
        render_sys.renderer.textures().remove(self.id);
    }

    pub(self) fn id(&self) -> TextureId { self.id }
    pub(self) fn size(&self) -> (u16, u16) { self.size }
}