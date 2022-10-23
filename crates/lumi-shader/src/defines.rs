use lumi_util::smallvec::SmallVec;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShaderDefHash {
    hash: u64,
}

impl ShaderDefHash {
    pub const ZERO: Self = Self { hash: 0 };

    /// A cheap compile-time hash of the shader definition.
    #[inline(always)]
    pub const fn from_str(s: &str) -> Self {
        let mut hash = 0u64;

        let mut i = 0;
        while i < s.len() {
            let c = s.as_bytes()[i];
            hash = hash.wrapping_mul(31).wrapping_add(c as u64);
            i += 1;
        }

        Self { hash }
    }
}

impl Default for ShaderDefHash {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<&str> for ShaderDefHash {
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShaderDefsHash {
    hash: ShaderDefHash,
}

impl ShaderDefsHash {
    pub const ZERO: Self = Self {
        hash: ShaderDefHash::ZERO,
    };

    #[inline(always)]
    pub fn add(&mut self, hash: ShaderDefHash) {
        self.hash.hash = self.hash.hash.wrapping_mul(31).wrapping_add(hash.hash);
    }
}

impl Default for ShaderDefsHash {
    #[inline]
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderDefs {
    defs: SmallVec<[ShaderDefHash; 16]>,
}

impl ShaderDefs {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn push(&mut self, def: impl Into<ShaderDefHash>) {
        self.defs.push(def.into())
    }

    #[inline]
    pub fn contains(&self, def: &ShaderDefHash) -> bool {
        self.defs.contains(def)
    }

    #[inline]
    pub fn finish(&mut self) {
        self.defs.dedup();
        self.defs.sort_unstable();
    }

    #[inline]
    pub fn hash(&self) -> ShaderDefsHash {
        let mut hash = ShaderDefsHash::default();

        for def in &self.defs {
            hash.add(*def);
        }

        hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shader_def_hash() {
        macro_rules! hashes {
            ($($lit:literal,)*) => {
                &[$(ShaderDefHash::from_str($lit).hash,)*]
            };
        }

        let hashes = hashes! {
            "FOO",
            "BAR",
            "BAZ",
            "CLEARCOAT",
            "CLEARCOAT_NORMAL",
            "CLEARCOAT_ROUGHNESS",
            "NORMAL_MAP",
            "TRANSMISSION",
            "THICKNESS",
            "EMISSIVE_MAP",
            "METALLIC_ROUGHNESS_TEXTURE",
            "OCCLUSION_MAP",
            "BASE_COLOR_TEXTURE",
            "SUBSURFACE",
            "THICKNESS_TEXTURE",
            "EMISSIVE",
            "METALLIC_ROUGHNESS",
            "OCCLUSION",
            "BASE_COLOR",
            "SUBSURFACE_TEXTURE",
            "CLEARCOAT_TEXTURE",
            "CLEARCOAT_ROUGHNESS_TEXTURE",
            "NORMAL_TEXTURE",
            "TRANSMISSION_TEXTURE",
            "EMISSIVE_TEXTURE",
            "METALLIC_ROUGHNESS_MAP",
            "OCCLUSION_TEXTURE",
            "BASE_COLOR_MAP",
            "SUBSURFACE_MAP",
            "THICKNESS_MAP",
            "EMISSIVE_MAP",
        };

        for (i, hash) in hashes.iter().enumerate() {
            for (j, other_hash) in hashes.iter().enumerate() {
                if i == j {
                    assert_eq!(hash, other_hash);
                } else {
                    assert_ne!(hash, other_hash);
                }
            }
        }
    }

    #[test]
    fn test_shader_defs() {}
}
