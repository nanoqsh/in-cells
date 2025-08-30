use {
    crate::{App, anim::Animate, sprite},
    dunge::{
        buffer::{Filter, Format, Sampler},
        glam::{IVec2, UVec2, Vec2, Vec4},
        group::BoundTexture,
        storage::Uniform,
    },
    dunge_winit::{prelude::*, winit::keyboard::KeyCode},
    futures_concurrency::prelude::*,
    sl::{Groups, PassVertex, Render},
    std::cell::Cell,
};

pub async fn run(control: Control) -> App<()> {
    #[repr(C)]
    #[derive(Vertex)]
    struct Vert {
        xy: Vec2,
        uv: Vec2,
    }

    const fn sprite(index: usize) -> [Vert; 4] {
        let q = index as f32 / 5.;
        let w = (index + 1) as f32 / 5.;

        [
            Vert {
                xy: Vec2::new(-1., -1.),
                uv: Vec2::new(q, 1.),
            },
            Vert {
                xy: Vec2::new(1., -1.),
                uv: Vec2::new(w, 1.),
            },
            Vert {
                xy: Vec2::new(1., 1.),
                uv: Vec2::new(w, 0.),
            },
            Vert {
                xy: Vec2::new(-1., 1.),
                uv: Vec2::new(q, 0.),
            },
        ]
    }

    #[derive(Group)]
    struct Map<'app> {
        texture: BoundTexture,
        sampler: Sampler,
        camera_scale_offset: &'app Uniform<Vec4>,
    }

    let sprite_shader = |PassVertex(v): PassVertex<Vert>, Groups(m): Groups<Map>| {
        let s = sl::thunk(m.camera_scale_offset.load());
        let scale = sl::vec2(s.clone().x(), s.clone().y());
        let offset = sl::vec2(s.clone().z(), s.w());
        let xy = v.xy * scale + offset;

        Render {
            place: sl::vec4_concat(xy, Vec2::new(1., 1.)),
            color: sl::texture_sample(m.texture, m.sampler, sl::fragment(v.uv)),
        }
    };

    let cx = dunge::context().await?;
    let shader = cx.make_shader(sprite_shader);

    let animate_camera = Cell::new(Animate::new(Vec2::ZERO));
    let screen_size = Cell::new(UVec2::ONE);
    let camera = || {
        let m = const {
            let sprite_size = 6;
            let sprite_scale = 8;
            (sprite_size * sprite_scale) as f32
        };

        let scale = m / screen_size.get().as_vec2();
        let offset = scale * animate_camera.get().point() * 2.;
        Vec4::new(scale.x, scale.y, offset.x, offset.y)
    };

    let camera_uniform = cx.make_uniform(&camera());
    let map = {
        let (spritemap, width, height) = sprite::map()?;
        let data = TextureData::new((width, height), Format::SrgbAlpha, &spritemap)?.bind();

        cx.make_set(
            &shader,
            Map {
                texture: cx.make_texture(data).bind(),
                sampler: cx.make_sampler(Filter::Nearest),
                camera_scale_offset: &camera_uniform,
            },
        )
    };

    let mesh = cx.make_mesh(&MeshData::from_quads(const { &[sprite(3)] })?);

    let window = control.make_window(&cx).await?;
    let layer = cx.make_layer(&shader, window.format());

    let bg = window.format().rgb_from_bytes([0; 3]);
    let render = async {
        loop {
            let redraw = window.redraw().await;

            // animate camera movement
            let dt = redraw.delta_time().as_secs_f32() * 4.;
            animate_camera.update(|a| a.advance(dt));
            camera_uniform.update(&cx, &camera());

            cx.shed(|s| {
                s.render(&redraw, bg).layer(&layer).set(&map).draw(&mesh);
            })
            .await;

            redraw.present();
        }
    };

    let resize = async {
        loop {
            let (width, height) = window.resized().await;
            screen_size.set(UVec2::new(width, height));
        }
    };

    let camera_position = Cell::new(IVec2::ZERO);
    let movement = {
        let moves = [
            (KeyCode::KeyW, IVec2::new(0, -1)),
            (KeyCode::KeyS, IVec2::new(0, 1)),
            (KeyCode::KeyA, IVec2::new(1, 0)),
            (KeyCode::KeyD, IVec2::new(-1, 0)),
        ];

        moves
            .map(async |(key, dp)| {
                loop {
                    window.key_pressed(key).await;
                    camera_position.update(|p| p + dp);
                    animate_camera.update(|a| a.with_target(camera_position.get().as_vec2()));
                }
            })
            .race()
    };

    let close = window.key_pressed(KeyCode::Escape);
    (render, resize, movement, close).race().await;

    Ok(())
}
