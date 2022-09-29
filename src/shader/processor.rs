use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::{Path, PathBuf},
};

use crate::shader::{DefaultShader, ShaderError, ShaderRef};

use super::{FsShaderIo, Shader, ShaderIo};

const INCLUDE_DIRECTIVE: &str = "#include";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShaderLanguage {
    Glsl,
    Wgsl,
}

impl ShaderLanguage {
    pub fn from_path(path: &Path) -> Result<Self, ShaderError> {
        let extension = path
            .extension()
            .ok_or_else(|| ShaderError::NoExtension(path.to_path_buf()))?;
        let extension = extension.to_str().unwrap();
        match extension {
            "glsl" | "vert" | "frag" | "comp" => Ok(Self::Glsl),
            "wgsl" => Ok(Self::Wgsl),
            _ => Err(ShaderError::UnknownExtension(extension.to_string())),
        }
    }
}

#[derive(Clone)]
struct CachedShader {
    includes: HashSet<ShaderRef>,
    source: String,
}

impl CachedShader {
    fn parse_shader_ref(source: &mut &str) -> Result<ShaderRef, ShaderError> {
        *source = source.trim_start();
        if source.starts_with('"') {
            let path = source
                .strip_prefix('"')
                .and_then(|s| s.find('"'))
                .ok_or_else(|| ShaderError::InvalidInclude(source.to_string()))?;

            let path = &source[1..path + 1];
            *source = &source[path.len() + 2..];

            Ok(ShaderRef::Path(PathBuf::from(path.trim()).into()))
        } else if source.starts_with('<') {
            let path = source
                .strip_prefix('<')
                .and_then(|s| s.find('>'))
                .ok_or_else(|| ShaderError::InvalidInclude(source.to_string()))?;

            let path = &source[1..path + 1];
            *source = &source[path.len() + 2..];

            Ok(ShaderRef::module(path.trim().to_string()))
        } else {
            return Err(ShaderError::InvalidInclude(source.to_string()));
        }
    }

    fn parse(parent_path: Option<&Path>, mut source: &str) -> Result<Self, ShaderError> {
        let mut stripped_source = String::new();

        let mut includes = HashSet::new();

        while let Some(i) = source.find(INCLUDE_DIRECTIVE) {
            // skip multi-line comment
            if let Some(comment) = source.find("/*") {
                if comment < i {
                    stripped_source += &source[..comment];
                    source = &source[comment..];

                    let comment_end = source[comment..]
                        .find("*/")
                        .ok_or_else(|| ShaderError::UnclosedComment)?
                        + 2;

                    stripped_source += &source[..comment_end];
                    source = &source[comment_end..];

                    continue;
                }
            }

            // skip single-line comment
            if let Some(comment) = source.find("//") {
                if comment < i {
                    stripped_source += &source[..comment];
                    source = &source[comment..];

                    let comment_end = source.find('\n').unwrap_or(source.len()) + 1;

                    stripped_source += &source[..comment_end];
                    source = &source[comment_end..];

                    continue;
                }
            }

            // add include
            source = &source[i + INCLUDE_DIRECTIVE.len()..];

            let mut include = Self::parse_shader_ref(&mut source)?;
            if let Some(parent_path) = &parent_path {
                include = include.joined(parent_path);
            }

            includes.insert(include);
        }

        stripped_source += source;

        Ok(Self {
            source: stripped_source,
            includes,
        })
    }
}

pub struct ShaderProcessor {
    modules: HashMap<String, String>,
    cache: HashMap<ShaderRef, CachedShader>,
    io: Box<dyn ShaderIo>,
}

impl Default for ShaderProcessor {
    fn default() -> Self {
        Self::new(FsShaderIo)
    }
}

impl ShaderProcessor {
    pub fn empty(io: impl ShaderIo + 'static) -> Self {
        Self {
            modules: HashMap::new(),
            cache: HashMap::new(),
            io: Box::new(io),
        }
    }

    pub fn new(io: impl ShaderIo + 'static) -> Self {
        let mut this = Self::empty(io);
        this.add_default_modules();
        this
    }

