use crate::sys;

macro_rules! reflection_enum {
    ($(#[$meta:meta])* $name:ident, $raw:ident, { $($variant:ident => $constant:ident),+ $(,)? }) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $($variant,)+
            /// A value from a Slang SDK newer than this wrapper.
            Unknown(sys::$raw),
        }

        impl $name {
            /// Converts a native value while preserving unrecognized values.
            pub const fn from_raw(value: sys::$raw) -> Self {
                match value {
                    $(sys::$constant => Self::$variant,)+
                    other => Self::Unknown(other),
                }
            }

            /// Returns the native Slang value.
            pub const fn as_raw(self) -> sys::$raw {
                match self {
                    $(Self::$variant => sys::$constant,)+
                    Self::Unknown(other) => other,
                }
            }
        }

        impl From<sys::$raw> for $name {
            fn from(value: sys::$raw) -> Self {
                Self::from_raw(value)
            }
        }

        impl From<$name> for sys::$raw {
            fn from(value: $name) -> Self {
                value.as_raw()
            }
        }
    };
}

reflection_enum!(
    /// A unit used by Slang layout queries. `Uniform` measures bytes.
    ParameterCategory, SlangParameterCategory, {
        None => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_NONE,
        Mixed => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_MIXED,
        ConstantBuffer => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_CONSTANT_BUFFER,
        ShaderResource => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_SHADER_RESOURCE,
        UnorderedAccess => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_UNORDERED_ACCESS,
        VaryingInput => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_VARYING_INPUT,
        VaryingOutput => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_VARYING_OUTPUT,
        SamplerState => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_SAMPLER_STATE,
        Uniform => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_UNIFORM,
        DescriptorTableSlot => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_DESCRIPTOR_TABLE_SLOT,
        SpecializationConstant => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_SPECIALIZATION_CONSTANT,
        PushConstantBuffer => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_PUSH_CONSTANT_BUFFER,
        RegisterSpace => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_REGISTER_SPACE,
        Generic => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_GENERIC,
        RayPayload => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_RAY_PAYLOAD,
        HitAttributes => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_HIT_ATTRIBUTES,
        CallablePayload => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_CALLABLE_PAYLOAD,
        ShaderRecord => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_SHADER_RECORD,
        ExistentialTypeParam => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_EXISTENTIAL_TYPE_PARAM,
        ExistentialObjectParam => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_EXISTENTIAL_OBJECT_PARAM,
        SubElementRegisterSpace => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_SUB_ELEMENT_REGISTER_SPACE,
        Subpass => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_SUBPASS,
        MetalArgumentBufferElement => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_METAL_ARGUMENT_BUFFER_ELEMENT,
        MetalAttribute => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_METAL_ATTRIBUTE,
        MetalPayload => SlangParameterCategory_SLANG_PARAMETER_CATEGORY_METAL_PAYLOAD,
    }
);

reflection_enum!(
    /// The shape of a reflected Slang type.
    TypeKind, SlangTypeKind, {
        None => SlangTypeKind_SLANG_TYPE_KIND_NONE,
        Struct => SlangTypeKind_SLANG_TYPE_KIND_STRUCT,
        Array => SlangTypeKind_SLANG_TYPE_KIND_ARRAY,
        Matrix => SlangTypeKind_SLANG_TYPE_KIND_MATRIX,
        Vector => SlangTypeKind_SLANG_TYPE_KIND_VECTOR,
        Scalar => SlangTypeKind_SLANG_TYPE_KIND_SCALAR,
        ConstantBuffer => SlangTypeKind_SLANG_TYPE_KIND_CONSTANT_BUFFER,
        Resource => SlangTypeKind_SLANG_TYPE_KIND_RESOURCE,
        SamplerState => SlangTypeKind_SLANG_TYPE_KIND_SAMPLER_STATE,
        TextureBuffer => SlangTypeKind_SLANG_TYPE_KIND_TEXTURE_BUFFER,
        ShaderStorageBuffer => SlangTypeKind_SLANG_TYPE_KIND_SHADER_STORAGE_BUFFER,
        ParameterBlock => SlangTypeKind_SLANG_TYPE_KIND_PARAMETER_BLOCK,
        GenericTypeParameter => SlangTypeKind_SLANG_TYPE_KIND_GENERIC_TYPE_PARAMETER,
        Interface => SlangTypeKind_SLANG_TYPE_KIND_INTERFACE,
        OutputStream => SlangTypeKind_SLANG_TYPE_KIND_OUTPUT_STREAM,
        MeshOutput => SlangTypeKind_SLANG_TYPE_KIND_MESH_OUTPUT,
        Specialized => SlangTypeKind_SLANG_TYPE_KIND_SPECIALIZED,
        Feedback => SlangTypeKind_SLANG_TYPE_KIND_FEEDBACK,
        Pointer => SlangTypeKind_SLANG_TYPE_KIND_POINTER,
        DynamicResource => SlangTypeKind_SLANG_TYPE_KIND_DYNAMIC_RESOURCE,
        Enum => SlangTypeKind_SLANG_TYPE_KIND_ENUM,
    }
);

reflection_enum!(
    /// The scalar component of a reflected type.
    ScalarType, SlangScalarType, {
        None => SlangScalarType_SLANG_SCALAR_TYPE_NONE,
        Void => SlangScalarType_SLANG_SCALAR_TYPE_VOID,
        Bool => SlangScalarType_SLANG_SCALAR_TYPE_BOOL,
        Int8 => SlangScalarType_SLANG_SCALAR_TYPE_INT8,
        UInt8 => SlangScalarType_SLANG_SCALAR_TYPE_UINT8,
        Int16 => SlangScalarType_SLANG_SCALAR_TYPE_INT16,
        UInt16 => SlangScalarType_SLANG_SCALAR_TYPE_UINT16,
        Int32 => SlangScalarType_SLANG_SCALAR_TYPE_INT32,
        UInt32 => SlangScalarType_SLANG_SCALAR_TYPE_UINT32,
        Int64 => SlangScalarType_SLANG_SCALAR_TYPE_INT64,
        UInt64 => SlangScalarType_SLANG_SCALAR_TYPE_UINT64,
        Float16 => SlangScalarType_SLANG_SCALAR_TYPE_FLOAT16,
        Float32 => SlangScalarType_SLANG_SCALAR_TYPE_FLOAT32,
        Float64 => SlangScalarType_SLANG_SCALAR_TYPE_FLOAT64,
        IntPtr => SlangScalarType_SLANG_SCALAR_TYPE_INTPTR,
        UIntPtr => SlangScalarType_SLANG_SCALAR_TYPE_UINTPTR,
        BFloat16 => SlangScalarType_SLANG_SCALAR_TYPE_BFLOAT16,
        FloatE4M3 => SlangScalarType_SLANG_SCALAR_TYPE_FLOAT_E4M3,
        FloatE5M2 => SlangScalarType_SLANG_SCALAR_TYPE_FLOAT_E5M2,
    }
);
