use super::errors::{Error, ErrorKind, Result};
use super::materials::Materials;
use super::meshes::Meshes;
use super::pipeline::{Model, RenderPipeline};
use super::projections::Projections;
use super::shaders::Shaders;
use super::system::System;
use super::text::TextRenderer;
use super::tick::Tick;
use super::transforms::Transforms;
use super::uniforms::Uniforms;
use super::window::Window;
use crate::internal_derive::DependenciesFrom;
use failchain::ResultExt;
use glium::{BackfaceCullingMode, Depth, DepthTest, DrawParameters, Surface};
use log::{error, info};
use math::prelude::*;
use math::Mat4;

// Зависимости рендера
#[derive(DependenciesFrom)]
pub struct Dependencies<'context> {
    pipe: &'context mut RenderPipeline,
    meshes: &'context Meshes,
    materials: &'context Materials,
    shaders: &'context Shaders,
    text: &'context TextRenderer,
    window: &'context Window,
    transforms: &'context Transforms,
    projections: &'context Projections,
    uniforms: &'context mut Uniforms,
    tick: &'context Tick,
}

// Непосредственно сам рендер
pub struct Renderer {
    draw_parameters: DrawParameters<'static>,
    removed: Vec<usize>,
}

// Реализация System для рендера
impl<'context> System<'context> for Renderer {
    // Указываем зависимости рендера
    type Dependencies = Dependencies<'context>;
    type Error = Error;

    fn debug_name() -> &'static str {
        "renderer"
    }

    // Создание рендера из конфига
    // Параметр не используется, но он состоит весь из ссылок
    fn create(_deps: Dependencies) -> Result<Self> {
        let depth_parameters = Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Depth::default() // Все остальные параметры получаем из дефолтного значения
        };

        let draw_parameters = DrawParameters {
            depth: depth_parameters,
            backface_culling: BackfaceCullingMode::CullClockwise,
            ..DrawParameters::default() // Все остальные параметры получаем из дефолтного значения
        };

        Ok(Renderer {
            draw_parameters: draw_parameters,
            removed: Vec::with_capacity(32),
        })
    }

    // Вызов отрисовки, зависимость состоит из ссылок
    fn update(&mut self, deps: Dependencies) -> Result<()> {
        // Если не нужно рендерить, то просто пропускаем
        if !deps.tick.need_render_frame() {
            return Ok(());
        }

        // Получаем ссылку на пайплайн
        let pipe = deps.pipe;

        // Если нету камеры, не рендерим
        let camera_id = if let Some(camera_id) = pipe.camera {
            camera_id
        } else {
            return Ok(());
        };

        // Вычисляем трансформацию отображения инвертируя камеру
        let view_transform = if let Some(transform) = deps.transforms.get_absolute(camera_id) {
            transform
                .inverse_transform()
                .expect("singular view transform")
        } else {
            info!("Camera transform missing, disabling renderer.");
            pipe.camera = None;
            return Ok(());
        };

        // С помощью into превращаем в матрицу
        let view_matrix = view_transform.into();

        // Устанавливаем матрицу проекции
        let proj_matrix = *deps
            .projections
            .get_matrix(camera_id)
            .expect("camera projection missing");
        *deps
            .uniforms
            .get_mat4_mut(pipe.projection)
            .expect("projection uniform missing") = proj_matrix;

        // Рендерим все модели
        let mut frame = deps.window.draw();
        let models_iter = pipe.models.access().iter().enumerate();
        for (index, &Model{ mesh, material }) in models_iter {
            // For each model we need to assemble three things to render it: transform, mesh and
            // material. We get the entity id and query the corresponding systems for it.
            // Получаем id нашей модели
            let entity = pipe
                .models
                .index_to_id(index)
                .expect("bad index enumerating models: mesh");

            // Если меша нету, то сущность может быть удалена
            // сохраняем в список удаленных
            let mesh = if let Some(mesh) = deps.meshes.get(mesh) {
                mesh
            } else {
                info!(
                    "Mesh missing {:?} in model for entity {:?}, removing.",
                    mesh, entity
                );
                self.removed.push(index);
                continue;
            };

            // Если у модели есть трансформ, тогда умножаем ее на матрицу вида для получения модель-вью матрицы.
            // Если нету - просто используем матрицу модели
            *deps
                .uniforms
                .get_mat4_mut(pipe.modelview)
                .expect("modelview uniform missing") =
                if let Some(model_transform) = deps.transforms.get_absolute(entity) {
                    Mat4::from(view_transform.concat(model_transform))
                } else {
                    view_matrix
                };

            // Выбираем материал отрисовки
            let material =
                if let Some(material) = deps.materials.get(deps.shaders, deps.uniforms, material) {
                    material
                } else {
                    // Если у мена нету материала, тогда модель кривая - надо удалить
                    error!(
                        "Material missing {:?} in model for entity {:?}, removing.",
                        material, entity
                    );
                    self.removed.push(index);
                    continue;
                };

            // Рисуем меш
            frame
                .draw(
                    &mesh,
                    &mesh,
                    material.shader(),
                    &material,
                    &self.draw_parameters,
                )
                .map_err(ErrorKind::glium("renderer"))?;
        }

        // Render text. TODO(cristicbz): text should render itself :(
        deps.text
            .render(&mut frame)
            .chain_err(|| ErrorKind::System("render bypass", TextRenderer::debug_name()))?;

        // TODO(cristicbz): Re-architect a little bit to support rebuilding the context.
        frame
            .finish()
            .expect("Cannot handle context loss currently :(");

        // Удаляем отсутствующие модели в обратном порядке
        for &index in self.removed.iter().rev() {
            pipe.models.remove_by_index(index);
        }
        self.removed.clear();
        Ok(())
    }
}