    pub fn add_module(&mut self, name: String, source: String) {
        self.modules.insert(name, source);
    }

    pub fn add_default_modules(&mut self) {
        macro_rules! add_module {
            ($name:literal, $source:literal) => {
                self.add_module(
                    concat!("lumi/", $name).to_string(),
                    include_str!($source).to_string(),
                );
            };
        }

        add_module!("camera.wgsl", "wgsl/camera.wgsl");
        add_module!("mesh.wgsl", "wgsl/mesh.wgsl");
        add_module!("light.wgsl", "wgsl/light.wgsl");
        add_module!("fullscreen.wgsl", "wgsl/fullscreen.wgsl");
        add_module!("tonemapping.wgsl", "wgsl/tonemapping.wgsl");
        add_module!("pbr_material.wgsl", "wgsl/pbr_material.wgsl");
        add_module!("pbr.wgsl", "wgsl/pbr.wgsl");
        add_module!("environment.wgsl", "wgsl/environment.wgsl");
        add_module!("default_env_frag.wgsl", "wgsl/default_env_frag.wgsl");

        add_module!("pbr_light.wgsl", "wgsl/pbr_light.wgsl");
        add_module!("fullscreen_vert.wgsl", "wgsl/fullscreen_vert.wgsl");
        add_module!("bloom_frag.wgsl", "wgsl/bloom_frag.wgsl");
        add_module!("tonemapping_frag.wgsl", "wgsl/tonemapping_frag.wgsl");
    }

    fn read_shader_source(
        &self,
        shader_ref: &ShaderRef,
        _language: ShaderLanguage,
    ) -> Result<String, ShaderError> {
        match shader_ref {
            ShaderRef::Default(default) => match default {
                DefaultShader::Vertex => Ok(include_str!("wgsl/default_vert.wgsl").to_string()),
                DefaultShader::Fragment => Ok(include_str!("wgsl/default_frag.wgsl").to_string()),
            },
            ShaderRef::Path(path) => Ok(self.io.read(Path::new(path.as_ref()))?),
            ShaderRef::Module(module) => self
                .modules
                .get(module.as_ref())
                .cloned()
                .ok_or_else(|| ShaderError::InvalidModule(module.to_string())),
        }
    }

    fn get_cached_shader(
        &mut self,
        shader_ref: &ShaderRef,
        parent_path: Option<&Path>,
        language: ShaderLanguage,
    ) -> Result<&CachedShader, ShaderError> {
        if self.cache.contains_key(shader_ref) {
            Ok(self.cache.get(shader_ref).unwrap())
        } else {
            let source = self.read_shader_source(shader_ref, language)?;
            let shader = CachedShader::parse(parent_path, &source)?;
            self.cache.insert(shader_ref.clone(), shader);

            Ok(self.cache.get(shader_ref).unwrap())
        }
    }

    pub fn process(&mut self, shader_ref: ShaderRef) -> Result<Shader, ShaderError> {
        let mut processed = String::new();

        let shader_language = shader_ref.language()?;

        let mut stack: VecDeque<_> = vec![shader_ref].into();
        let mut included = Vec::new();
        let mut visited = HashSet::new();

        while let Some(shader_ref) = stack.pop_front() {
            if included.contains(&shader_ref) {
                continue;
            }

            if !visited.insert(shader_ref.clone()) {
                return Err(ShaderError::CircularInclude(shader_ref));
            }

            let shader = self.get_cached_shader(
                &shader_ref,
                shader_ref.parent_path().as_deref(),
                shader_language,
            )?;

            let mut can_include = true;
            for include in shader.includes.iter() {
                if !included.contains(include) {
                    if !stack.contains(include) {
                        stack.push_front(include.clone());
                    }

                    can_include = false;
                }
            }

            if !can_include {
                stack.push_back(shader_ref);
                continue;
            }

            visited.clear();

            processed += &shader.source;
            included.push(shader_ref);
        }

        Shader::new(&processed, shader_language)
    }
}
