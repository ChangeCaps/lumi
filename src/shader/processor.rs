use std::{
    borrow::Cow,
    collections::VecDeque,
    path::{Path, PathBuf},
};

use smallvec::SmallVec;

use crate::{
    shader::{DefaultShader, ShaderError, ShaderRef},
    util::{HashMap, HashSet},
};

use super::{FsShaderIo, Shader, ShaderIo};

const INCLUDE_DIRECTIVE: &str = "#include";
const IFDEF_DIRECTIVE: &str = "#ifdef";
const IFNDEF_DIRECTIVE: &str = "#ifndef";
const ENDIF_DIRECTIVE: &str = "#endif";

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

    fn strip_comments(mut source: &str) -> Result<String, ShaderError> {
        let mut result = String::with_capacity(source.len());

        while !source.is_empty() {
            if let Some(index) = source.find("//") {
                result.push_str(&source[..index]);
                source = &source[index + 2..];

                if let Some(index) = source.find('\n') {
                    source = &source[index + 1..];
                } else {
                    break;
                }

                continue;
            }

            if let Some(index) = source.find("/*") {
                result.push_str(&source[..index]);
                source = &source[index + 2..];

                if let Some(index) = source.find("*/") {
                    source = &source[index + 2..];
                } else {
                    return Err(ShaderError::UnclosedComment);
                }

                continue;
            }

            break;
        }

        result.push_str(source);

        Ok(result)
    }

    fn find_def_directive(source: &str) -> Option<(usize, bool)> {
        let a = source.find(IFDEF_DIRECTIVE);
        let b = source.find(IFNDEF_DIRECTIVE);

        match (a, b) {
            (Some(a), Some(b)) => Some(if a < b { (a, true) } else { (b, false) }),
            (Some(a), None) => Some((a, true)),
            (None, Some(b)) => Some((b, false)),
            (None, None) => None,
        }
    }

    fn def_len(is_def: bool) -> usize {
        if is_def {
            IFDEF_DIRECTIVE.len()
        } else {
            IFNDEF_DIRECTIVE.len()
        }
    }

    fn find_end(mut source: &str) -> Option<usize> {
        let mut directives = 0;

        loop {
            let end = source.find(ENDIF_DIRECTIVE)?;

            if let Some((index, is_def)) = Self::find_def_directive(&source[..end]) {
                directives += 1;
                source = &source[index + Self::def_len(is_def)..];
            } else {
                if directives == 0 {
                    return Some(end);
                } else {
                    directives -= 1;
                    source = &source[end + ENDIF_DIRECTIVE.len()..];
                }
            }
        }
    }

    fn parse(
        parent_path: Option<&Path>,
        source: &str,
        defs: &ShaderDefs,
    ) -> Result<Self, ShaderError> {
        let source = Self::strip_comments(source)?;
        let mut source = source.as_str();
        let mut stripped_source = String::new();

        loop {
            if let Some((i, is_def)) = Self::find_def_directive(source) {
                stripped_source.push_str(&source[..i]);

                source = &source[i + Self::def_len(is_def)..];
                source = source.trim_start();

                let end = source
                    .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .unwrap_or_else(|| source.len());

                let def = &source[..end];
                source = &source[end..];

                if def.is_empty() {
                    return Err(ShaderError::InvalidDefine(source.to_string()));
                }

                let end = Self::find_end(source).ok_or_else(|| ShaderError::UnclosedDirective)?;

                if defs.contains(def) == is_def {
                    stripped_source.push_str(&source[..end]);
                }

                source = &source[end + ENDIF_DIRECTIVE.len()..];
            } else {
                stripped_source.push_str(source);
                break;
            }
        }

        let mut source = stripped_source.as_str();
        let mut stripped_source = String::new();

        let mut includes = HashSet::default();

        loop {
            if let Some(i) = source.find(INCLUDE_DIRECTIVE) {
                stripped_source.push_str(&source[..i]);

                // add include
                source = &source[i + INCLUDE_DIRECTIVE.len()..];

                let mut include = Self::parse_shader_ref(&mut source)?;
                if let Some(parent_path) = &parent_path {
                    include = include.joined(parent_path);
                }

                includes.insert(include);
            } else {
                stripped_source.push_str(source);
                break;
            }
        }
        Ok(Self {
            source: stripped_source,
            includes,
        })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderDefs {
    defs: SmallVec<[Cow<'static, str>; 8]>,
}

impl ShaderDefs {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn set(&mut self, def: impl Into<Cow<'static, str>>) {
        self.defs.push(def.into())
    }

    #[inline]
    pub fn contains(&self, def: &str) -> bool {
        self.defs.contains(&Cow::Borrowed(def))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct ShaderCacheKey {
    shader_ref: ShaderRef,
    defs: ShaderDefs,
}

pub struct ShaderProcessor {
    modules: HashMap<String, String>,
    cache: HashMap<ShaderCacheKey, CachedShader>,
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
            modules: HashMap::default(),
            cache: HashMap::default(),
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
                    #[cfg(not(feature = "load-shaders"))]
                    include_str!(concat!("../../shaders/", $source)).to_string(),
                    #[cfg(feature = "load-shaders")]
                    self.io.read(concat!("shaders/", $source).as_ref()).unwrap(),
                );
            };
        }

        add_module!("camera.wgsl", "wgsl/camera.wgsl");
        add_module!("mesh.wgsl", "wgsl/mesh.wgsl");
        add_module!("light.wgsl", "wgsl/light.wgsl");
        add_module!("fullscreen.wgsl", "wgsl/fullscreen.wgsl");
        add_module!("tonemapping.wgsl", "wgsl/tonemapping.wgsl");
        add_module!("standard_material.wgsl", "wgsl/standard_material.wgsl");
        add_module!("integrated_brdf.wgsl", "wgsl/integrated_brdf.wgsl");
        add_module!("pbr_types.wgsl", "wgsl/pbr_types.wgsl");
        add_module!("pbr.wgsl", "wgsl/pbr.wgsl");
        add_module!("ssr.wgsl", "wgsl/ssr.wgsl");
        add_module!("environment.wgsl", "wgsl/environment.wgsl");
        add_module!("pbr_light.wgsl", "wgsl/pbr_light.wgsl");
        add_module!("sky.wgsl", "wgsl/sky.wgsl");
        add_module!("poisson.wgsl", "wgsl/poisson.wgsl");
        add_module!("shadow.wgsl", "wgsl/shadow.wgsl");
        add_module!("shadow_mesh.wgsl", "wgsl/shadow_mesh.wgsl");

        add_module!("gaussian.wgsl", "wgsl/gaussian.wgsl");
        add_module!("unlit.wgsl", "wgsl/unlit.wgsl");
        add_module!("sky_vert.wgsl", "wgsl/sky_vert.wgsl");
        add_module!("fullscreen_vert.wgsl", "wgsl/fullscreen_vert.wgsl");
        add_module!("bloom_frag.wgsl", "wgsl/bloom_frag.wgsl");
        add_module!("tonemapping_frag.wgsl", "wgsl/tonemapping_frag.wgsl");
        add_module!("standard_frag.wgsl", "wgsl/standard_frag.wgsl");
    }

    fn read_shader_source(
        &self,
        shader_ref: &ShaderRef,
        _language: ShaderLanguage,
    ) -> Result<String, ShaderError> {
        match shader_ref {
            ShaderRef::Default(default) => match default {
                DefaultShader::Vertex => {
                    Ok(include_str!("../../shaders/wgsl/default_vert.wgsl").to_string())
                }
                DefaultShader::Fragment => {
                    Ok(include_str!("../../shaders/wgsl/default_frag.wgsl").to_string())
                }
                DefaultShader::ShadowVertex => {
                    Ok(include_str!("../../shaders/wgsl/default_shadow_vert.wgsl").to_string())
                }
                DefaultShader::ShadowFragment => {
                    unimplemented!()
                }
                DefaultShader::Sky => {
                    Ok(include_str!("../../shaders/wgsl/default_sky.wgsl").to_string())
                }
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
        key: &ShaderCacheKey,
        parent_path: Option<&Path>,
        language: ShaderLanguage,
    ) -> Result<&CachedShader, ShaderError> {
        if self.cache.contains_key(key) {
            Ok(self.cache.get(key).unwrap())
        } else {
            let source = self.read_shader_source(&key.shader_ref, language)?;
            let shader = CachedShader::parse(parent_path, &source, &key.defs)?;
            self.cache.insert(key.clone(), shader);

            Ok(self.cache.get(key).unwrap())
        }
    }

    pub fn process(
        &mut self,
        shader_ref: ShaderRef,
        defs: &ShaderDefs,
    ) -> Result<Shader, ShaderError> {
        let mut processed = String::new();

        let shader_language = shader_ref.language()?;

        let mut stack: VecDeque<_> = vec![shader_ref].into();
        let mut included = Vec::new();
        let mut visited = HashSet::default();

        while let Some(shader_ref) = stack.pop_front() {
            if included.contains(&shader_ref) {
                continue;
            }

            if !visited.insert(shader_ref.clone()) {
                return Err(ShaderError::CircularInclude(shader_ref));
            }

            let key = ShaderCacheKey {
                shader_ref: shader_ref.clone(),
                defs: defs.clone(),
            };

            let shader =
                self.get_cached_shader(&key, shader_ref.parent_path().as_deref(), shader_language)?;

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
