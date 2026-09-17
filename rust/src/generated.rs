pub use root::*;

const _: () = ::planus::check_version_compatibility("planus-1.3.0");

/// The root namespace
///
/// Generated from these locations:
/// * File `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs`
#[no_implicit_prelude]
#[allow(clippy::needless_lifetimes)]
mod root {
    /// The namespace `ModAbi`
    ///
    /// Generated from these locations:
    /// * File `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs`
    pub mod mod_abi {
        /// The enum `Encoding` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Enum `Encoding` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:47`
        #[derive(
            Copy,
            Clone,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #[repr(u8)]
        pub enum Encoding {
            /// The variant `Json` in the enum `Encoding`
            Json = 0,

            /// The variant `Typed` in the enum `Encoding`
            Typed = 1,
        }

        impl Encoding {
            /// Array containing all valid variants of Encoding
            pub const ENUM_VALUES: [Self; 2] = [Self::Json, Self::Typed];
        }

        impl ::core::convert::TryFrom<u8> for Encoding {
            type Error = ::planus::errors::UnknownEnumTagKind;
            #[inline]
            fn try_from(
                value: u8,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTagKind> {
                #[allow(clippy::match_single_binding)]
                match value {
                    0 => ::core::result::Result::Ok(Encoding::Json),
                    1 => ::core::result::Result::Ok(Encoding::Typed),

                    _ => ::core::result::Result::Err(::planus::errors::UnknownEnumTagKind {
                        tag: value as i128,
                    }),
                }
            }
        }

        impl ::core::convert::From<Encoding> for u8 {
            #[inline]
            fn from(value: Encoding) -> Self {
                value as u8
            }
        }

        /// # Safety
        /// The Planus compiler correctly calculates `ALIGNMENT` and `SIZE`.
        unsafe impl ::planus::Primitive for Encoding {
            const ALIGNMENT: usize = 1;
            const SIZE: usize = 1;
        }

        impl ::planus::WriteAsPrimitive<Encoding> for Encoding {
            #[inline]
            fn write<const N: usize>(&self, cursor: ::planus::Cursor<'_, N>, buffer_position: u32) {
                (*self as u8).write(cursor, buffer_position);
            }
        }

        impl ::planus::WriteAs<Encoding> for Encoding {
            type Prepared = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> Encoding {
                *self
            }
        }

        impl ::planus::WriteAsDefault<Encoding, Encoding> for Encoding {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
                default: &Encoding,
            ) -> ::core::option::Option<Encoding> {
                if self == default {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(*self)
                }
            }
        }

        impl ::planus::WriteAsOptional<Encoding> for Encoding {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<Encoding> {
                ::core::option::Option::Some(*self)
            }
        }

        impl<'buf> ::planus::TableRead<'buf> for Encoding {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                let n: u8 = ::planus::TableRead::from_buffer(buffer, offset)?;
                ::core::result::Result::Ok(::core::convert::TryInto::try_into(n)?)
            }
        }

        impl<'buf> ::planus::VectorReadInner<'buf> for Encoding {
            type Error = ::planus::errors::UnknownEnumTag;
            const STRIDE: usize = 1;
            #[inline]
            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTag> {
                let value = unsafe { *buffer.buffer.get_unchecked(offset) };
                let value: ::core::result::Result<Self, _> =
                    ::core::convert::TryInto::try_into(value);
                value.map_err(|error_kind| {
                    error_kind.with_error_location(
                        "Encoding",
                        "VectorRead::from_buffer",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<Encoding> for Encoding {
            const STRIDE: usize = 1;

            type Value = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> Self {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[Self],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 1];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - i as u32,
                    );
                }
            }
        }

        /// The table `CompValue` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `CompValue` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:53`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct CompValue {
            /// The field `type_id` in the table `CompValue`
            pub type_id: u16,
            /// The field `encoding` in the table `CompValue`
            pub encoding: self::Encoding,
            /// The field `data` in the table `CompValue`
            pub data: ::core::option::Option<::planus::alloc::vec::Vec<u8>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for CompValue {
            fn default() -> Self {
                Self {
                    type_id: 0,
                    encoding: self::Encoding::Json,
                    data: ::core::default::Default::default(),
                }
            }
        }

        impl CompValue {
            /// Creates a [CompValueBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> CompValueBuilder<()> {
                CompValueBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_type_id: impl ::planus::WriteAsDefault<u16, u16>,
                field_encoding: impl ::planus::WriteAsDefault<self::Encoding, self::Encoding>,
                field_data: impl ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            ) -> ::planus::Offset<Self> {
                let prepared_type_id = field_type_id.prepare(builder, &0);
                let prepared_encoding = field_encoding.prepare(builder, &self::Encoding::Json);
                let prepared_data = field_data.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<10> =
                    ::core::default::Default::default();
                if prepared_data.is_some() {
                    table_writer.write_entry::<::planus::Offset<[u8]>>(2);
                }
                if prepared_type_id.is_some() {
                    table_writer.write_entry::<u16>(0);
                }
                if prepared_encoding.is_some() {
                    table_writer.write_entry::<self::Encoding>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_data) = prepared_data {
                            object_writer.write::<_, _, 4>(&prepared_data);
                        }
                        if let ::core::option::Option::Some(prepared_type_id) = prepared_type_id {
                            object_writer.write::<_, _, 2>(&prepared_type_id);
                        }
                        if let ::core::option::Option::Some(prepared_encoding) = prepared_encoding {
                            object_writer.write::<_, _, 1>(&prepared_encoding);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<CompValue>> for CompValue {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<CompValue> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<CompValue>> for CompValue {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<CompValue>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<CompValue> for CompValue {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<CompValue> {
                CompValue::create(builder, self.type_id, self.encoding, &self.data)
            }
        }

        /// Builder for serializing an instance of the [CompValue] type.
        ///
        /// Can be created using the [CompValue::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct CompValueBuilder<State>(State);

        impl CompValueBuilder<()> {
            /// Setter for the [`type_id` field](CompValue#structfield.type_id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_id<T0>(self, value: T0) -> CompValueBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u16, u16>,
            {
                CompValueBuilder((value,))
            }

            /// Sets the [`type_id` field](CompValue#structfield.type_id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_id_as_default(self) -> CompValueBuilder<(::planus::DefaultValue,)> {
                self.type_id(::planus::DefaultValue)
            }
        }

        impl<T0> CompValueBuilder<(T0,)> {
            /// Setter for the [`encoding` field](CompValue#structfield.encoding).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn encoding<T1>(self, value: T1) -> CompValueBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsDefault<self::Encoding, self::Encoding>,
            {
                let (v0,) = self.0;
                CompValueBuilder((v0, value))
            }

            /// Sets the [`encoding` field](CompValue#structfield.encoding) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn encoding_as_default(self) -> CompValueBuilder<(T0, ::planus::DefaultValue)> {
                self.encoding(::planus::DefaultValue)
            }
        }

        impl<T0, T1> CompValueBuilder<(T0, T1)> {
            /// Setter for the [`data` field](CompValue#structfield.data).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn data<T2>(self, value: T2) -> CompValueBuilder<(T0, T1, T2)>
            where
                T2: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            {
                let (v0, v1) = self.0;
                CompValueBuilder((v0, v1, value))
            }

            /// Sets the [`data` field](CompValue#structfield.data) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn data_as_null(self) -> CompValueBuilder<(T0, T1, ())> {
                self.data(())
            }
        }

        impl<T0, T1, T2> CompValueBuilder<(T0, T1, T2)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [CompValue].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<CompValue>
            where
                Self: ::planus::WriteAsOffset<CompValue>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u16, u16>,
                T1: ::planus::WriteAsDefault<self::Encoding, self::Encoding>,
                T2: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            > ::planus::WriteAs<::planus::Offset<CompValue>> for CompValueBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<CompValue>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<CompValue> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u16, u16>,
                T1: ::planus::WriteAsDefault<self::Encoding, self::Encoding>,
                T2: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            > ::planus::WriteAsOptional<::planus::Offset<CompValue>>
            for CompValueBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<CompValue>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<CompValue>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u16, u16>,
                T1: ::planus::WriteAsDefault<self::Encoding, self::Encoding>,
                T2: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            > ::planus::WriteAsOffset<CompValue> for CompValueBuilder<(T0, T1, T2)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<CompValue> {
                let (v0, v1, v2) = &self.0;
                CompValue::create(builder, v0, v1, v2)
            }
        }

        /// Reference to a deserialized [CompValue].
        #[derive(Copy, Clone)]
        pub struct CompValueRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> CompValueRef<'a> {
            /// Getter for the [`type_id` field](CompValue#structfield.type_id).
            #[inline]
            pub fn type_id(&self) -> ::planus::Result<u16> {
                ::core::result::Result::Ok(self.0.access(0, "CompValue", "type_id")?.unwrap_or(0))
            }

            /// Getter for the [`encoding` field](CompValue#structfield.encoding).
            #[inline]
            pub fn encoding(&self) -> ::planus::Result<self::Encoding> {
                ::core::result::Result::Ok(
                    self.0
                        .access(1, "CompValue", "encoding")?
                        .unwrap_or(self::Encoding::Json),
                )
            }

            /// Getter for the [`data` field](CompValue#structfield.data).
            #[inline]
            pub fn data(&self) -> ::planus::Result<::core::option::Option<&'a [u8]>> {
                self.0.access(2, "CompValue", "data")
            }
        }

        impl<'a> ::core::fmt::Debug for CompValueRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("CompValueRef");
                f.field("type_id", &self.type_id());
                f.field("encoding", &self.encoding());
                if let ::core::option::Option::Some(field_data) = self.data().transpose() {
                    f.field("data", &field_data);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<CompValueRef<'a>> for CompValue {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: CompValueRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    type_id: ::core::convert::TryInto::try_into(value.type_id()?)?,
                    encoding: ::core::convert::TryInto::try_into(value.encoding()?)?,
                    data: value.data()?.map(|v| v.to_vec()),
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for CompValueRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for CompValueRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[CompValueRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<CompValue>> for CompValue {
            type Value = ::planus::Offset<CompValue>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<CompValue>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for CompValueRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[CompValueRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `TypePath` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `TypePath` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:62`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct TypePath {
            /// The field `id` in the table `TypePath`
            pub id: u16,
            /// The field `path` in the table `TypePath`
            pub path: ::core::option::Option<::planus::alloc::string::String>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for TypePath {
            fn default() -> Self {
                Self {
                    id: 0,
                    path: ::core::default::Default::default(),
                }
            }
        }

        impl TypePath {
            /// Creates a [TypePathBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> TypePathBuilder<()> {
                TypePathBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_id: impl ::planus::WriteAsDefault<u16, u16>,
                field_path: impl ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            ) -> ::planus::Offset<Self> {
                let prepared_id = field_id.prepare(builder, &0);
                let prepared_path = field_path.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_path.is_some() {
                    table_writer.write_entry::<::planus::Offset<str>>(1);
                }
                if prepared_id.is_some() {
                    table_writer.write_entry::<u16>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_path) = prepared_path {
                            object_writer.write::<_, _, 4>(&prepared_path);
                        }
                        if let ::core::option::Option::Some(prepared_id) = prepared_id {
                            object_writer.write::<_, _, 2>(&prepared_id);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<TypePath>> for TypePath {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<TypePath> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<TypePath>> for TypePath {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<TypePath>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<TypePath> for TypePath {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<TypePath> {
                TypePath::create(builder, self.id, &self.path)
            }
        }

        /// Builder for serializing an instance of the [TypePath] type.
        ///
        /// Can be created using the [TypePath::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct TypePathBuilder<State>(State);

        impl TypePathBuilder<()> {
            /// Setter for the [`id` field](TypePath#structfield.id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn id<T0>(self, value: T0) -> TypePathBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u16, u16>,
            {
                TypePathBuilder((value,))
            }

            /// Sets the [`id` field](TypePath#structfield.id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn id_as_default(self) -> TypePathBuilder<(::planus::DefaultValue,)> {
                self.id(::planus::DefaultValue)
            }
        }

        impl<T0> TypePathBuilder<(T0,)> {
            /// Setter for the [`path` field](TypePath#structfield.path).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn path<T1>(self, value: T1) -> TypePathBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            {
                let (v0,) = self.0;
                TypePathBuilder((v0, value))
            }

            /// Sets the [`path` field](TypePath#structfield.path) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn path_as_null(self) -> TypePathBuilder<(T0, ())> {
                self.path(())
            }
        }

        impl<T0, T1> TypePathBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [TypePath].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<TypePath>
            where
                Self: ::planus::WriteAsOffset<TypePath>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u16, u16>,
                T1: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            > ::planus::WriteAs<::planus::Offset<TypePath>> for TypePathBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<TypePath>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<TypePath> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u16, u16>,
                T1: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            > ::planus::WriteAsOptional<::planus::Offset<TypePath>> for TypePathBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<TypePath>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<TypePath>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u16, u16>,
                T1: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            > ::planus::WriteAsOffset<TypePath> for TypePathBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<TypePath> {
                let (v0, v1) = &self.0;
                TypePath::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [TypePath].
        #[derive(Copy, Clone)]
        pub struct TypePathRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> TypePathRef<'a> {
            /// Getter for the [`id` field](TypePath#structfield.id).
            #[inline]
            pub fn id(&self) -> ::planus::Result<u16> {
                ::core::result::Result::Ok(self.0.access(0, "TypePath", "id")?.unwrap_or(0))
            }

            /// Getter for the [`path` field](TypePath#structfield.path).
            #[inline]
            pub fn path(
                &self,
            ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>> {
                self.0.access(1, "TypePath", "path")
            }
        }

        impl<'a> ::core::fmt::Debug for TypePathRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("TypePathRef");
                f.field("id", &self.id());
                if let ::core::option::Option::Some(field_path) = self.path().transpose() {
                    f.field("path", &field_path);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<TypePathRef<'a>> for TypePath {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: TypePathRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    id: ::core::convert::TryInto::try_into(value.id()?)?,
                    path: value.path()?.map(::core::convert::Into::into),
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for TypePathRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for TypePathRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location("[TypePathRef]", "get", buffer.offset_from_start)
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<TypePath>> for TypePath {
            type Value = ::planus::Offset<TypePath>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<TypePath>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for TypePathRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[TypePathRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `Handshake` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `Handshake` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:67`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct Handshake {
            /// The field `abi_version` in the table `Handshake`
            pub abi_version: u32,
            /// The field `type_paths` in the table `Handshake`
            pub type_paths: ::core::option::Option<::planus::alloc::vec::Vec<self::TypePath>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for Handshake {
            fn default() -> Self {
                Self {
                    abi_version: 0,
                    type_paths: ::core::default::Default::default(),
                }
            }
        }

        impl Handshake {
            /// Creates a [HandshakeBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> HandshakeBuilder<()> {
                HandshakeBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_abi_version: impl ::planus::WriteAsDefault<u32, u32>,
                field_type_paths: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::TypePath>]>,
                >,
            ) -> ::planus::Offset<Self> {
                let prepared_abi_version = field_abi_version.prepare(builder, &0);
                let prepared_type_paths = field_type_paths.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_abi_version.is_some() {
                    table_writer.write_entry::<u32>(0);
                }
                if prepared_type_paths.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::TypePath>]>>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_abi_version) =
                            prepared_abi_version
                        {
                            object_writer.write::<_, _, 4>(&prepared_abi_version);
                        }
                        if let ::core::option::Option::Some(prepared_type_paths) =
                            prepared_type_paths
                        {
                            object_writer.write::<_, _, 4>(&prepared_type_paths);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<Handshake>> for Handshake {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Handshake> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<Handshake>> for Handshake {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<Handshake>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<Handshake> for Handshake {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Handshake> {
                Handshake::create(builder, self.abi_version, &self.type_paths)
            }
        }

        /// Builder for serializing an instance of the [Handshake] type.
        ///
        /// Can be created using the [Handshake::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct HandshakeBuilder<State>(State);

        impl HandshakeBuilder<()> {
            /// Setter for the [`abi_version` field](Handshake#structfield.abi_version).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn abi_version<T0>(self, value: T0) -> HandshakeBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                HandshakeBuilder((value,))
            }

            /// Sets the [`abi_version` field](Handshake#structfield.abi_version) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn abi_version_as_default(self) -> HandshakeBuilder<(::planus::DefaultValue,)> {
                self.abi_version(::planus::DefaultValue)
            }
        }

        impl<T0> HandshakeBuilder<(T0,)> {
            /// Setter for the [`type_paths` field](Handshake#structfield.type_paths).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_paths<T1>(self, value: T1) -> HandshakeBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::TypePath>]>>,
            {
                let (v0,) = self.0;
                HandshakeBuilder((v0, value))
            }

            /// Sets the [`type_paths` field](Handshake#structfield.type_paths) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_paths_as_null(self) -> HandshakeBuilder<(T0, ())> {
                self.type_paths(())
            }
        }

        impl<T0, T1> HandshakeBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [Handshake].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<Handshake>
            where
                Self: ::planus::WriteAsOffset<Handshake>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::TypePath>]>>,
            > ::planus::WriteAs<::planus::Offset<Handshake>> for HandshakeBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<Handshake>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Handshake> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::TypePath>]>>,
            > ::planus::WriteAsOptional<::planus::Offset<Handshake>>
            for HandshakeBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<Handshake>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<Handshake>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::TypePath>]>>,
            > ::planus::WriteAsOffset<Handshake> for HandshakeBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Handshake> {
                let (v0, v1) = &self.0;
                Handshake::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [Handshake].
        #[derive(Copy, Clone)]
        pub struct HandshakeRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> HandshakeRef<'a> {
            /// Getter for the [`abi_version` field](Handshake#structfield.abi_version).
            #[inline]
            pub fn abi_version(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(
                    self.0.access(0, "Handshake", "abi_version")?.unwrap_or(0),
                )
            }

            /// Getter for the [`type_paths` field](Handshake#structfield.type_paths).
            #[inline]
            pub fn type_paths(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::TypePathRef<'a>>>,
                >,
            > {
                self.0.access(1, "Handshake", "type_paths")
            }
        }

        impl<'a> ::core::fmt::Debug for HandshakeRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("HandshakeRef");
                f.field("abi_version", &self.abi_version());
                if let ::core::option::Option::Some(field_type_paths) =
                    self.type_paths().transpose()
                {
                    f.field("type_paths", &field_type_paths);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<HandshakeRef<'a>> for Handshake {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: HandshakeRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    abi_version: ::core::convert::TryInto::try_into(value.abi_version()?)?,
                    type_paths: if let ::core::option::Option::Some(type_paths) =
                        value.type_paths()?
                    {
                        ::core::option::Option::Some(type_paths.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for HandshakeRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for HandshakeRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[HandshakeRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<Handshake>> for Handshake {
            type Value = ::planus::Offset<Handshake>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<Handshake>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for HandshakeRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[HandshakeRef]", "read_as_root", 0)
                })
            }
        }

        /// The enum `Schedule` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Enum `Schedule` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:75`
        #[derive(
            Copy,
            Clone,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #[repr(u8)]
        pub enum Schedule {
            /// The variant `ModStartup` in the enum `Schedule`
            ModStartup = 0,

            /// The variant `First` in the enum `Schedule`
            First = 1,

            /// The variant `PreUpdate` in the enum `Schedule`
            PreUpdate = 2,

            /// The variant `Update` in the enum `Schedule`
            Update = 3,

            /// The variant `PostUpdate` in the enum `Schedule`
            PostUpdate = 4,

            /// The variant `Last` in the enum `Schedule`
            Last = 5,

            /// The variant `Custom` in the enum `Schedule`
            Custom = 6,
        }

        impl Schedule {
            /// Array containing all valid variants of Schedule
            pub const ENUM_VALUES: [Self; 7] = [
                Self::ModStartup,
                Self::First,
                Self::PreUpdate,
                Self::Update,
                Self::PostUpdate,
                Self::Last,
                Self::Custom,
            ];
        }

        impl ::core::convert::TryFrom<u8> for Schedule {
            type Error = ::planus::errors::UnknownEnumTagKind;
            #[inline]
            fn try_from(
                value: u8,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTagKind> {
                #[allow(clippy::match_single_binding)]
                match value {
                    0 => ::core::result::Result::Ok(Schedule::ModStartup),
                    1 => ::core::result::Result::Ok(Schedule::First),
                    2 => ::core::result::Result::Ok(Schedule::PreUpdate),
                    3 => ::core::result::Result::Ok(Schedule::Update),
                    4 => ::core::result::Result::Ok(Schedule::PostUpdate),
                    5 => ::core::result::Result::Ok(Schedule::Last),
                    6 => ::core::result::Result::Ok(Schedule::Custom),

                    _ => ::core::result::Result::Err(::planus::errors::UnknownEnumTagKind {
                        tag: value as i128,
                    }),
                }
            }
        }

        impl ::core::convert::From<Schedule> for u8 {
            #[inline]
            fn from(value: Schedule) -> Self {
                value as u8
            }
        }

        /// # Safety
        /// The Planus compiler correctly calculates `ALIGNMENT` and `SIZE`.
        unsafe impl ::planus::Primitive for Schedule {
            const ALIGNMENT: usize = 1;
            const SIZE: usize = 1;
        }

        impl ::planus::WriteAsPrimitive<Schedule> for Schedule {
            #[inline]
            fn write<const N: usize>(&self, cursor: ::planus::Cursor<'_, N>, buffer_position: u32) {
                (*self as u8).write(cursor, buffer_position);
            }
        }

        impl ::planus::WriteAs<Schedule> for Schedule {
            type Prepared = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> Schedule {
                *self
            }
        }

        impl ::planus::WriteAsDefault<Schedule, Schedule> for Schedule {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
                default: &Schedule,
            ) -> ::core::option::Option<Schedule> {
                if self == default {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(*self)
                }
            }
        }

        impl ::planus::WriteAsOptional<Schedule> for Schedule {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<Schedule> {
                ::core::option::Option::Some(*self)
            }
        }

        impl<'buf> ::planus::TableRead<'buf> for Schedule {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                let n: u8 = ::planus::TableRead::from_buffer(buffer, offset)?;
                ::core::result::Result::Ok(::core::convert::TryInto::try_into(n)?)
            }
        }

        impl<'buf> ::planus::VectorReadInner<'buf> for Schedule {
            type Error = ::planus::errors::UnknownEnumTag;
            const STRIDE: usize = 1;
            #[inline]
            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTag> {
                let value = unsafe { *buffer.buffer.get_unchecked(offset) };
                let value: ::core::result::Result<Self, _> =
                    ::core::convert::TryInto::try_into(value);
                value.map_err(|error_kind| {
                    error_kind.with_error_location(
                        "Schedule",
                        "VectorRead::from_buffer",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<Schedule> for Schedule {
            const STRIDE: usize = 1;

            type Value = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> Self {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[Self],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 1];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - i as u32,
                    );
                }
            }
        }

        /// The enum `QueryTermKind` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Enum `QueryTermKind` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:85`
        #[derive(
            Copy,
            Clone,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #[repr(u8)]
        pub enum QueryTermKind {
            /// The variant `Ref` in the enum `QueryTermKind`
            Ref = 0,

            /// The variant `Mut` in the enum `QueryTermKind`
            Mut = 1,

            /// The variant `With` in the enum `QueryTermKind`
            With = 2,

            /// The variant `Without` in the enum `QueryTermKind`
            Without = 3,

            /// The variant `Changed` in the enum `QueryTermKind`
            Changed = 4,
        }

        impl QueryTermKind {
            /// Array containing all valid variants of QueryTermKind
            pub const ENUM_VALUES: [Self; 5] = [
                Self::Ref,
                Self::Mut,
                Self::With,
                Self::Without,
                Self::Changed,
            ];
        }

        impl ::core::convert::TryFrom<u8> for QueryTermKind {
            type Error = ::planus::errors::UnknownEnumTagKind;
            #[inline]
            fn try_from(
                value: u8,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTagKind> {
                #[allow(clippy::match_single_binding)]
                match value {
                    0 => ::core::result::Result::Ok(QueryTermKind::Ref),
                    1 => ::core::result::Result::Ok(QueryTermKind::Mut),
                    2 => ::core::result::Result::Ok(QueryTermKind::With),
                    3 => ::core::result::Result::Ok(QueryTermKind::Without),
                    4 => ::core::result::Result::Ok(QueryTermKind::Changed),

                    _ => ::core::result::Result::Err(::planus::errors::UnknownEnumTagKind {
                        tag: value as i128,
                    }),
                }
            }
        }

        impl ::core::convert::From<QueryTermKind> for u8 {
            #[inline]
            fn from(value: QueryTermKind) -> Self {
                value as u8
            }
        }

        /// # Safety
        /// The Planus compiler correctly calculates `ALIGNMENT` and `SIZE`.
        unsafe impl ::planus::Primitive for QueryTermKind {
            const ALIGNMENT: usize = 1;
            const SIZE: usize = 1;
        }

        impl ::planus::WriteAsPrimitive<QueryTermKind> for QueryTermKind {
            #[inline]
            fn write<const N: usize>(&self, cursor: ::planus::Cursor<'_, N>, buffer_position: u32) {
                (*self as u8).write(cursor, buffer_position);
            }
        }

        impl ::planus::WriteAs<QueryTermKind> for QueryTermKind {
            type Prepared = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> QueryTermKind {
                *self
            }
        }

        impl ::planus::WriteAsDefault<QueryTermKind, QueryTermKind> for QueryTermKind {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
                default: &QueryTermKind,
            ) -> ::core::option::Option<QueryTermKind> {
                if self == default {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(*self)
                }
            }
        }

        impl ::planus::WriteAsOptional<QueryTermKind> for QueryTermKind {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<QueryTermKind> {
                ::core::option::Option::Some(*self)
            }
        }

        impl<'buf> ::planus::TableRead<'buf> for QueryTermKind {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                let n: u8 = ::planus::TableRead::from_buffer(buffer, offset)?;
                ::core::result::Result::Ok(::core::convert::TryInto::try_into(n)?)
            }
        }

        impl<'buf> ::planus::VectorReadInner<'buf> for QueryTermKind {
            type Error = ::planus::errors::UnknownEnumTag;
            const STRIDE: usize = 1;
            #[inline]
            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTag> {
                let value = unsafe { *buffer.buffer.get_unchecked(offset) };
                let value: ::core::result::Result<Self, _> =
                    ::core::convert::TryInto::try_into(value);
                value.map_err(|error_kind| {
                    error_kind.with_error_location(
                        "QueryTermKind",
                        "VectorRead::from_buffer",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<QueryTermKind> for QueryTermKind {
            const STRIDE: usize = 1;

            type Value = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> Self {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[Self],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 1];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - i as u32,
                    );
                }
            }
        }

        /// The enum `ParamKind` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Enum `ParamKind` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:97`
        #[derive(
            Copy,
            Clone,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #[repr(u8)]
        pub enum ParamKind {
            /// The variant `Commands` in the enum `ParamKind`
            Commands = 0,

            /// The variant `Query` in the enum `ParamKind`
            Query = 1,
        }

        impl ParamKind {
            /// Array containing all valid variants of ParamKind
            pub const ENUM_VALUES: [Self; 2] = [Self::Commands, Self::Query];
        }

        impl ::core::convert::TryFrom<u8> for ParamKind {
            type Error = ::planus::errors::UnknownEnumTagKind;
            #[inline]
            fn try_from(
                value: u8,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTagKind> {
                #[allow(clippy::match_single_binding)]
                match value {
                    0 => ::core::result::Result::Ok(ParamKind::Commands),
                    1 => ::core::result::Result::Ok(ParamKind::Query),

                    _ => ::core::result::Result::Err(::planus::errors::UnknownEnumTagKind {
                        tag: value as i128,
                    }),
                }
            }
        }

        impl ::core::convert::From<ParamKind> for u8 {
            #[inline]
            fn from(value: ParamKind) -> Self {
                value as u8
            }
        }

        /// # Safety
        /// The Planus compiler correctly calculates `ALIGNMENT` and `SIZE`.
        unsafe impl ::planus::Primitive for ParamKind {
            const ALIGNMENT: usize = 1;
            const SIZE: usize = 1;
        }

        impl ::planus::WriteAsPrimitive<ParamKind> for ParamKind {
            #[inline]
            fn write<const N: usize>(&self, cursor: ::planus::Cursor<'_, N>, buffer_position: u32) {
                (*self as u8).write(cursor, buffer_position);
            }
        }

        impl ::planus::WriteAs<ParamKind> for ParamKind {
            type Prepared = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> ParamKind {
                *self
            }
        }

        impl ::planus::WriteAsDefault<ParamKind, ParamKind> for ParamKind {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
                default: &ParamKind,
            ) -> ::core::option::Option<ParamKind> {
                if self == default {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(*self)
                }
            }
        }

        impl ::planus::WriteAsOptional<ParamKind> for ParamKind {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<ParamKind> {
                ::core::option::Option::Some(*self)
            }
        }

        impl<'buf> ::planus::TableRead<'buf> for ParamKind {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                let n: u8 = ::planus::TableRead::from_buffer(buffer, offset)?;
                ::core::result::Result::Ok(::core::convert::TryInto::try_into(n)?)
            }
        }

        impl<'buf> ::planus::VectorReadInner<'buf> for ParamKind {
            type Error = ::planus::errors::UnknownEnumTag;
            const STRIDE: usize = 1;
            #[inline]
            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTag> {
                let value = unsafe { *buffer.buffer.get_unchecked(offset) };
                let value: ::core::result::Result<Self, _> =
                    ::core::convert::TryInto::try_into(value);
                value.map_err(|error_kind| {
                    error_kind.with_error_location(
                        "ParamKind",
                        "VectorRead::from_buffer",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<ParamKind> for ParamKind {
            const STRIDE: usize = 1;

            type Value = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> Self {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[Self],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 1];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - i as u32,
                    );
                }
            }
        }

        /// The enum `ObserverKind` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Enum `ObserverKind` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:102`
        #[derive(
            Copy,
            Clone,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        #[repr(u8)]
        pub enum ObserverKind {
            /// The variant `Spawn` in the enum `ObserverKind`
            Spawn = 0,

            /// The variant `Despawn` in the enum `ObserverKind`
            Despawn = 1,

            /// The variant `Insert` in the enum `ObserverKind`
            Insert = 2,

            /// The variant `Remove` in the enum `ObserverKind`
            Remove = 3,

            /// The variant `Custom` in the enum `ObserverKind`
            Custom = 4,
        }

        impl ObserverKind {
            /// Array containing all valid variants of ObserverKind
            pub const ENUM_VALUES: [Self; 5] = [
                Self::Spawn,
                Self::Despawn,
                Self::Insert,
                Self::Remove,
                Self::Custom,
            ];
        }

        impl ::core::convert::TryFrom<u8> for ObserverKind {
            type Error = ::planus::errors::UnknownEnumTagKind;
            #[inline]
            fn try_from(
                value: u8,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTagKind> {
                #[allow(clippy::match_single_binding)]
                match value {
                    0 => ::core::result::Result::Ok(ObserverKind::Spawn),
                    1 => ::core::result::Result::Ok(ObserverKind::Despawn),
                    2 => ::core::result::Result::Ok(ObserverKind::Insert),
                    3 => ::core::result::Result::Ok(ObserverKind::Remove),
                    4 => ::core::result::Result::Ok(ObserverKind::Custom),

                    _ => ::core::result::Result::Err(::planus::errors::UnknownEnumTagKind {
                        tag: value as i128,
                    }),
                }
            }
        }

        impl ::core::convert::From<ObserverKind> for u8 {
            #[inline]
            fn from(value: ObserverKind) -> Self {
                value as u8
            }
        }

        /// # Safety
        /// The Planus compiler correctly calculates `ALIGNMENT` and `SIZE`.
        unsafe impl ::planus::Primitive for ObserverKind {
            const ALIGNMENT: usize = 1;
            const SIZE: usize = 1;
        }

        impl ::planus::WriteAsPrimitive<ObserverKind> for ObserverKind {
            #[inline]
            fn write<const N: usize>(&self, cursor: ::planus::Cursor<'_, N>, buffer_position: u32) {
                (*self as u8).write(cursor, buffer_position);
            }
        }

        impl ::planus::WriteAs<ObserverKind> for ObserverKind {
            type Prepared = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> ObserverKind {
                *self
            }
        }

        impl ::planus::WriteAsDefault<ObserverKind, ObserverKind> for ObserverKind {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
                default: &ObserverKind,
            ) -> ::core::option::Option<ObserverKind> {
                if self == default {
                    ::core::option::Option::None
                } else {
                    ::core::option::Option::Some(*self)
                }
            }
        }

        impl ::planus::WriteAsOptional<ObserverKind> for ObserverKind {
            type Prepared = Self;

            #[inline]
            fn prepare(
                &self,
                _builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<ObserverKind> {
                ::core::option::Option::Some(*self)
            }
        }

        impl<'buf> ::planus::TableRead<'buf> for ObserverKind {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                let n: u8 = ::planus::TableRead::from_buffer(buffer, offset)?;
                ::core::result::Result::Ok(::core::convert::TryInto::try_into(n)?)
            }
        }

        impl<'buf> ::planus::VectorReadInner<'buf> for ObserverKind {
            type Error = ::planus::errors::UnknownEnumTag;
            const STRIDE: usize = 1;
            #[inline]
            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'buf>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::UnknownEnumTag> {
                let value = unsafe { *buffer.buffer.get_unchecked(offset) };
                let value: ::core::result::Result<Self, _> =
                    ::core::convert::TryInto::try_into(value);
                value.map_err(|error_kind| {
                    error_kind.with_error_location(
                        "ObserverKind",
                        "VectorRead::from_buffer",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<ObserverKind> for ObserverKind {
            const STRIDE: usize = 1;

            type Value = Self;

            #[inline]
            fn prepare(&self, _builder: &mut ::planus::Builder) -> Self {
                *self
            }

            #[inline]
            unsafe fn write_values(
                values: &[Self],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 1];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - i as u32,
                    );
                }
            }
        }

        /// The table `QueryTerm` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `QueryTerm` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:110`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct QueryTerm {
            /// The field `kind` in the table `QueryTerm`
            pub kind: self::QueryTermKind,
            /// The field `type_id` in the table `QueryTerm`
            pub type_id: u16,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for QueryTerm {
            fn default() -> Self {
                Self {
                    kind: self::QueryTermKind::Ref,
                    type_id: 0,
                }
            }
        }

        impl QueryTerm {
            /// Creates a [QueryTermBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> QueryTermBuilder<()> {
                QueryTermBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_kind: impl ::planus::WriteAsDefault<self::QueryTermKind, self::QueryTermKind>,
                field_type_id: impl ::planus::WriteAsDefault<u16, u16>,
            ) -> ::planus::Offset<Self> {
                let prepared_kind = field_kind.prepare(builder, &self::QueryTermKind::Ref);
                let prepared_type_id = field_type_id.prepare(builder, &0);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_type_id.is_some() {
                    table_writer.write_entry::<u16>(1);
                }
                if prepared_kind.is_some() {
                    table_writer.write_entry::<self::QueryTermKind>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_type_id) = prepared_type_id {
                            object_writer.write::<_, _, 2>(&prepared_type_id);
                        }
                        if let ::core::option::Option::Some(prepared_kind) = prepared_kind {
                            object_writer.write::<_, _, 1>(&prepared_kind);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<QueryTerm>> for QueryTerm {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryTerm> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<QueryTerm>> for QueryTerm {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<QueryTerm>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<QueryTerm> for QueryTerm {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryTerm> {
                QueryTerm::create(builder, self.kind, self.type_id)
            }
        }

        /// Builder for serializing an instance of the [QueryTerm] type.
        ///
        /// Can be created using the [QueryTerm::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct QueryTermBuilder<State>(State);

        impl QueryTermBuilder<()> {
            /// Setter for the [`kind` field](QueryTerm#structfield.kind).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn kind<T0>(self, value: T0) -> QueryTermBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<self::QueryTermKind, self::QueryTermKind>,
            {
                QueryTermBuilder((value,))
            }

            /// Sets the [`kind` field](QueryTerm#structfield.kind) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn kind_as_default(self) -> QueryTermBuilder<(::planus::DefaultValue,)> {
                self.kind(::planus::DefaultValue)
            }
        }

        impl<T0> QueryTermBuilder<(T0,)> {
            /// Setter for the [`type_id` field](QueryTerm#structfield.type_id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_id<T1>(self, value: T1) -> QueryTermBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsDefault<u16, u16>,
            {
                let (v0,) = self.0;
                QueryTermBuilder((v0, value))
            }

            /// Sets the [`type_id` field](QueryTerm#structfield.type_id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_id_as_default(self) -> QueryTermBuilder<(T0, ::planus::DefaultValue)> {
                self.type_id(::planus::DefaultValue)
            }
        }

        impl<T0, T1> QueryTermBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [QueryTerm].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryTerm>
            where
                Self: ::planus::WriteAsOffset<QueryTerm>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<self::QueryTermKind, self::QueryTermKind>,
                T1: ::planus::WriteAsDefault<u16, u16>,
            > ::planus::WriteAs<::planus::Offset<QueryTerm>> for QueryTermBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<QueryTerm>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryTerm> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<self::QueryTermKind, self::QueryTermKind>,
                T1: ::planus::WriteAsDefault<u16, u16>,
            > ::planus::WriteAsOptional<::planus::Offset<QueryTerm>>
            for QueryTermBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<QueryTerm>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<QueryTerm>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<self::QueryTermKind, self::QueryTermKind>,
                T1: ::planus::WriteAsDefault<u16, u16>,
            > ::planus::WriteAsOffset<QueryTerm> for QueryTermBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryTerm> {
                let (v0, v1) = &self.0;
                QueryTerm::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [QueryTerm].
        #[derive(Copy, Clone)]
        pub struct QueryTermRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> QueryTermRef<'a> {
            /// Getter for the [`kind` field](QueryTerm#structfield.kind).
            #[inline]
            pub fn kind(&self) -> ::planus::Result<self::QueryTermKind> {
                ::core::result::Result::Ok(
                    self.0
                        .access(0, "QueryTerm", "kind")?
                        .unwrap_or(self::QueryTermKind::Ref),
                )
            }

            /// Getter for the [`type_id` field](QueryTerm#structfield.type_id).
            #[inline]
            pub fn type_id(&self) -> ::planus::Result<u16> {
                ::core::result::Result::Ok(self.0.access(1, "QueryTerm", "type_id")?.unwrap_or(0))
            }
        }

        impl<'a> ::core::fmt::Debug for QueryTermRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("QueryTermRef");
                f.field("kind", &self.kind());
                f.field("type_id", &self.type_id());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<QueryTermRef<'a>> for QueryTerm {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: QueryTermRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    kind: ::core::convert::TryInto::try_into(value.kind()?)?,
                    type_id: ::core::convert::TryInto::try_into(value.type_id()?)?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for QueryTermRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for QueryTermRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[QueryTermRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<QueryTerm>> for QueryTerm {
            type Value = ::planus::Offset<QueryTerm>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<QueryTerm>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for QueryTermRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[QueryTermRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `QueryDecl` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `QueryDecl` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:115`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct QueryDecl {
            /// The field `terms` in the table `QueryDecl`
            pub terms: ::core::option::Option<::planus::alloc::vec::Vec<self::QueryTerm>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for QueryDecl {
            fn default() -> Self {
                Self {
                    terms: ::core::default::Default::default(),
                }
            }
        }

        impl QueryDecl {
            /// Creates a [QueryDeclBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> QueryDeclBuilder<()> {
                QueryDeclBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_terms: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::QueryTerm>]>,
                >,
            ) -> ::planus::Offset<Self> {
                let prepared_terms = field_terms.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<6> =
                    ::core::default::Default::default();
                if prepared_terms.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::QueryTerm>]>>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_terms) = prepared_terms {
                            object_writer.write::<_, _, 4>(&prepared_terms);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<QueryDecl>> for QueryDecl {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryDecl> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<QueryDecl>> for QueryDecl {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<QueryDecl>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<QueryDecl> for QueryDecl {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryDecl> {
                QueryDecl::create(builder, &self.terms)
            }
        }

        /// Builder for serializing an instance of the [QueryDecl] type.
        ///
        /// Can be created using the [QueryDecl::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct QueryDeclBuilder<State>(State);

        impl QueryDeclBuilder<()> {
            /// Setter for the [`terms` field](QueryDecl#structfield.terms).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn terms<T0>(self, value: T0) -> QueryDeclBuilder<(T0,)>
            where
                T0: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::QueryTerm>]>,
                >,
            {
                QueryDeclBuilder((value,))
            }

            /// Sets the [`terms` field](QueryDecl#structfield.terms) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn terms_as_null(self) -> QueryDeclBuilder<((),)> {
                self.terms(())
            }
        }

        impl<T0> QueryDeclBuilder<(T0,)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [QueryDecl].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryDecl>
            where
                Self: ::planus::WriteAsOffset<QueryDecl>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::QueryTerm>]>>,
            > ::planus::WriteAs<::planus::Offset<QueryDecl>> for QueryDeclBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<QueryDecl>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryDecl> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::QueryTerm>]>>,
            > ::planus::WriteAsOptional<::planus::Offset<QueryDecl>> for QueryDeclBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<QueryDecl>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<QueryDecl>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::QueryTerm>]>>,
            > ::planus::WriteAsOffset<QueryDecl> for QueryDeclBuilder<(T0,)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryDecl> {
                let (v0,) = &self.0;
                QueryDecl::create(builder, v0)
            }
        }

        /// Reference to a deserialized [QueryDecl].
        #[derive(Copy, Clone)]
        pub struct QueryDeclRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> QueryDeclRef<'a> {
            /// Getter for the [`terms` field](QueryDecl#structfield.terms).
            #[inline]
            pub fn terms(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::QueryTermRef<'a>>>,
                >,
            > {
                self.0.access(0, "QueryDecl", "terms")
            }
        }

        impl<'a> ::core::fmt::Debug for QueryDeclRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("QueryDeclRef");
                if let ::core::option::Option::Some(field_terms) = self.terms().transpose() {
                    f.field("terms", &field_terms);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<QueryDeclRef<'a>> for QueryDecl {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: QueryDeclRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    terms: if let ::core::option::Option::Some(terms) = value.terms()? {
                        ::core::option::Option::Some(terms.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for QueryDeclRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for QueryDeclRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[QueryDeclRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<QueryDecl>> for QueryDecl {
            type Value = ::planus::Offset<QueryDecl>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<QueryDecl>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for QueryDeclRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[QueryDeclRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `ParamDecl` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `ParamDecl` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:119`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct ParamDecl {
            /// The field `kind` in the table `ParamDecl`
            pub kind: self::ParamKind,
            /// The field `query` in the table `ParamDecl`
            pub query: ::core::option::Option<::planus::alloc::boxed::Box<self::QueryDecl>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for ParamDecl {
            fn default() -> Self {
                Self {
                    kind: self::ParamKind::Commands,
                    query: ::core::default::Default::default(),
                }
            }
        }

        impl ParamDecl {
            /// Creates a [ParamDeclBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> ParamDeclBuilder<()> {
                ParamDeclBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_kind: impl ::planus::WriteAsDefault<self::ParamKind, self::ParamKind>,
                field_query: impl ::planus::WriteAsOptional<::planus::Offset<self::QueryDecl>>,
            ) -> ::planus::Offset<Self> {
                let prepared_kind = field_kind.prepare(builder, &self::ParamKind::Commands);
                let prepared_query = field_query.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_query.is_some() {
                    table_writer.write_entry::<::planus::Offset<self::QueryDecl>>(1);
                }
                if prepared_kind.is_some() {
                    table_writer.write_entry::<self::ParamKind>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_query) = prepared_query {
                            object_writer.write::<_, _, 4>(&prepared_query);
                        }
                        if let ::core::option::Option::Some(prepared_kind) = prepared_kind {
                            object_writer.write::<_, _, 1>(&prepared_kind);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<ParamDecl>> for ParamDecl {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ParamDecl> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<ParamDecl>> for ParamDecl {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ParamDecl>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<ParamDecl> for ParamDecl {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ParamDecl> {
                ParamDecl::create(builder, self.kind, &self.query)
            }
        }

        /// Builder for serializing an instance of the [ParamDecl] type.
        ///
        /// Can be created using the [ParamDecl::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct ParamDeclBuilder<State>(State);

        impl ParamDeclBuilder<()> {
            /// Setter for the [`kind` field](ParamDecl#structfield.kind).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn kind<T0>(self, value: T0) -> ParamDeclBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<self::ParamKind, self::ParamKind>,
            {
                ParamDeclBuilder((value,))
            }

            /// Sets the [`kind` field](ParamDecl#structfield.kind) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn kind_as_default(self) -> ParamDeclBuilder<(::planus::DefaultValue,)> {
                self.kind(::planus::DefaultValue)
            }
        }

        impl<T0> ParamDeclBuilder<(T0,)> {
            /// Setter for the [`query` field](ParamDecl#structfield.query).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn query<T1>(self, value: T1) -> ParamDeclBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<::planus::Offset<self::QueryDecl>>,
            {
                let (v0,) = self.0;
                ParamDeclBuilder((v0, value))
            }

            /// Sets the [`query` field](ParamDecl#structfield.query) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn query_as_null(self) -> ParamDeclBuilder<(T0, ())> {
                self.query(())
            }
        }

        impl<T0, T1> ParamDeclBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [ParamDecl].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<ParamDecl>
            where
                Self: ::planus::WriteAsOffset<ParamDecl>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<self::ParamKind, self::ParamKind>,
                T1: ::planus::WriteAsOptional<::planus::Offset<self::QueryDecl>>,
            > ::planus::WriteAs<::planus::Offset<ParamDecl>> for ParamDeclBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<ParamDecl>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ParamDecl> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<self::ParamKind, self::ParamKind>,
                T1: ::planus::WriteAsOptional<::planus::Offset<self::QueryDecl>>,
            > ::planus::WriteAsOptional<::planus::Offset<ParamDecl>>
            for ParamDeclBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<ParamDecl>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ParamDecl>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<self::ParamKind, self::ParamKind>,
                T1: ::planus::WriteAsOptional<::planus::Offset<self::QueryDecl>>,
            > ::planus::WriteAsOffset<ParamDecl> for ParamDeclBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ParamDecl> {
                let (v0, v1) = &self.0;
                ParamDecl::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [ParamDecl].
        #[derive(Copy, Clone)]
        pub struct ParamDeclRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> ParamDeclRef<'a> {
            /// Getter for the [`kind` field](ParamDecl#structfield.kind).
            #[inline]
            pub fn kind(&self) -> ::planus::Result<self::ParamKind> {
                ::core::result::Result::Ok(
                    self.0
                        .access(0, "ParamDecl", "kind")?
                        .unwrap_or(self::ParamKind::Commands),
                )
            }

            /// Getter for the [`query` field](ParamDecl#structfield.query).
            #[inline]
            pub fn query(
                &self,
            ) -> ::planus::Result<::core::option::Option<self::QueryDeclRef<'a>>> {
                self.0.access(1, "ParamDecl", "query")
            }
        }

        impl<'a> ::core::fmt::Debug for ParamDeclRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("ParamDeclRef");
                f.field("kind", &self.kind());
                if let ::core::option::Option::Some(field_query) = self.query().transpose() {
                    f.field("query", &field_query);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<ParamDeclRef<'a>> for ParamDecl {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: ParamDeclRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    kind: ::core::convert::TryInto::try_into(value.kind()?)?,
                    query: if let ::core::option::Option::Some(query) = value.query()? {
                        ::core::option::Option::Some(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryInto::try_into(query)?,
                        ))
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for ParamDeclRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for ParamDeclRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[ParamDeclRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<ParamDecl>> for ParamDecl {
            type Value = ::planus::Offset<ParamDecl>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<ParamDecl>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for ParamDeclRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[ParamDeclRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `SystemDecl` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `SystemDecl` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:124`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct SystemDecl {
            /// The field `id` in the table `SystemDecl`
            pub id: u32,
            /// The field `name` in the table `SystemDecl`
            pub name: ::core::option::Option<::planus::alloc::string::String>,
            /// The field `schedule` in the table `SystemDecl`
            pub schedule: self::Schedule,
            /// The field `custom_stage` in the table `SystemDecl`
            pub custom_stage: ::core::option::Option<::planus::alloc::string::String>,
            /// The field `params` in the table `SystemDecl`
            pub params: ::core::option::Option<::planus::alloc::vec::Vec<self::ParamDecl>>,
            /// The field `after` in the table `SystemDecl`
            pub after: ::core::option::Option<::planus::alloc::vec::Vec<u32>>,
            /// The field `before` in the table `SystemDecl`
            pub before: ::core::option::Option<::planus::alloc::vec::Vec<u32>>,
            /// The field `interval_ms` in the table `SystemDecl`
            pub interval_ms: u32,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for SystemDecl {
            fn default() -> Self {
                Self {
                    id: 0,
                    name: ::core::default::Default::default(),
                    schedule: self::Schedule::ModStartup,
                    custom_stage: ::core::default::Default::default(),
                    params: ::core::default::Default::default(),
                    after: ::core::default::Default::default(),
                    before: ::core::default::Default::default(),
                    interval_ms: 0,
                }
            }
        }

        impl SystemDecl {
            /// Creates a [SystemDeclBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> SystemDeclBuilder<()> {
                SystemDeclBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_id: impl ::planus::WriteAsDefault<u32, u32>,
                field_name: impl ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                field_schedule: impl ::planus::WriteAsDefault<self::Schedule, self::Schedule>,
                field_custom_stage: impl ::planus::WriteAsOptional<
                    ::planus::Offset<::core::primitive::str>,
                >,
                field_params: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::ParamDecl>]>,
                >,
                field_after: impl ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
                field_before: impl ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
                field_interval_ms: impl ::planus::WriteAsDefault<u32, u32>,
            ) -> ::planus::Offset<Self> {
                let prepared_id = field_id.prepare(builder, &0);
                let prepared_name = field_name.prepare(builder);
                let prepared_schedule =
                    field_schedule.prepare(builder, &self::Schedule::ModStartup);
                let prepared_custom_stage = field_custom_stage.prepare(builder);
                let prepared_params = field_params.prepare(builder);
                let prepared_after = field_after.prepare(builder);
                let prepared_before = field_before.prepare(builder);
                let prepared_interval_ms = field_interval_ms.prepare(builder, &0);

                let mut table_writer: ::planus::table_writer::TableWriter<20> =
                    ::core::default::Default::default();
                if prepared_id.is_some() {
                    table_writer.write_entry::<u32>(0);
                }
                if prepared_name.is_some() {
                    table_writer.write_entry::<::planus::Offset<str>>(1);
                }
                if prepared_custom_stage.is_some() {
                    table_writer.write_entry::<::planus::Offset<str>>(3);
                }
                if prepared_params.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::ParamDecl>]>>(4);
                }
                if prepared_after.is_some() {
                    table_writer.write_entry::<::planus::Offset<[u32]>>(5);
                }
                if prepared_before.is_some() {
                    table_writer.write_entry::<::planus::Offset<[u32]>>(6);
                }
                if prepared_interval_ms.is_some() {
                    table_writer.write_entry::<u32>(7);
                }
                if prepared_schedule.is_some() {
                    table_writer.write_entry::<self::Schedule>(2);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_id) = prepared_id {
                            object_writer.write::<_, _, 4>(&prepared_id);
                        }
                        if let ::core::option::Option::Some(prepared_name) = prepared_name {
                            object_writer.write::<_, _, 4>(&prepared_name);
                        }
                        if let ::core::option::Option::Some(prepared_custom_stage) =
                            prepared_custom_stage
                        {
                            object_writer.write::<_, _, 4>(&prepared_custom_stage);
                        }
                        if let ::core::option::Option::Some(prepared_params) = prepared_params {
                            object_writer.write::<_, _, 4>(&prepared_params);
                        }
                        if let ::core::option::Option::Some(prepared_after) = prepared_after {
                            object_writer.write::<_, _, 4>(&prepared_after);
                        }
                        if let ::core::option::Option::Some(prepared_before) = prepared_before {
                            object_writer.write::<_, _, 4>(&prepared_before);
                        }
                        if let ::core::option::Option::Some(prepared_interval_ms) =
                            prepared_interval_ms
                        {
                            object_writer.write::<_, _, 4>(&prepared_interval_ms);
                        }
                        if let ::core::option::Option::Some(prepared_schedule) = prepared_schedule {
                            object_writer.write::<_, _, 1>(&prepared_schedule);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<SystemDecl>> for SystemDecl {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemDecl> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<SystemDecl>> for SystemDecl {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SystemDecl>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<SystemDecl> for SystemDecl {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemDecl> {
                SystemDecl::create(
                    builder,
                    self.id,
                    &self.name,
                    self.schedule,
                    &self.custom_stage,
                    &self.params,
                    &self.after,
                    &self.before,
                    self.interval_ms,
                )
            }
        }

        /// Builder for serializing an instance of the [SystemDecl] type.
        ///
        /// Can be created using the [SystemDecl::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct SystemDeclBuilder<State>(State);

        impl SystemDeclBuilder<()> {
            /// Setter for the [`id` field](SystemDecl#structfield.id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn id<T0>(self, value: T0) -> SystemDeclBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                SystemDeclBuilder((value,))
            }

            /// Sets the [`id` field](SystemDecl#structfield.id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn id_as_default(self) -> SystemDeclBuilder<(::planus::DefaultValue,)> {
                self.id(::planus::DefaultValue)
            }
        }

        impl<T0> SystemDeclBuilder<(T0,)> {
            /// Setter for the [`name` field](SystemDecl#structfield.name).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn name<T1>(self, value: T1) -> SystemDeclBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            {
                let (v0,) = self.0;
                SystemDeclBuilder((v0, value))
            }

            /// Sets the [`name` field](SystemDecl#structfield.name) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn name_as_null(self) -> SystemDeclBuilder<(T0, ())> {
                self.name(())
            }
        }

        impl<T0, T1> SystemDeclBuilder<(T0, T1)> {
            /// Setter for the [`schedule` field](SystemDecl#structfield.schedule).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn schedule<T2>(self, value: T2) -> SystemDeclBuilder<(T0, T1, T2)>
            where
                T2: ::planus::WriteAsDefault<self::Schedule, self::Schedule>,
            {
                let (v0, v1) = self.0;
                SystemDeclBuilder((v0, v1, value))
            }

            /// Sets the [`schedule` field](SystemDecl#structfield.schedule) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn schedule_as_default(
                self,
            ) -> SystemDeclBuilder<(T0, T1, ::planus::DefaultValue)> {
                self.schedule(::planus::DefaultValue)
            }
        }

        impl<T0, T1, T2> SystemDeclBuilder<(T0, T1, T2)> {
            /// Setter for the [`custom_stage` field](SystemDecl#structfield.custom_stage).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn custom_stage<T3>(self, value: T3) -> SystemDeclBuilder<(T0, T1, T2, T3)>
            where
                T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            {
                let (v0, v1, v2) = self.0;
                SystemDeclBuilder((v0, v1, v2, value))
            }

            /// Sets the [`custom_stage` field](SystemDecl#structfield.custom_stage) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn custom_stage_as_null(self) -> SystemDeclBuilder<(T0, T1, T2, ())> {
                self.custom_stage(())
            }
        }

        impl<T0, T1, T2, T3> SystemDeclBuilder<(T0, T1, T2, T3)> {
            /// Setter for the [`params` field](SystemDecl#structfield.params).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn params<T4>(self, value: T4) -> SystemDeclBuilder<(T0, T1, T2, T3, T4)>
            where
                T4: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::ParamDecl>]>,
                >,
            {
                let (v0, v1, v2, v3) = self.0;
                SystemDeclBuilder((v0, v1, v2, v3, value))
            }

            /// Sets the [`params` field](SystemDecl#structfield.params) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn params_as_null(self) -> SystemDeclBuilder<(T0, T1, T2, T3, ())> {
                self.params(())
            }
        }

        impl<T0, T1, T2, T3, T4> SystemDeclBuilder<(T0, T1, T2, T3, T4)> {
            /// Setter for the [`after` field](SystemDecl#structfield.after).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn after<T5>(self, value: T5) -> SystemDeclBuilder<(T0, T1, T2, T3, T4, T5)>
            where
                T5: ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
            {
                let (v0, v1, v2, v3, v4) = self.0;
                SystemDeclBuilder((v0, v1, v2, v3, v4, value))
            }

            /// Sets the [`after` field](SystemDecl#structfield.after) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn after_as_null(self) -> SystemDeclBuilder<(T0, T1, T2, T3, T4, ())> {
                self.after(())
            }
        }

        impl<T0, T1, T2, T3, T4, T5> SystemDeclBuilder<(T0, T1, T2, T3, T4, T5)> {
            /// Setter for the [`before` field](SystemDecl#structfield.before).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn before<T6>(self, value: T6) -> SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, T6)>
            where
                T6: ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
            {
                let (v0, v1, v2, v3, v4, v5) = self.0;
                SystemDeclBuilder((v0, v1, v2, v3, v4, v5, value))
            }

            /// Sets the [`before` field](SystemDecl#structfield.before) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn before_as_null(self) -> SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, ())> {
                self.before(())
            }
        }

        impl<T0, T1, T2, T3, T4, T5, T6> SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, T6)> {
            /// Setter for the [`interval_ms` field](SystemDecl#structfield.interval_ms).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn interval_ms<T7>(
                self,
                value: T7,
            ) -> SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, T6, T7)>
            where
                T7: ::planus::WriteAsDefault<u32, u32>,
            {
                let (v0, v1, v2, v3, v4, v5, v6) = self.0;
                SystemDeclBuilder((v0, v1, v2, v3, v4, v5, v6, value))
            }

            /// Sets the [`interval_ms` field](SystemDecl#structfield.interval_ms) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn interval_ms_as_default(
                self,
            ) -> SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, T6, ::planus::DefaultValue)>
            {
                self.interval_ms(::planus::DefaultValue)
            }
        }

        impl<T0, T1, T2, T3, T4, T5, T6, T7> SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, T6, T7)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [SystemDecl].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemDecl>
            where
                Self: ::planus::WriteAsOffset<SystemDecl>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T2: ::planus::WriteAsDefault<self::Schedule, self::Schedule>,
                T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T4: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::ParamDecl>]>>,
                T5: ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
                T6: ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
                T7: ::planus::WriteAsDefault<u32, u32>,
            > ::planus::WriteAs<::planus::Offset<SystemDecl>>
            for SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, T6, T7)>
        {
            type Prepared = ::planus::Offset<SystemDecl>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemDecl> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T2: ::planus::WriteAsDefault<self::Schedule, self::Schedule>,
                T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T4: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::ParamDecl>]>>,
                T5: ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
                T6: ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
                T7: ::planus::WriteAsDefault<u32, u32>,
            > ::planus::WriteAsOptional<::planus::Offset<SystemDecl>>
            for SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, T6, T7)>
        {
            type Prepared = ::planus::Offset<SystemDecl>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SystemDecl>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T2: ::planus::WriteAsDefault<self::Schedule, self::Schedule>,
                T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T4: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::ParamDecl>]>>,
                T5: ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
                T6: ::planus::WriteAsOptional<::planus::Offset<[u32]>>,
                T7: ::planus::WriteAsDefault<u32, u32>,
            > ::planus::WriteAsOffset<SystemDecl>
            for SystemDeclBuilder<(T0, T1, T2, T3, T4, T5, T6, T7)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemDecl> {
                let (v0, v1, v2, v3, v4, v5, v6, v7) = &self.0;
                SystemDecl::create(builder, v0, v1, v2, v3, v4, v5, v6, v7)
            }
        }

        /// Reference to a deserialized [SystemDecl].
        #[derive(Copy, Clone)]
        pub struct SystemDeclRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> SystemDeclRef<'a> {
            /// Getter for the [`id` field](SystemDecl#structfield.id).
            #[inline]
            pub fn id(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(self.0.access(0, "SystemDecl", "id")?.unwrap_or(0))
            }

            /// Getter for the [`name` field](SystemDecl#structfield.name).
            #[inline]
            pub fn name(
                &self,
            ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>> {
                self.0.access(1, "SystemDecl", "name")
            }

            /// Getter for the [`schedule` field](SystemDecl#structfield.schedule).
            #[inline]
            pub fn schedule(&self) -> ::planus::Result<self::Schedule> {
                ::core::result::Result::Ok(
                    self.0
                        .access(2, "SystemDecl", "schedule")?
                        .unwrap_or(self::Schedule::ModStartup),
                )
            }

            /// Getter for the [`custom_stage` field](SystemDecl#structfield.custom_stage).
            #[inline]
            pub fn custom_stage(
                &self,
            ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>> {
                self.0.access(3, "SystemDecl", "custom_stage")
            }

            /// Getter for the [`params` field](SystemDecl#structfield.params).
            #[inline]
            pub fn params(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::ParamDeclRef<'a>>>,
                >,
            > {
                self.0.access(4, "SystemDecl", "params")
            }

            /// Getter for the [`after` field](SystemDecl#structfield.after).
            #[inline]
            pub fn after(
                &self,
            ) -> ::planus::Result<::core::option::Option<::planus::Vector<'a, u32>>> {
                self.0.access(5, "SystemDecl", "after")
            }

            /// Getter for the [`before` field](SystemDecl#structfield.before).
            #[inline]
            pub fn before(
                &self,
            ) -> ::planus::Result<::core::option::Option<::planus::Vector<'a, u32>>> {
                self.0.access(6, "SystemDecl", "before")
            }

            /// Getter for the [`interval_ms` field](SystemDecl#structfield.interval_ms).
            #[inline]
            pub fn interval_ms(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(
                    self.0.access(7, "SystemDecl", "interval_ms")?.unwrap_or(0),
                )
            }
        }

        impl<'a> ::core::fmt::Debug for SystemDeclRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("SystemDeclRef");
                f.field("id", &self.id());
                if let ::core::option::Option::Some(field_name) = self.name().transpose() {
                    f.field("name", &field_name);
                }
                f.field("schedule", &self.schedule());
                if let ::core::option::Option::Some(field_custom_stage) =
                    self.custom_stage().transpose()
                {
                    f.field("custom_stage", &field_custom_stage);
                }
                if let ::core::option::Option::Some(field_params) = self.params().transpose() {
                    f.field("params", &field_params);
                }
                if let ::core::option::Option::Some(field_after) = self.after().transpose() {
                    f.field("after", &field_after);
                }
                if let ::core::option::Option::Some(field_before) = self.before().transpose() {
                    f.field("before", &field_before);
                }
                f.field("interval_ms", &self.interval_ms());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<SystemDeclRef<'a>> for SystemDecl {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: SystemDeclRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    id: ::core::convert::TryInto::try_into(value.id()?)?,
                    name: value.name()?.map(::core::convert::Into::into),
                    schedule: ::core::convert::TryInto::try_into(value.schedule()?)?,
                    custom_stage: value.custom_stage()?.map(::core::convert::Into::into),
                    params: if let ::core::option::Option::Some(params) = value.params()? {
                        ::core::option::Option::Some(params.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                    after: if let ::core::option::Option::Some(after) = value.after()? {
                        ::core::option::Option::Some(after.to_vec()?)
                    } else {
                        ::core::option::Option::None
                    },
                    before: if let ::core::option::Option::Some(before) = value.before()? {
                        ::core::option::Option::Some(before.to_vec()?)
                    } else {
                        ::core::option::Option::None
                    },
                    interval_ms: ::core::convert::TryInto::try_into(value.interval_ms()?)?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for SystemDeclRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for SystemDeclRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[SystemDeclRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<SystemDecl>> for SystemDecl {
            type Value = ::planus::Offset<SystemDecl>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<SystemDecl>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for SystemDeclRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[SystemDeclRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `ObserverDecl` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `ObserverDecl` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:135`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct ObserverDecl {
            /// The field `id` in the table `ObserverDecl`
            pub id: u32,
            /// The field `kind` in the table `ObserverDecl`
            pub kind: self::ObserverKind,
            /// The field `type_id` in the table `ObserverDecl`
            pub type_id: u16,
            /// The field `event_name` in the table `ObserverDecl`
            pub event_name: ::core::option::Option<::planus::alloc::string::String>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for ObserverDecl {
            fn default() -> Self {
                Self {
                    id: 0,
                    kind: self::ObserverKind::Spawn,
                    type_id: 0,
                    event_name: ::core::default::Default::default(),
                }
            }
        }

        impl ObserverDecl {
            /// Creates a [ObserverDeclBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> ObserverDeclBuilder<()> {
                ObserverDeclBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_id: impl ::planus::WriteAsDefault<u32, u32>,
                field_kind: impl ::planus::WriteAsDefault<self::ObserverKind, self::ObserverKind>,
                field_type_id: impl ::planus::WriteAsDefault<u16, u16>,
                field_event_name: impl ::planus::WriteAsOptional<
                    ::planus::Offset<::core::primitive::str>,
                >,
            ) -> ::planus::Offset<Self> {
                let prepared_id = field_id.prepare(builder, &0);
                let prepared_kind = field_kind.prepare(builder, &self::ObserverKind::Spawn);
                let prepared_type_id = field_type_id.prepare(builder, &0);
                let prepared_event_name = field_event_name.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<12> =
                    ::core::default::Default::default();
                if prepared_id.is_some() {
                    table_writer.write_entry::<u32>(0);
                }
                if prepared_event_name.is_some() {
                    table_writer.write_entry::<::planus::Offset<str>>(3);
                }
                if prepared_type_id.is_some() {
                    table_writer.write_entry::<u16>(2);
                }
                if prepared_kind.is_some() {
                    table_writer.write_entry::<self::ObserverKind>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_id) = prepared_id {
                            object_writer.write::<_, _, 4>(&prepared_id);
                        }
                        if let ::core::option::Option::Some(prepared_event_name) =
                            prepared_event_name
                        {
                            object_writer.write::<_, _, 4>(&prepared_event_name);
                        }
                        if let ::core::option::Option::Some(prepared_type_id) = prepared_type_id {
                            object_writer.write::<_, _, 2>(&prepared_type_id);
                        }
                        if let ::core::option::Option::Some(prepared_kind) = prepared_kind {
                            object_writer.write::<_, _, 1>(&prepared_kind);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<ObserverDecl>> for ObserverDecl {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverDecl> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<ObserverDecl>> for ObserverDecl {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ObserverDecl>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<ObserverDecl> for ObserverDecl {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverDecl> {
                ObserverDecl::create(builder, self.id, self.kind, self.type_id, &self.event_name)
            }
        }

        /// Builder for serializing an instance of the [ObserverDecl] type.
        ///
        /// Can be created using the [ObserverDecl::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct ObserverDeclBuilder<State>(State);

        impl ObserverDeclBuilder<()> {
            /// Setter for the [`id` field](ObserverDecl#structfield.id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn id<T0>(self, value: T0) -> ObserverDeclBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                ObserverDeclBuilder((value,))
            }

            /// Sets the [`id` field](ObserverDecl#structfield.id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn id_as_default(self) -> ObserverDeclBuilder<(::planus::DefaultValue,)> {
                self.id(::planus::DefaultValue)
            }
        }

        impl<T0> ObserverDeclBuilder<(T0,)> {
            /// Setter for the [`kind` field](ObserverDecl#structfield.kind).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn kind<T1>(self, value: T1) -> ObserverDeclBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsDefault<self::ObserverKind, self::ObserverKind>,
            {
                let (v0,) = self.0;
                ObserverDeclBuilder((v0, value))
            }

            /// Sets the [`kind` field](ObserverDecl#structfield.kind) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn kind_as_default(self) -> ObserverDeclBuilder<(T0, ::planus::DefaultValue)> {
                self.kind(::planus::DefaultValue)
            }
        }

        impl<T0, T1> ObserverDeclBuilder<(T0, T1)> {
            /// Setter for the [`type_id` field](ObserverDecl#structfield.type_id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_id<T2>(self, value: T2) -> ObserverDeclBuilder<(T0, T1, T2)>
            where
                T2: ::planus::WriteAsDefault<u16, u16>,
            {
                let (v0, v1) = self.0;
                ObserverDeclBuilder((v0, v1, value))
            }

            /// Sets the [`type_id` field](ObserverDecl#structfield.type_id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_id_as_default(
                self,
            ) -> ObserverDeclBuilder<(T0, T1, ::planus::DefaultValue)> {
                self.type_id(::planus::DefaultValue)
            }
        }

        impl<T0, T1, T2> ObserverDeclBuilder<(T0, T1, T2)> {
            /// Setter for the [`event_name` field](ObserverDecl#structfield.event_name).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn event_name<T3>(self, value: T3) -> ObserverDeclBuilder<(T0, T1, T2, T3)>
            where
                T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            {
                let (v0, v1, v2) = self.0;
                ObserverDeclBuilder((v0, v1, v2, value))
            }

            /// Sets the [`event_name` field](ObserverDecl#structfield.event_name) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn event_name_as_null(self) -> ObserverDeclBuilder<(T0, T1, T2, ())> {
                self.event_name(())
            }
        }

        impl<T0, T1, T2, T3> ObserverDeclBuilder<(T0, T1, T2, T3)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [ObserverDecl].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverDecl>
            where
                Self: ::planus::WriteAsOffset<ObserverDecl>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsDefault<self::ObserverKind, self::ObserverKind>,
                T2: ::planus::WriteAsDefault<u16, u16>,
                T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            > ::planus::WriteAs<::planus::Offset<ObserverDecl>>
            for ObserverDeclBuilder<(T0, T1, T2, T3)>
        {
            type Prepared = ::planus::Offset<ObserverDecl>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverDecl> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsDefault<self::ObserverKind, self::ObserverKind>,
                T2: ::planus::WriteAsDefault<u16, u16>,
                T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            > ::planus::WriteAsOptional<::planus::Offset<ObserverDecl>>
            for ObserverDeclBuilder<(T0, T1, T2, T3)>
        {
            type Prepared = ::planus::Offset<ObserverDecl>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ObserverDecl>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsDefault<self::ObserverKind, self::ObserverKind>,
                T2: ::planus::WriteAsDefault<u16, u16>,
                T3: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            > ::planus::WriteAsOffset<ObserverDecl> for ObserverDeclBuilder<(T0, T1, T2, T3)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverDecl> {
                let (v0, v1, v2, v3) = &self.0;
                ObserverDecl::create(builder, v0, v1, v2, v3)
            }
        }

        /// Reference to a deserialized [ObserverDecl].
        #[derive(Copy, Clone)]
        pub struct ObserverDeclRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> ObserverDeclRef<'a> {
            /// Getter for the [`id` field](ObserverDecl#structfield.id).
            #[inline]
            pub fn id(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(self.0.access(0, "ObserverDecl", "id")?.unwrap_or(0))
            }

            /// Getter for the [`kind` field](ObserverDecl#structfield.kind).
            #[inline]
            pub fn kind(&self) -> ::planus::Result<self::ObserverKind> {
                ::core::result::Result::Ok(
                    self.0
                        .access(1, "ObserverDecl", "kind")?
                        .unwrap_or(self::ObserverKind::Spawn),
                )
            }

            /// Getter for the [`type_id` field](ObserverDecl#structfield.type_id).
            #[inline]
            pub fn type_id(&self) -> ::planus::Result<u16> {
                ::core::result::Result::Ok(
                    self.0.access(2, "ObserverDecl", "type_id")?.unwrap_or(0),
                )
            }

            /// Getter for the [`event_name` field](ObserverDecl#structfield.event_name).
            #[inline]
            pub fn event_name(
                &self,
            ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>> {
                self.0.access(3, "ObserverDecl", "event_name")
            }
        }

        impl<'a> ::core::fmt::Debug for ObserverDeclRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("ObserverDeclRef");
                f.field("id", &self.id());
                f.field("kind", &self.kind());
                f.field("type_id", &self.type_id());
                if let ::core::option::Option::Some(field_event_name) =
                    self.event_name().transpose()
                {
                    f.field("event_name", &field_event_name);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<ObserverDeclRef<'a>> for ObserverDecl {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: ObserverDeclRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    id: ::core::convert::TryInto::try_into(value.id()?)?,
                    kind: ::core::convert::TryInto::try_into(value.kind()?)?,
                    type_id: ::core::convert::TryInto::try_into(value.type_id()?)?,
                    event_name: value.event_name()?.map(::core::convert::Into::into),
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for ObserverDeclRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for ObserverDeclRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[ObserverDeclRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<ObserverDecl>> for ObserverDecl {
            type Value = ::planus::Offset<ObserverDecl>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<ObserverDecl>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for ObserverDeclRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[ObserverDeclRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `SetupReply` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `SetupReply` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:142`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct SetupReply {
            /// The field `systems` in the table `SetupReply`
            pub systems: ::core::option::Option<::planus::alloc::vec::Vec<self::SystemDecl>>,
            /// The field `observers` in the table `SetupReply`
            pub observers: ::core::option::Option<::planus::alloc::vec::Vec<self::ObserverDecl>>,
            /// The field `wants_filter` in the table `SetupReply`
            pub wants_filter: bool,
            /// The field `wants_filter_out` in the table `SetupReply`
            pub wants_filter_out: bool,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for SetupReply {
            fn default() -> Self {
                Self {
                    systems: ::core::default::Default::default(),
                    observers: ::core::default::Default::default(),
                    wants_filter: false,
                    wants_filter_out: false,
                }
            }
        }

        impl SetupReply {
            /// Creates a [SetupReplyBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> SetupReplyBuilder<()> {
                SetupReplyBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_systems: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::SystemDecl>]>,
                >,
                field_observers: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::ObserverDecl>]>,
                >,
                field_wants_filter: impl ::planus::WriteAsDefault<bool, bool>,
                field_wants_filter_out: impl ::planus::WriteAsDefault<bool, bool>,
            ) -> ::planus::Offset<Self> {
                let prepared_systems = field_systems.prepare(builder);
                let prepared_observers = field_observers.prepare(builder);
                let prepared_wants_filter = field_wants_filter.prepare(builder, &false);
                let prepared_wants_filter_out = field_wants_filter_out.prepare(builder, &false);

                let mut table_writer: ::planus::table_writer::TableWriter<12> =
                    ::core::default::Default::default();
                if prepared_systems.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::SystemDecl>]>>(0);
                }
                if prepared_observers.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::ObserverDecl>]>>(1);
                }
                if prepared_wants_filter.is_some() {
                    table_writer.write_entry::<bool>(2);
                }
                if prepared_wants_filter_out.is_some() {
                    table_writer.write_entry::<bool>(3);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_systems) = prepared_systems {
                            object_writer.write::<_, _, 4>(&prepared_systems);
                        }
                        if let ::core::option::Option::Some(prepared_observers) = prepared_observers
                        {
                            object_writer.write::<_, _, 4>(&prepared_observers);
                        }
                        if let ::core::option::Option::Some(prepared_wants_filter) =
                            prepared_wants_filter
                        {
                            object_writer.write::<_, _, 1>(&prepared_wants_filter);
                        }
                        if let ::core::option::Option::Some(prepared_wants_filter_out) =
                            prepared_wants_filter_out
                        {
                            object_writer.write::<_, _, 1>(&prepared_wants_filter_out);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<SetupReply>> for SetupReply {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SetupReply> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<SetupReply>> for SetupReply {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SetupReply>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<SetupReply> for SetupReply {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SetupReply> {
                SetupReply::create(
                    builder,
                    &self.systems,
                    &self.observers,
                    self.wants_filter,
                    self.wants_filter_out,
                )
            }
        }

        /// Builder for serializing an instance of the [SetupReply] type.
        ///
        /// Can be created using the [SetupReply::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct SetupReplyBuilder<State>(State);

        impl SetupReplyBuilder<()> {
            /// Setter for the [`systems` field](SetupReply#structfield.systems).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn systems<T0>(self, value: T0) -> SetupReplyBuilder<(T0,)>
            where
                T0: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::SystemDecl>]>,
                >,
            {
                SetupReplyBuilder((value,))
            }

            /// Sets the [`systems` field](SetupReply#structfield.systems) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn systems_as_null(self) -> SetupReplyBuilder<((),)> {
                self.systems(())
            }
        }

        impl<T0> SetupReplyBuilder<(T0,)> {
            /// Setter for the [`observers` field](SetupReply#structfield.observers).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn observers<T1>(self, value: T1) -> SetupReplyBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::ObserverDecl>]>,
                >,
            {
                let (v0,) = self.0;
                SetupReplyBuilder((v0, value))
            }

            /// Sets the [`observers` field](SetupReply#structfield.observers) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn observers_as_null(self) -> SetupReplyBuilder<(T0, ())> {
                self.observers(())
            }
        }

        impl<T0, T1> SetupReplyBuilder<(T0, T1)> {
            /// Setter for the [`wants_filter` field](SetupReply#structfield.wants_filter).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn wants_filter<T2>(self, value: T2) -> SetupReplyBuilder<(T0, T1, T2)>
            where
                T2: ::planus::WriteAsDefault<bool, bool>,
            {
                let (v0, v1) = self.0;
                SetupReplyBuilder((v0, v1, value))
            }

            /// Sets the [`wants_filter` field](SetupReply#structfield.wants_filter) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn wants_filter_as_default(
                self,
            ) -> SetupReplyBuilder<(T0, T1, ::planus::DefaultValue)> {
                self.wants_filter(::planus::DefaultValue)
            }
        }

        impl<T0, T1, T2> SetupReplyBuilder<(T0, T1, T2)> {
            /// Setter for the [`wants_filter_out` field](SetupReply#structfield.wants_filter_out).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn wants_filter_out<T3>(self, value: T3) -> SetupReplyBuilder<(T0, T1, T2, T3)>
            where
                T3: ::planus::WriteAsDefault<bool, bool>,
            {
                let (v0, v1, v2) = self.0;
                SetupReplyBuilder((v0, v1, v2, value))
            }

            /// Sets the [`wants_filter_out` field](SetupReply#structfield.wants_filter_out) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn wants_filter_out_as_default(
                self,
            ) -> SetupReplyBuilder<(T0, T1, T2, ::planus::DefaultValue)> {
                self.wants_filter_out(::planus::DefaultValue)
            }
        }

        impl<T0, T1, T2, T3> SetupReplyBuilder<(T0, T1, T2, T3)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [SetupReply].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<SetupReply>
            where
                Self: ::planus::WriteAsOffset<SetupReply>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::SystemDecl>]>>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::ObserverDecl>]>>,
                T2: ::planus::WriteAsDefault<bool, bool>,
                T3: ::planus::WriteAsDefault<bool, bool>,
            > ::planus::WriteAs<::planus::Offset<SetupReply>>
            for SetupReplyBuilder<(T0, T1, T2, T3)>
        {
            type Prepared = ::planus::Offset<SetupReply>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SetupReply> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::SystemDecl>]>>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::ObserverDecl>]>>,
                T2: ::planus::WriteAsDefault<bool, bool>,
                T3: ::planus::WriteAsDefault<bool, bool>,
            > ::planus::WriteAsOptional<::planus::Offset<SetupReply>>
            for SetupReplyBuilder<(T0, T1, T2, T3)>
        {
            type Prepared = ::planus::Offset<SetupReply>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SetupReply>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::SystemDecl>]>>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::ObserverDecl>]>>,
                T2: ::planus::WriteAsDefault<bool, bool>,
                T3: ::planus::WriteAsDefault<bool, bool>,
            > ::planus::WriteAsOffset<SetupReply> for SetupReplyBuilder<(T0, T1, T2, T3)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SetupReply> {
                let (v0, v1, v2, v3) = &self.0;
                SetupReply::create(builder, v0, v1, v2, v3)
            }
        }

        /// Reference to a deserialized [SetupReply].
        #[derive(Copy, Clone)]
        pub struct SetupReplyRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> SetupReplyRef<'a> {
            /// Getter for the [`systems` field](SetupReply#structfield.systems).
            #[inline]
            pub fn systems(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::SystemDeclRef<'a>>>,
                >,
            > {
                self.0.access(0, "SetupReply", "systems")
            }

            /// Getter for the [`observers` field](SetupReply#structfield.observers).
            #[inline]
            pub fn observers(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::ObserverDeclRef<'a>>>,
                >,
            > {
                self.0.access(1, "SetupReply", "observers")
            }

            /// Getter for the [`wants_filter` field](SetupReply#structfield.wants_filter).
            #[inline]
            pub fn wants_filter(&self) -> ::planus::Result<bool> {
                ::core::result::Result::Ok(
                    self.0
                        .access(2, "SetupReply", "wants_filter")?
                        .unwrap_or(false),
                )
            }

            /// Getter for the [`wants_filter_out` field](SetupReply#structfield.wants_filter_out).
            #[inline]
            pub fn wants_filter_out(&self) -> ::planus::Result<bool> {
                ::core::result::Result::Ok(
                    self.0
                        .access(3, "SetupReply", "wants_filter_out")?
                        .unwrap_or(false),
                )
            }
        }

        impl<'a> ::core::fmt::Debug for SetupReplyRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("SetupReplyRef");
                if let ::core::option::Option::Some(field_systems) = self.systems().transpose() {
                    f.field("systems", &field_systems);
                }
                if let ::core::option::Option::Some(field_observers) = self.observers().transpose()
                {
                    f.field("observers", &field_observers);
                }
                f.field("wants_filter", &self.wants_filter());
                f.field("wants_filter_out", &self.wants_filter_out());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<SetupReplyRef<'a>> for SetupReply {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: SetupReplyRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    systems: if let ::core::option::Option::Some(systems) = value.systems()? {
                        ::core::option::Option::Some(systems.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                    observers: if let ::core::option::Option::Some(observers) = value.observers()? {
                        ::core::option::Option::Some(observers.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                    wants_filter: ::core::convert::TryInto::try_into(value.wants_filter()?)?,
                    wants_filter_out: ::core::convert::TryInto::try_into(
                        value.wants_filter_out()?,
                    )?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for SetupReplyRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for SetupReplyRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[SetupReplyRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<SetupReply>> for SetupReply {
            type Value = ::planus::Offset<SetupReply>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<SetupReply>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for SetupReplyRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[SetupReplyRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `Row` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `Row` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:152`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct Row {
            /// The field `entity` in the table `Row`
            pub entity: u64,
            /// The field `comps` in the table `Row`
            pub comps: ::core::option::Option<::planus::alloc::vec::Vec<self::CompValue>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for Row {
            fn default() -> Self {
                Self {
                    entity: 0,
                    comps: ::core::default::Default::default(),
                }
            }
        }

        impl Row {
            /// Creates a [RowBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> RowBuilder<()> {
                RowBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_entity: impl ::planus::WriteAsDefault<u64, u64>,
                field_comps: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::CompValue>]>,
                >,
            ) -> ::planus::Offset<Self> {
                let prepared_entity = field_entity.prepare(builder, &0);
                let prepared_comps = field_comps.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_entity.is_some() {
                    table_writer.write_entry::<u64>(0);
                }
                if prepared_comps.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::CompValue>]>>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_entity) = prepared_entity {
                            object_writer.write::<_, _, 8>(&prepared_entity);
                        }
                        if let ::core::option::Option::Some(prepared_comps) = prepared_comps {
                            object_writer.write::<_, _, 4>(&prepared_comps);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<Row>> for Row {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Row> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<Row>> for Row {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<Row>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<Row> for Row {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Row> {
                Row::create(builder, self.entity, &self.comps)
            }
        }

        /// Builder for serializing an instance of the [Row] type.
        ///
        /// Can be created using the [Row::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct RowBuilder<State>(State);

        impl RowBuilder<()> {
            /// Setter for the [`entity` field](Row#structfield.entity).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity<T0>(self, value: T0) -> RowBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u64, u64>,
            {
                RowBuilder((value,))
            }

            /// Sets the [`entity` field](Row#structfield.entity) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity_as_default(self) -> RowBuilder<(::planus::DefaultValue,)> {
                self.entity(::planus::DefaultValue)
            }
        }

        impl<T0> RowBuilder<(T0,)> {
            /// Setter for the [`comps` field](Row#structfield.comps).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn comps<T1>(self, value: T1) -> RowBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::CompValue>]>,
                >,
            {
                let (v0,) = self.0;
                RowBuilder((v0, value))
            }

            /// Sets the [`comps` field](Row#structfield.comps) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn comps_as_null(self) -> RowBuilder<(T0, ())> {
                self.comps(())
            }
        }

        impl<T0, T1> RowBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [Row].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<Row>
            where
                Self: ::planus::WriteAsOffset<Row>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u64, u64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAs<::planus::Offset<Row>> for RowBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<Row>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Row> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u64, u64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAsOptional<::planus::Offset<Row>> for RowBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<Row>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<Row>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u64, u64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAsOffset<Row> for RowBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<Row> {
                let (v0, v1) = &self.0;
                Row::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [Row].
        #[derive(Copy, Clone)]
        pub struct RowRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> RowRef<'a> {
            /// Getter for the [`entity` field](Row#structfield.entity).
            #[inline]
            pub fn entity(&self) -> ::planus::Result<u64> {
                ::core::result::Result::Ok(self.0.access(0, "Row", "entity")?.unwrap_or(0))
            }

            /// Getter for the [`comps` field](Row#structfield.comps).
            #[inline]
            pub fn comps(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::CompValueRef<'a>>>,
                >,
            > {
                self.0.access(1, "Row", "comps")
            }
        }

        impl<'a> ::core::fmt::Debug for RowRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("RowRef");
                f.field("entity", &self.entity());
                if let ::core::option::Option::Some(field_comps) = self.comps().transpose() {
                    f.field("comps", &field_comps);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<RowRef<'a>> for Row {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: RowRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    entity: ::core::convert::TryInto::try_into(value.entity()?)?,
                    comps: if let ::core::option::Option::Some(comps) = value.comps()? {
                        ::core::option::Option::Some(comps.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for RowRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for RowRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location("[RowRef]", "get", buffer.offset_from_start)
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<Row>> for Row {
            type Value = ::planus::Offset<Row>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<Row>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for RowRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| error_kind.with_error_location("[RowRef]", "read_as_root", 0))
            }
        }

        /// The table `QueryRows` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `QueryRows` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:157`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct QueryRows {
            /// The field `param_index` in the table `QueryRows`
            pub param_index: u32,
            /// The field `rows` in the table `QueryRows`
            pub rows: ::core::option::Option<::planus::alloc::vec::Vec<self::Row>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for QueryRows {
            fn default() -> Self {
                Self {
                    param_index: 0,
                    rows: ::core::default::Default::default(),
                }
            }
        }

        impl QueryRows {
            /// Creates a [QueryRowsBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> QueryRowsBuilder<()> {
                QueryRowsBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_param_index: impl ::planus::WriteAsDefault<u32, u32>,
                field_rows: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::Row>]>,
                >,
            ) -> ::planus::Offset<Self> {
                let prepared_param_index = field_param_index.prepare(builder, &0);
                let prepared_rows = field_rows.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_param_index.is_some() {
                    table_writer.write_entry::<u32>(0);
                }
                if prepared_rows.is_some() {
                    table_writer.write_entry::<::planus::Offset<[::planus::Offset<self::Row>]>>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_param_index) =
                            prepared_param_index
                        {
                            object_writer.write::<_, _, 4>(&prepared_param_index);
                        }
                        if let ::core::option::Option::Some(prepared_rows) = prepared_rows {
                            object_writer.write::<_, _, 4>(&prepared_rows);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<QueryRows>> for QueryRows {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryRows> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<QueryRows>> for QueryRows {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<QueryRows>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<QueryRows> for QueryRows {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryRows> {
                QueryRows::create(builder, self.param_index, &self.rows)
            }
        }

        /// Builder for serializing an instance of the [QueryRows] type.
        ///
        /// Can be created using the [QueryRows::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct QueryRowsBuilder<State>(State);

        impl QueryRowsBuilder<()> {
            /// Setter for the [`param_index` field](QueryRows#structfield.param_index).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn param_index<T0>(self, value: T0) -> QueryRowsBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                QueryRowsBuilder((value,))
            }

            /// Sets the [`param_index` field](QueryRows#structfield.param_index) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn param_index_as_default(self) -> QueryRowsBuilder<(::planus::DefaultValue,)> {
                self.param_index(::planus::DefaultValue)
            }
        }

        impl<T0> QueryRowsBuilder<(T0,)> {
            /// Setter for the [`rows` field](QueryRows#structfield.rows).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn rows<T1>(self, value: T1) -> QueryRowsBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Row>]>>,
            {
                let (v0,) = self.0;
                QueryRowsBuilder((v0, value))
            }

            /// Sets the [`rows` field](QueryRows#structfield.rows) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn rows_as_null(self) -> QueryRowsBuilder<(T0, ())> {
                self.rows(())
            }
        }

        impl<T0, T1> QueryRowsBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [QueryRows].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryRows>
            where
                Self: ::planus::WriteAsOffset<QueryRows>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Row>]>>,
            > ::planus::WriteAs<::planus::Offset<QueryRows>> for QueryRowsBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<QueryRows>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryRows> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Row>]>>,
            > ::planus::WriteAsOptional<::planus::Offset<QueryRows>>
            for QueryRowsBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<QueryRows>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<QueryRows>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::Row>]>>,
            > ::planus::WriteAsOffset<QueryRows> for QueryRowsBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<QueryRows> {
                let (v0, v1) = &self.0;
                QueryRows::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [QueryRows].
        #[derive(Copy, Clone)]
        pub struct QueryRowsRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> QueryRowsRef<'a> {
            /// Getter for the [`param_index` field](QueryRows#structfield.param_index).
            #[inline]
            pub fn param_index(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(
                    self.0.access(0, "QueryRows", "param_index")?.unwrap_or(0),
                )
            }

            /// Getter for the [`rows` field](QueryRows#structfield.rows).
            #[inline]
            pub fn rows(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<::planus::Vector<'a, ::planus::Result<self::RowRef<'a>>>>,
            > {
                self.0.access(1, "QueryRows", "rows")
            }
        }

        impl<'a> ::core::fmt::Debug for QueryRowsRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("QueryRowsRef");
                f.field("param_index", &self.param_index());
                if let ::core::option::Option::Some(field_rows) = self.rows().transpose() {
                    f.field("rows", &field_rows);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<QueryRowsRef<'a>> for QueryRows {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: QueryRowsRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    param_index: ::core::convert::TryInto::try_into(value.param_index()?)?,
                    rows: if let ::core::option::Option::Some(rows) = value.rows()? {
                        ::core::option::Option::Some(rows.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for QueryRowsRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for QueryRowsRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[QueryRowsRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<QueryRows>> for QueryRows {
            type Value = ::planus::Offset<QueryRows>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<QueryRows>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for QueryRowsRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[QueryRowsRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `SystemInput` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `SystemInput` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:162`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct SystemInput {
            /// The field `sys_id` in the table `SystemInput`
            pub sys_id: u32,
            /// The field `queries` in the table `SystemInput`
            pub queries: ::core::option::Option<::planus::alloc::vec::Vec<self::QueryRows>>,
            /// The field `tick` in the table `SystemInput`
            pub tick: u64,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for SystemInput {
            fn default() -> Self {
                Self {
                    sys_id: 0,
                    queries: ::core::default::Default::default(),
                    tick: 0,
                }
            }
        }

        impl SystemInput {
            /// Creates a [SystemInputBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> SystemInputBuilder<()> {
                SystemInputBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_sys_id: impl ::planus::WriteAsDefault<u32, u32>,
                field_queries: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::QueryRows>]>,
                >,
                field_tick: impl ::planus::WriteAsDefault<u64, u64>,
            ) -> ::planus::Offset<Self> {
                let prepared_sys_id = field_sys_id.prepare(builder, &0);
                let prepared_queries = field_queries.prepare(builder);
                let prepared_tick = field_tick.prepare(builder, &0);

                let mut table_writer: ::planus::table_writer::TableWriter<10> =
                    ::core::default::Default::default();
                if prepared_tick.is_some() {
                    table_writer.write_entry::<u64>(2);
                }
                if prepared_sys_id.is_some() {
                    table_writer.write_entry::<u32>(0);
                }
                if prepared_queries.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::QueryRows>]>>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_tick) = prepared_tick {
                            object_writer.write::<_, _, 8>(&prepared_tick);
                        }
                        if let ::core::option::Option::Some(prepared_sys_id) = prepared_sys_id {
                            object_writer.write::<_, _, 4>(&prepared_sys_id);
                        }
                        if let ::core::option::Option::Some(prepared_queries) = prepared_queries {
                            object_writer.write::<_, _, 4>(&prepared_queries);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<SystemInput>> for SystemInput {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemInput> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<SystemInput>> for SystemInput {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SystemInput>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<SystemInput> for SystemInput {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemInput> {
                SystemInput::create(builder, self.sys_id, &self.queries, self.tick)
            }
        }

        /// Builder for serializing an instance of the [SystemInput] type.
        ///
        /// Can be created using the [SystemInput::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct SystemInputBuilder<State>(State);

        impl SystemInputBuilder<()> {
            /// Setter for the [`sys_id` field](SystemInput#structfield.sys_id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn sys_id<T0>(self, value: T0) -> SystemInputBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                SystemInputBuilder((value,))
            }

            /// Sets the [`sys_id` field](SystemInput#structfield.sys_id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn sys_id_as_default(self) -> SystemInputBuilder<(::planus::DefaultValue,)> {
                self.sys_id(::planus::DefaultValue)
            }
        }

        impl<T0> SystemInputBuilder<(T0,)> {
            /// Setter for the [`queries` field](SystemInput#structfield.queries).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn queries<T1>(self, value: T1) -> SystemInputBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::QueryRows>]>,
                >,
            {
                let (v0,) = self.0;
                SystemInputBuilder((v0, value))
            }

            /// Sets the [`queries` field](SystemInput#structfield.queries) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn queries_as_null(self) -> SystemInputBuilder<(T0, ())> {
                self.queries(())
            }
        }

        impl<T0, T1> SystemInputBuilder<(T0, T1)> {
            /// Setter for the [`tick` field](SystemInput#structfield.tick).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn tick<T2>(self, value: T2) -> SystemInputBuilder<(T0, T1, T2)>
            where
                T2: ::planus::WriteAsDefault<u64, u64>,
            {
                let (v0, v1) = self.0;
                SystemInputBuilder((v0, v1, value))
            }

            /// Sets the [`tick` field](SystemInput#structfield.tick) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn tick_as_default(self) -> SystemInputBuilder<(T0, T1, ::planus::DefaultValue)> {
                self.tick(::planus::DefaultValue)
            }
        }

        impl<T0, T1, T2> SystemInputBuilder<(T0, T1, T2)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [SystemInput].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemInput>
            where
                Self: ::planus::WriteAsOffset<SystemInput>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::QueryRows>]>>,
                T2: ::planus::WriteAsDefault<u64, u64>,
            > ::planus::WriteAs<::planus::Offset<SystemInput>>
            for SystemInputBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<SystemInput>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemInput> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::QueryRows>]>>,
                T2: ::planus::WriteAsDefault<u64, u64>,
            > ::planus::WriteAsOptional<::planus::Offset<SystemInput>>
            for SystemInputBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<SystemInput>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SystemInput>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::QueryRows>]>>,
                T2: ::planus::WriteAsDefault<u64, u64>,
            > ::planus::WriteAsOffset<SystemInput> for SystemInputBuilder<(T0, T1, T2)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SystemInput> {
                let (v0, v1, v2) = &self.0;
                SystemInput::create(builder, v0, v1, v2)
            }
        }

        /// Reference to a deserialized [SystemInput].
        #[derive(Copy, Clone)]
        pub struct SystemInputRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> SystemInputRef<'a> {
            /// Getter for the [`sys_id` field](SystemInput#structfield.sys_id).
            #[inline]
            pub fn sys_id(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(self.0.access(0, "SystemInput", "sys_id")?.unwrap_or(0))
            }

            /// Getter for the [`queries` field](SystemInput#structfield.queries).
            #[inline]
            pub fn queries(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::QueryRowsRef<'a>>>,
                >,
            > {
                self.0.access(1, "SystemInput", "queries")
            }

            /// Getter for the [`tick` field](SystemInput#structfield.tick).
            #[inline]
            pub fn tick(&self) -> ::planus::Result<u64> {
                ::core::result::Result::Ok(self.0.access(2, "SystemInput", "tick")?.unwrap_or(0))
            }
        }

        impl<'a> ::core::fmt::Debug for SystemInputRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("SystemInputRef");
                f.field("sys_id", &self.sys_id());
                if let ::core::option::Option::Some(field_queries) = self.queries().transpose() {
                    f.field("queries", &field_queries);
                }
                f.field("tick", &self.tick());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<SystemInputRef<'a>> for SystemInput {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: SystemInputRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    sys_id: ::core::convert::TryInto::try_into(value.sys_id()?)?,
                    queries: if let ::core::option::Option::Some(queries) = value.queries()? {
                        ::core::option::Option::Some(queries.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                    tick: ::core::convert::TryInto::try_into(value.tick()?)?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for SystemInputRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for SystemInputRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[SystemInputRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<SystemInput>> for SystemInput {
            type Value = ::planus::Offset<SystemInput>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<SystemInput>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for SystemInputRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[SystemInputRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `SpawnCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `SpawnCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:177`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct SpawnCmd {
            /// The field `temp_id` in the table `SpawnCmd`
            pub temp_id: u32,
            /// The field `comps` in the table `SpawnCmd`
            pub comps: ::core::option::Option<::planus::alloc::vec::Vec<self::CompValue>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for SpawnCmd {
            fn default() -> Self {
                Self {
                    temp_id: 0,
                    comps: ::core::default::Default::default(),
                }
            }
        }

        impl SpawnCmd {
            /// Creates a [SpawnCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> SpawnCmdBuilder<()> {
                SpawnCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_temp_id: impl ::planus::WriteAsDefault<u32, u32>,
                field_comps: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::CompValue>]>,
                >,
            ) -> ::planus::Offset<Self> {
                let prepared_temp_id = field_temp_id.prepare(builder, &0);
                let prepared_comps = field_comps.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_temp_id.is_some() {
                    table_writer.write_entry::<u32>(0);
                }
                if prepared_comps.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::CompValue>]>>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_temp_id) = prepared_temp_id {
                            object_writer.write::<_, _, 4>(&prepared_temp_id);
                        }
                        if let ::core::option::Option::Some(prepared_comps) = prepared_comps {
                            object_writer.write::<_, _, 4>(&prepared_comps);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<SpawnCmd>> for SpawnCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<SpawnCmd>> for SpawnCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SpawnCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<SpawnCmd> for SpawnCmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnCmd> {
                SpawnCmd::create(builder, self.temp_id, &self.comps)
            }
        }

        /// Builder for serializing an instance of the [SpawnCmd] type.
        ///
        /// Can be created using the [SpawnCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct SpawnCmdBuilder<State>(State);

        impl SpawnCmdBuilder<()> {
            /// Setter for the [`temp_id` field](SpawnCmd#structfield.temp_id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn temp_id<T0>(self, value: T0) -> SpawnCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                SpawnCmdBuilder((value,))
            }

            /// Sets the [`temp_id` field](SpawnCmd#structfield.temp_id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn temp_id_as_default(self) -> SpawnCmdBuilder<(::planus::DefaultValue,)> {
                self.temp_id(::planus::DefaultValue)
            }
        }

        impl<T0> SpawnCmdBuilder<(T0,)> {
            /// Setter for the [`comps` field](SpawnCmd#structfield.comps).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn comps<T1>(self, value: T1) -> SpawnCmdBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::CompValue>]>,
                >,
            {
                let (v0,) = self.0;
                SpawnCmdBuilder((v0, value))
            }

            /// Sets the [`comps` field](SpawnCmd#structfield.comps) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn comps_as_null(self) -> SpawnCmdBuilder<(T0, ())> {
                self.comps(())
            }
        }

        impl<T0, T1> SpawnCmdBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [SpawnCmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnCmd>
            where
                Self: ::planus::WriteAsOffset<SpawnCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAs<::planus::Offset<SpawnCmd>> for SpawnCmdBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<SpawnCmd>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAsOptional<::planus::Offset<SpawnCmd>> for SpawnCmdBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<SpawnCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SpawnCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAsOffset<SpawnCmd> for SpawnCmdBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnCmd> {
                let (v0, v1) = &self.0;
                SpawnCmd::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [SpawnCmd].
        #[derive(Copy, Clone)]
        pub struct SpawnCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> SpawnCmdRef<'a> {
            /// Getter for the [`temp_id` field](SpawnCmd#structfield.temp_id).
            #[inline]
            pub fn temp_id(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(self.0.access(0, "SpawnCmd", "temp_id")?.unwrap_or(0))
            }

            /// Getter for the [`comps` field](SpawnCmd#structfield.comps).
            #[inline]
            pub fn comps(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::CompValueRef<'a>>>,
                >,
            > {
                self.0.access(1, "SpawnCmd", "comps")
            }
        }

        impl<'a> ::core::fmt::Debug for SpawnCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("SpawnCmdRef");
                f.field("temp_id", &self.temp_id());
                if let ::core::option::Option::Some(field_comps) = self.comps().transpose() {
                    f.field("comps", &field_comps);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<SpawnCmdRef<'a>> for SpawnCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: SpawnCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    temp_id: ::core::convert::TryInto::try_into(value.temp_id()?)?,
                    comps: if let ::core::option::Option::Some(comps) = value.comps()? {
                        ::core::option::Option::Some(comps.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for SpawnCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for SpawnCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location("[SpawnCmdRef]", "get", buffer.offset_from_start)
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<SpawnCmd>> for SpawnCmd {
            type Value = ::planus::Offset<SpawnCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<SpawnCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for SpawnCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[SpawnCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `InsertCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `InsertCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:182`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct InsertCmd {
            /// The field `entity` in the table `InsertCmd`
            pub entity: i64,
            /// The field `comps` in the table `InsertCmd`
            pub comps: ::core::option::Option<::planus::alloc::vec::Vec<self::CompValue>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for InsertCmd {
            fn default() -> Self {
                Self {
                    entity: 0,
                    comps: ::core::default::Default::default(),
                }
            }
        }

        impl InsertCmd {
            /// Creates a [InsertCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> InsertCmdBuilder<()> {
                InsertCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_entity: impl ::planus::WriteAsDefault<i64, i64>,
                field_comps: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::CompValue>]>,
                >,
            ) -> ::planus::Offset<Self> {
                let prepared_entity = field_entity.prepare(builder, &0);
                let prepared_comps = field_comps.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_entity.is_some() {
                    table_writer.write_entry::<i64>(0);
                }
                if prepared_comps.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::CompValue>]>>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_entity) = prepared_entity {
                            object_writer.write::<_, _, 8>(&prepared_entity);
                        }
                        if let ::core::option::Option::Some(prepared_comps) = prepared_comps {
                            object_writer.write::<_, _, 4>(&prepared_comps);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<InsertCmd>> for InsertCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<InsertCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<InsertCmd>> for InsertCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<InsertCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<InsertCmd> for InsertCmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<InsertCmd> {
                InsertCmd::create(builder, self.entity, &self.comps)
            }
        }

        /// Builder for serializing an instance of the [InsertCmd] type.
        ///
        /// Can be created using the [InsertCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct InsertCmdBuilder<State>(State);

        impl InsertCmdBuilder<()> {
            /// Setter for the [`entity` field](InsertCmd#structfield.entity).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity<T0>(self, value: T0) -> InsertCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<i64, i64>,
            {
                InsertCmdBuilder((value,))
            }

            /// Sets the [`entity` field](InsertCmd#structfield.entity) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity_as_default(self) -> InsertCmdBuilder<(::planus::DefaultValue,)> {
                self.entity(::planus::DefaultValue)
            }
        }

        impl<T0> InsertCmdBuilder<(T0,)> {
            /// Setter for the [`comps` field](InsertCmd#structfield.comps).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn comps<T1>(self, value: T1) -> InsertCmdBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::CompValue>]>,
                >,
            {
                let (v0,) = self.0;
                InsertCmdBuilder((v0, value))
            }

            /// Sets the [`comps` field](InsertCmd#structfield.comps) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn comps_as_null(self) -> InsertCmdBuilder<(T0, ())> {
                self.comps(())
            }
        }

        impl<T0, T1> InsertCmdBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [InsertCmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<InsertCmd>
            where
                Self: ::planus::WriteAsOffset<InsertCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAs<::planus::Offset<InsertCmd>> for InsertCmdBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<InsertCmd>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<InsertCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAsOptional<::planus::Offset<InsertCmd>>
            for InsertCmdBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<InsertCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<InsertCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[::planus::Offset<self::CompValue>]>>,
            > ::planus::WriteAsOffset<InsertCmd> for InsertCmdBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<InsertCmd> {
                let (v0, v1) = &self.0;
                InsertCmd::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [InsertCmd].
        #[derive(Copy, Clone)]
        pub struct InsertCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> InsertCmdRef<'a> {
            /// Getter for the [`entity` field](InsertCmd#structfield.entity).
            #[inline]
            pub fn entity(&self) -> ::planus::Result<i64> {
                ::core::result::Result::Ok(self.0.access(0, "InsertCmd", "entity")?.unwrap_or(0))
            }

            /// Getter for the [`comps` field](InsertCmd#structfield.comps).
            #[inline]
            pub fn comps(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::CompValueRef<'a>>>,
                >,
            > {
                self.0.access(1, "InsertCmd", "comps")
            }
        }

        impl<'a> ::core::fmt::Debug for InsertCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("InsertCmdRef");
                f.field("entity", &self.entity());
                if let ::core::option::Option::Some(field_comps) = self.comps().transpose() {
                    f.field("comps", &field_comps);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<InsertCmdRef<'a>> for InsertCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: InsertCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    entity: ::core::convert::TryInto::try_into(value.entity()?)?,
                    comps: if let ::core::option::Option::Some(comps) = value.comps()? {
                        ::core::option::Option::Some(comps.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for InsertCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for InsertCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[InsertCmdRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<InsertCmd>> for InsertCmd {
            type Value = ::planus::Offset<InsertCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<InsertCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for InsertCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[InsertCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `RemoveCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `RemoveCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:187`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct RemoveCmd {
            /// The field `entity` in the table `RemoveCmd`
            pub entity: i64,
            /// The field `type_ids` in the table `RemoveCmd`
            pub type_ids: ::core::option::Option<::planus::alloc::vec::Vec<u16>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for RemoveCmd {
            fn default() -> Self {
                Self {
                    entity: 0,
                    type_ids: ::core::default::Default::default(),
                }
            }
        }

        impl RemoveCmd {
            /// Creates a [RemoveCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> RemoveCmdBuilder<()> {
                RemoveCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_entity: impl ::planus::WriteAsDefault<i64, i64>,
                field_type_ids: impl ::planus::WriteAsOptional<::planus::Offset<[u16]>>,
            ) -> ::planus::Offset<Self> {
                let prepared_entity = field_entity.prepare(builder, &0);
                let prepared_type_ids = field_type_ids.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_entity.is_some() {
                    table_writer.write_entry::<i64>(0);
                }
                if prepared_type_ids.is_some() {
                    table_writer.write_entry::<::planus::Offset<[u16]>>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_entity) = prepared_entity {
                            object_writer.write::<_, _, 8>(&prepared_entity);
                        }
                        if let ::core::option::Option::Some(prepared_type_ids) = prepared_type_ids {
                            object_writer.write::<_, _, 4>(&prepared_type_ids);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<RemoveCmd>> for RemoveCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<RemoveCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<RemoveCmd>> for RemoveCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<RemoveCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<RemoveCmd> for RemoveCmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<RemoveCmd> {
                RemoveCmd::create(builder, self.entity, &self.type_ids)
            }
        }

        /// Builder for serializing an instance of the [RemoveCmd] type.
        ///
        /// Can be created using the [RemoveCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct RemoveCmdBuilder<State>(State);

        impl RemoveCmdBuilder<()> {
            /// Setter for the [`entity` field](RemoveCmd#structfield.entity).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity<T0>(self, value: T0) -> RemoveCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<i64, i64>,
            {
                RemoveCmdBuilder((value,))
            }

            /// Sets the [`entity` field](RemoveCmd#structfield.entity) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity_as_default(self) -> RemoveCmdBuilder<(::planus::DefaultValue,)> {
                self.entity(::planus::DefaultValue)
            }
        }

        impl<T0> RemoveCmdBuilder<(T0,)> {
            /// Setter for the [`type_ids` field](RemoveCmd#structfield.type_ids).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_ids<T1>(self, value: T1) -> RemoveCmdBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsOptional<::planus::Offset<[u16]>>,
            {
                let (v0,) = self.0;
                RemoveCmdBuilder((v0, value))
            }

            /// Sets the [`type_ids` field](RemoveCmd#structfield.type_ids) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn type_ids_as_null(self) -> RemoveCmdBuilder<(T0, ())> {
                self.type_ids(())
            }
        }

        impl<T0, T1> RemoveCmdBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [RemoveCmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<RemoveCmd>
            where
                Self: ::planus::WriteAsOffset<RemoveCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[u16]>>,
            > ::planus::WriteAs<::planus::Offset<RemoveCmd>> for RemoveCmdBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<RemoveCmd>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<RemoveCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[u16]>>,
            > ::planus::WriteAsOptional<::planus::Offset<RemoveCmd>>
            for RemoveCmdBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<RemoveCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<RemoveCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsOptional<::planus::Offset<[u16]>>,
            > ::planus::WriteAsOffset<RemoveCmd> for RemoveCmdBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<RemoveCmd> {
                let (v0, v1) = &self.0;
                RemoveCmd::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [RemoveCmd].
        #[derive(Copy, Clone)]
        pub struct RemoveCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> RemoveCmdRef<'a> {
            /// Getter for the [`entity` field](RemoveCmd#structfield.entity).
            #[inline]
            pub fn entity(&self) -> ::planus::Result<i64> {
                ::core::result::Result::Ok(self.0.access(0, "RemoveCmd", "entity")?.unwrap_or(0))
            }

            /// Getter for the [`type_ids` field](RemoveCmd#structfield.type_ids).
            #[inline]
            pub fn type_ids(
                &self,
            ) -> ::planus::Result<::core::option::Option<::planus::Vector<'a, u16>>> {
                self.0.access(1, "RemoveCmd", "type_ids")
            }
        }

        impl<'a> ::core::fmt::Debug for RemoveCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("RemoveCmdRef");
                f.field("entity", &self.entity());
                if let ::core::option::Option::Some(field_type_ids) = self.type_ids().transpose() {
                    f.field("type_ids", &field_type_ids);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<RemoveCmdRef<'a>> for RemoveCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: RemoveCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    entity: ::core::convert::TryInto::try_into(value.entity()?)?,
                    type_ids: if let ::core::option::Option::Some(type_ids) = value.type_ids()? {
                        ::core::option::Option::Some(type_ids.to_vec()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for RemoveCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for RemoveCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[RemoveCmdRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<RemoveCmd>> for RemoveCmd {
            type Value = ::planus::Offset<RemoveCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<RemoveCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for RemoveCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[RemoveCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `DespawnCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `DespawnCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:192`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct DespawnCmd {
            /// The field `entity` in the table `DespawnCmd`
            pub entity: i64,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for DespawnCmd {
            fn default() -> Self {
                Self { entity: 0 }
            }
        }

        impl DespawnCmd {
            /// Creates a [DespawnCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> DespawnCmdBuilder<()> {
                DespawnCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_entity: impl ::planus::WriteAsDefault<i64, i64>,
            ) -> ::planus::Offset<Self> {
                let prepared_entity = field_entity.prepare(builder, &0);

                let mut table_writer: ::planus::table_writer::TableWriter<6> =
                    ::core::default::Default::default();
                if prepared_entity.is_some() {
                    table_writer.write_entry::<i64>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_entity) = prepared_entity {
                            object_writer.write::<_, _, 8>(&prepared_entity);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<DespawnCmd>> for DespawnCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DespawnCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<DespawnCmd>> for DespawnCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<DespawnCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<DespawnCmd> for DespawnCmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DespawnCmd> {
                DespawnCmd::create(builder, self.entity)
            }
        }

        /// Builder for serializing an instance of the [DespawnCmd] type.
        ///
        /// Can be created using the [DespawnCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct DespawnCmdBuilder<State>(State);

        impl DespawnCmdBuilder<()> {
            /// Setter for the [`entity` field](DespawnCmd#structfield.entity).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity<T0>(self, value: T0) -> DespawnCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<i64, i64>,
            {
                DespawnCmdBuilder((value,))
            }

            /// Sets the [`entity` field](DespawnCmd#structfield.entity) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity_as_default(self) -> DespawnCmdBuilder<(::planus::DefaultValue,)> {
                self.entity(::planus::DefaultValue)
            }
        }

        impl<T0> DespawnCmdBuilder<(T0,)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [DespawnCmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<DespawnCmd>
            where
                Self: ::planus::WriteAsOffset<DespawnCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<T0: ::planus::WriteAsDefault<i64, i64>> ::planus::WriteAs<::planus::Offset<DespawnCmd>>
            for DespawnCmdBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<DespawnCmd>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DespawnCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<T0: ::planus::WriteAsDefault<i64, i64>>
            ::planus::WriteAsOptional<::planus::Offset<DespawnCmd>> for DespawnCmdBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<DespawnCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<DespawnCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<T0: ::planus::WriteAsDefault<i64, i64>> ::planus::WriteAsOffset<DespawnCmd>
            for DespawnCmdBuilder<(T0,)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<DespawnCmd> {
                let (v0,) = &self.0;
                DespawnCmd::create(builder, v0)
            }
        }

        /// Reference to a deserialized [DespawnCmd].
        #[derive(Copy, Clone)]
        pub struct DespawnCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> DespawnCmdRef<'a> {
            /// Getter for the [`entity` field](DespawnCmd#structfield.entity).
            #[inline]
            pub fn entity(&self) -> ::planus::Result<i64> {
                ::core::result::Result::Ok(self.0.access(0, "DespawnCmd", "entity")?.unwrap_or(0))
            }
        }

        impl<'a> ::core::fmt::Debug for DespawnCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("DespawnCmdRef");
                f.field("entity", &self.entity());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<DespawnCmdRef<'a>> for DespawnCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: DespawnCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    entity: ::core::convert::TryInto::try_into(value.entity()?)?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for DespawnCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for DespawnCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[DespawnCmdRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<DespawnCmd>> for DespawnCmd {
            type Value = ::planus::Offset<DespawnCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<DespawnCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for DespawnCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[DespawnCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `AddChildCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `AddChildCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:196`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct AddChildCmd {
            /// The field `parent` in the table `AddChildCmd`
            pub parent: i64,
            /// The field `child` in the table `AddChildCmd`
            pub child: i64,
            /// The field `index` in the table `AddChildCmd`
            pub index: u32,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for AddChildCmd {
            fn default() -> Self {
                Self {
                    parent: 0,
                    child: 0,
                    index: 0,
                }
            }
        }

        impl AddChildCmd {
            /// Creates a [AddChildCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> AddChildCmdBuilder<()> {
                AddChildCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_parent: impl ::planus::WriteAsDefault<i64, i64>,
                field_child: impl ::planus::WriteAsDefault<i64, i64>,
                field_index: impl ::planus::WriteAsDefault<u32, u32>,
            ) -> ::planus::Offset<Self> {
                let prepared_parent = field_parent.prepare(builder, &0);
                let prepared_child = field_child.prepare(builder, &0);
                let prepared_index = field_index.prepare(builder, &0);

                let mut table_writer: ::planus::table_writer::TableWriter<10> =
                    ::core::default::Default::default();
                if prepared_parent.is_some() {
                    table_writer.write_entry::<i64>(0);
                }
                if prepared_child.is_some() {
                    table_writer.write_entry::<i64>(1);
                }
                if prepared_index.is_some() {
                    table_writer.write_entry::<u32>(2);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_parent) = prepared_parent {
                            object_writer.write::<_, _, 8>(&prepared_parent);
                        }
                        if let ::core::option::Option::Some(prepared_child) = prepared_child {
                            object_writer.write::<_, _, 8>(&prepared_child);
                        }
                        if let ::core::option::Option::Some(prepared_index) = prepared_index {
                            object_writer.write::<_, _, 4>(&prepared_index);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<AddChildCmd>> for AddChildCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<AddChildCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<AddChildCmd>> for AddChildCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<AddChildCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<AddChildCmd> for AddChildCmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<AddChildCmd> {
                AddChildCmd::create(builder, self.parent, self.child, self.index)
            }
        }

        /// Builder for serializing an instance of the [AddChildCmd] type.
        ///
        /// Can be created using the [AddChildCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct AddChildCmdBuilder<State>(State);

        impl AddChildCmdBuilder<()> {
            /// Setter for the [`parent` field](AddChildCmd#structfield.parent).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn parent<T0>(self, value: T0) -> AddChildCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<i64, i64>,
            {
                AddChildCmdBuilder((value,))
            }

            /// Sets the [`parent` field](AddChildCmd#structfield.parent) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn parent_as_default(self) -> AddChildCmdBuilder<(::planus::DefaultValue,)> {
                self.parent(::planus::DefaultValue)
            }
        }

        impl<T0> AddChildCmdBuilder<(T0,)> {
            /// Setter for the [`child` field](AddChildCmd#structfield.child).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn child<T1>(self, value: T1) -> AddChildCmdBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsDefault<i64, i64>,
            {
                let (v0,) = self.0;
                AddChildCmdBuilder((v0, value))
            }

            /// Sets the [`child` field](AddChildCmd#structfield.child) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn child_as_default(self) -> AddChildCmdBuilder<(T0, ::planus::DefaultValue)> {
                self.child(::planus::DefaultValue)
            }
        }

        impl<T0, T1> AddChildCmdBuilder<(T0, T1)> {
            /// Setter for the [`index` field](AddChildCmd#structfield.index).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn index<T2>(self, value: T2) -> AddChildCmdBuilder<(T0, T1, T2)>
            where
                T2: ::planus::WriteAsDefault<u32, u32>,
            {
                let (v0, v1) = self.0;
                AddChildCmdBuilder((v0, v1, value))
            }

            /// Sets the [`index` field](AddChildCmd#structfield.index) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn index_as_default(self) -> AddChildCmdBuilder<(T0, T1, ::planus::DefaultValue)> {
                self.index(::planus::DefaultValue)
            }
        }

        impl<T0, T1, T2> AddChildCmdBuilder<(T0, T1, T2)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [AddChildCmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<AddChildCmd>
            where
                Self: ::planus::WriteAsOffset<AddChildCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsDefault<i64, i64>,
                T2: ::planus::WriteAsDefault<u32, u32>,
            > ::planus::WriteAs<::planus::Offset<AddChildCmd>>
            for AddChildCmdBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<AddChildCmd>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<AddChildCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsDefault<i64, i64>,
                T2: ::planus::WriteAsDefault<u32, u32>,
            > ::planus::WriteAsOptional<::planus::Offset<AddChildCmd>>
            for AddChildCmdBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<AddChildCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<AddChildCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<i64, i64>,
                T1: ::planus::WriteAsDefault<i64, i64>,
                T2: ::planus::WriteAsDefault<u32, u32>,
            > ::planus::WriteAsOffset<AddChildCmd> for AddChildCmdBuilder<(T0, T1, T2)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<AddChildCmd> {
                let (v0, v1, v2) = &self.0;
                AddChildCmd::create(builder, v0, v1, v2)
            }
        }

        /// Reference to a deserialized [AddChildCmd].
        #[derive(Copy, Clone)]
        pub struct AddChildCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> AddChildCmdRef<'a> {
            /// Getter for the [`parent` field](AddChildCmd#structfield.parent).
            #[inline]
            pub fn parent(&self) -> ::planus::Result<i64> {
                ::core::result::Result::Ok(self.0.access(0, "AddChildCmd", "parent")?.unwrap_or(0))
            }

            /// Getter for the [`child` field](AddChildCmd#structfield.child).
            #[inline]
            pub fn child(&self) -> ::planus::Result<i64> {
                ::core::result::Result::Ok(self.0.access(1, "AddChildCmd", "child")?.unwrap_or(0))
            }

            /// Getter for the [`index` field](AddChildCmd#structfield.index).
            #[inline]
            pub fn index(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(self.0.access(2, "AddChildCmd", "index")?.unwrap_or(0))
            }
        }

        impl<'a> ::core::fmt::Debug for AddChildCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("AddChildCmdRef");
                f.field("parent", &self.parent());
                f.field("child", &self.child());
                f.field("index", &self.index());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<AddChildCmdRef<'a>> for AddChildCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: AddChildCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    parent: ::core::convert::TryInto::try_into(value.parent()?)?,
                    child: ::core::convert::TryInto::try_into(value.child()?)?,
                    index: ::core::convert::TryInto::try_into(value.index()?)?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for AddChildCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for AddChildCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[AddChildCmdRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<AddChildCmd>> for AddChildCmd {
            type Value = ::planus::Offset<AddChildCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<AddChildCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for AddChildCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[AddChildCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `ResourceSetCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `ResourceSetCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:202`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct ResourceSetCmd {
            /// The field `value` in the table `ResourceSetCmd`
            pub value: ::core::option::Option<::planus::alloc::boxed::Box<self::CompValue>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for ResourceSetCmd {
            fn default() -> Self {
                Self {
                    value: ::core::default::Default::default(),
                }
            }
        }

        impl ResourceSetCmd {
            /// Creates a [ResourceSetCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> ResourceSetCmdBuilder<()> {
                ResourceSetCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_value: impl ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>,
            ) -> ::planus::Offset<Self> {
                let prepared_value = field_value.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<6> =
                    ::core::default::Default::default();
                if prepared_value.is_some() {
                    table_writer.write_entry::<::planus::Offset<self::CompValue>>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_value) = prepared_value {
                            object_writer.write::<_, _, 4>(&prepared_value);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<ResourceSetCmd>> for ResourceSetCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ResourceSetCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<ResourceSetCmd>> for ResourceSetCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ResourceSetCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<ResourceSetCmd> for ResourceSetCmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ResourceSetCmd> {
                ResourceSetCmd::create(builder, &self.value)
            }
        }

        /// Builder for serializing an instance of the [ResourceSetCmd] type.
        ///
        /// Can be created using the [ResourceSetCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct ResourceSetCmdBuilder<State>(State);

        impl ResourceSetCmdBuilder<()> {
            /// Setter for the [`value` field](ResourceSetCmd#structfield.value).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn value<T0>(self, value: T0) -> ResourceSetCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>,
            {
                ResourceSetCmdBuilder((value,))
            }

            /// Sets the [`value` field](ResourceSetCmd#structfield.value) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn value_as_null(self) -> ResourceSetCmdBuilder<((),)> {
                self.value(())
            }
        }

        impl<T0> ResourceSetCmdBuilder<(T0,)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [ResourceSetCmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<ResourceSetCmd>
            where
                Self: ::planus::WriteAsOffset<ResourceSetCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<T0: ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>>
            ::planus::WriteAs<::planus::Offset<ResourceSetCmd>> for ResourceSetCmdBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<ResourceSetCmd>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ResourceSetCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<T0: ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>>
            ::planus::WriteAsOptional<::planus::Offset<ResourceSetCmd>>
            for ResourceSetCmdBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<ResourceSetCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ResourceSetCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<T0: ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>>
            ::planus::WriteAsOffset<ResourceSetCmd> for ResourceSetCmdBuilder<(T0,)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ResourceSetCmd> {
                let (v0,) = &self.0;
                ResourceSetCmd::create(builder, v0)
            }
        }

        /// Reference to a deserialized [ResourceSetCmd].
        #[derive(Copy, Clone)]
        pub struct ResourceSetCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> ResourceSetCmdRef<'a> {
            /// Getter for the [`value` field](ResourceSetCmd#structfield.value).
            #[inline]
            pub fn value(
                &self,
            ) -> ::planus::Result<::core::option::Option<self::CompValueRef<'a>>> {
                self.0.access(0, "ResourceSetCmd", "value")
            }
        }

        impl<'a> ::core::fmt::Debug for ResourceSetCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("ResourceSetCmdRef");
                if let ::core::option::Option::Some(field_value) = self.value().transpose() {
                    f.field("value", &field_value);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<ResourceSetCmdRef<'a>> for ResourceSetCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: ResourceSetCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    value: if let ::core::option::Option::Some(value) = value.value()? {
                        ::core::option::Option::Some(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryInto::try_into(value)?,
                        ))
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for ResourceSetCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for ResourceSetCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[ResourceSetCmdRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<ResourceSetCmd>> for ResourceSetCmd {
            type Value = ::planus::Offset<ResourceSetCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<ResourceSetCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for ResourceSetCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[ResourceSetCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `EmitEventCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `EmitEventCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:206`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct EmitEventCmd {
            /// The field `event_name` in the table `EmitEventCmd`
            pub event_name: ::core::option::Option<::planus::alloc::string::String>,
            /// The field `entity` in the table `EmitEventCmd`
            pub entity: u64,
            /// The field `data` in the table `EmitEventCmd`
            pub data: ::core::option::Option<::planus::alloc::vec::Vec<u8>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for EmitEventCmd {
            fn default() -> Self {
                Self {
                    event_name: ::core::default::Default::default(),
                    entity: 0,
                    data: ::core::default::Default::default(),
                }
            }
        }

        impl EmitEventCmd {
            /// Creates a [EmitEventCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> EmitEventCmdBuilder<()> {
                EmitEventCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_event_name: impl ::planus::WriteAsOptional<
                    ::planus::Offset<::core::primitive::str>,
                >,
                field_entity: impl ::planus::WriteAsDefault<u64, u64>,
                field_data: impl ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            ) -> ::planus::Offset<Self> {
                let prepared_event_name = field_event_name.prepare(builder);
                let prepared_entity = field_entity.prepare(builder, &0);
                let prepared_data = field_data.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<10> =
                    ::core::default::Default::default();
                if prepared_entity.is_some() {
                    table_writer.write_entry::<u64>(1);
                }
                if prepared_event_name.is_some() {
                    table_writer.write_entry::<::planus::Offset<str>>(0);
                }
                if prepared_data.is_some() {
                    table_writer.write_entry::<::planus::Offset<[u8]>>(2);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_entity) = prepared_entity {
                            object_writer.write::<_, _, 8>(&prepared_entity);
                        }
                        if let ::core::option::Option::Some(prepared_event_name) =
                            prepared_event_name
                        {
                            object_writer.write::<_, _, 4>(&prepared_event_name);
                        }
                        if let ::core::option::Option::Some(prepared_data) = prepared_data {
                            object_writer.write::<_, _, 4>(&prepared_data);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<EmitEventCmd>> for EmitEventCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<EmitEventCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<EmitEventCmd>> for EmitEventCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<EmitEventCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<EmitEventCmd> for EmitEventCmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<EmitEventCmd> {
                EmitEventCmd::create(builder, &self.event_name, self.entity, &self.data)
            }
        }

        /// Builder for serializing an instance of the [EmitEventCmd] type.
        ///
        /// Can be created using the [EmitEventCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct EmitEventCmdBuilder<State>(State);

        impl EmitEventCmdBuilder<()> {
            /// Setter for the [`event_name` field](EmitEventCmd#structfield.event_name).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn event_name<T0>(self, value: T0) -> EmitEventCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
            {
                EmitEventCmdBuilder((value,))
            }

            /// Sets the [`event_name` field](EmitEventCmd#structfield.event_name) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn event_name_as_null(self) -> EmitEventCmdBuilder<((),)> {
                self.event_name(())
            }
        }

        impl<T0> EmitEventCmdBuilder<(T0,)> {
            /// Setter for the [`entity` field](EmitEventCmd#structfield.entity).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity<T1>(self, value: T1) -> EmitEventCmdBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsDefault<u64, u64>,
            {
                let (v0,) = self.0;
                EmitEventCmdBuilder((v0, value))
            }

            /// Sets the [`entity` field](EmitEventCmd#structfield.entity) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity_as_default(self) -> EmitEventCmdBuilder<(T0, ::planus::DefaultValue)> {
                self.entity(::planus::DefaultValue)
            }
        }

        impl<T0, T1> EmitEventCmdBuilder<(T0, T1)> {
            /// Setter for the [`data` field](EmitEventCmd#structfield.data).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn data<T2>(self, value: T2) -> EmitEventCmdBuilder<(T0, T1, T2)>
            where
                T2: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            {
                let (v0, v1) = self.0;
                EmitEventCmdBuilder((v0, v1, value))
            }

            /// Sets the [`data` field](EmitEventCmd#structfield.data) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn data_as_null(self) -> EmitEventCmdBuilder<(T0, T1, ())> {
                self.data(())
            }
        }

        impl<T0, T1, T2> EmitEventCmdBuilder<(T0, T1, T2)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [EmitEventCmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<EmitEventCmd>
            where
                Self: ::planus::WriteAsOffset<EmitEventCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T1: ::planus::WriteAsDefault<u64, u64>,
                T2: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            > ::planus::WriteAs<::planus::Offset<EmitEventCmd>>
            for EmitEventCmdBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<EmitEventCmd>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<EmitEventCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T1: ::planus::WriteAsDefault<u64, u64>,
                T2: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            > ::planus::WriteAsOptional<::planus::Offset<EmitEventCmd>>
            for EmitEventCmdBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<EmitEventCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<EmitEventCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<::planus::Offset<::core::primitive::str>>,
                T1: ::planus::WriteAsDefault<u64, u64>,
                T2: ::planus::WriteAsOptional<::planus::Offset<[u8]>>,
            > ::planus::WriteAsOffset<EmitEventCmd> for EmitEventCmdBuilder<(T0, T1, T2)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<EmitEventCmd> {
                let (v0, v1, v2) = &self.0;
                EmitEventCmd::create(builder, v0, v1, v2)
            }
        }

        /// Reference to a deserialized [EmitEventCmd].
        #[derive(Copy, Clone)]
        pub struct EmitEventCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> EmitEventCmdRef<'a> {
            /// Getter for the [`event_name` field](EmitEventCmd#structfield.event_name).
            #[inline]
            pub fn event_name(
                &self,
            ) -> ::planus::Result<::core::option::Option<&'a ::core::primitive::str>> {
                self.0.access(0, "EmitEventCmd", "event_name")
            }

            /// Getter for the [`entity` field](EmitEventCmd#structfield.entity).
            #[inline]
            pub fn entity(&self) -> ::planus::Result<u64> {
                ::core::result::Result::Ok(self.0.access(1, "EmitEventCmd", "entity")?.unwrap_or(0))
            }

            /// Getter for the [`data` field](EmitEventCmd#structfield.data).
            #[inline]
            pub fn data(&self) -> ::planus::Result<::core::option::Option<&'a [u8]>> {
                self.0.access(2, "EmitEventCmd", "data")
            }
        }

        impl<'a> ::core::fmt::Debug for EmitEventCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("EmitEventCmdRef");
                if let ::core::option::Option::Some(field_event_name) =
                    self.event_name().transpose()
                {
                    f.field("event_name", &field_event_name);
                }
                f.field("entity", &self.entity());
                if let ::core::option::Option::Some(field_data) = self.data().transpose() {
                    f.field("data", &field_data);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<EmitEventCmdRef<'a>> for EmitEventCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: EmitEventCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    event_name: value.event_name()?.map(::core::convert::Into::into),
                    entity: ::core::convert::TryInto::try_into(value.entity()?)?,
                    data: value.data()?.map(|v| v.to_vec()),
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for EmitEventCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for EmitEventCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[EmitEventCmdRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<EmitEventCmd>> for EmitEventCmd {
            type Value = ::planus::Offset<EmitEventCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<EmitEventCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for EmitEventCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[EmitEventCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `ConsumeMouseCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `ConsumeMouseCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:212`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct ConsumeMouseCmd {
            /// The field `button` in the table `ConsumeMouseCmd`
            pub button: u8,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for ConsumeMouseCmd {
            fn default() -> Self {
                Self { button: 0 }
            }
        }

        impl ConsumeMouseCmd {
            /// Creates a [ConsumeMouseCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> ConsumeMouseCmdBuilder<()> {
                ConsumeMouseCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_button: impl ::planus::WriteAsDefault<u8, u8>,
            ) -> ::planus::Offset<Self> {
                let prepared_button = field_button.prepare(builder, &0);

                let mut table_writer: ::planus::table_writer::TableWriter<6> =
                    ::core::default::Default::default();
                if prepared_button.is_some() {
                    table_writer.write_entry::<u8>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_button) = prepared_button {
                            object_writer.write::<_, _, 1>(&prepared_button);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<ConsumeMouseCmd>> for ConsumeMouseCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::planus::Offset<ConsumeMouseCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<ConsumeMouseCmd>> for ConsumeMouseCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ConsumeMouseCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<ConsumeMouseCmd> for ConsumeMouseCmd {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::planus::Offset<ConsumeMouseCmd> {
                ConsumeMouseCmd::create(builder, self.button)
            }
        }

        /// Builder for serializing an instance of the [ConsumeMouseCmd] type.
        ///
        /// Can be created using the [ConsumeMouseCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct ConsumeMouseCmdBuilder<State>(State);

        impl ConsumeMouseCmdBuilder<()> {
            /// Setter for the [`button` field](ConsumeMouseCmd#structfield.button).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn button<T0>(self, value: T0) -> ConsumeMouseCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u8, u8>,
            {
                ConsumeMouseCmdBuilder((value,))
            }

            /// Sets the [`button` field](ConsumeMouseCmd#structfield.button) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn button_as_default(self) -> ConsumeMouseCmdBuilder<(::planus::DefaultValue,)> {
                self.button(::planus::DefaultValue)
            }
        }

        impl<T0> ConsumeMouseCmdBuilder<(T0,)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [ConsumeMouseCmd].
            #[inline]
            pub fn finish(
                self,
                builder: &mut ::planus::Builder,
            ) -> ::planus::Offset<ConsumeMouseCmd>
            where
                Self: ::planus::WriteAsOffset<ConsumeMouseCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<T0: ::planus::WriteAsDefault<u8, u8>>
            ::planus::WriteAs<::planus::Offset<ConsumeMouseCmd>> for ConsumeMouseCmdBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<ConsumeMouseCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::planus::Offset<ConsumeMouseCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<T0: ::planus::WriteAsDefault<u8, u8>>
            ::planus::WriteAsOptional<::planus::Offset<ConsumeMouseCmd>>
            for ConsumeMouseCmdBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<ConsumeMouseCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ConsumeMouseCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<T0: ::planus::WriteAsDefault<u8, u8>> ::planus::WriteAsOffset<ConsumeMouseCmd>
            for ConsumeMouseCmdBuilder<(T0,)>
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::planus::Offset<ConsumeMouseCmd> {
                let (v0,) = &self.0;
                ConsumeMouseCmd::create(builder, v0)
            }
        }

        /// Reference to a deserialized [ConsumeMouseCmd].
        #[derive(Copy, Clone)]
        pub struct ConsumeMouseCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> ConsumeMouseCmdRef<'a> {
            /// Getter for the [`button` field](ConsumeMouseCmd#structfield.button).
            #[inline]
            pub fn button(&self) -> ::planus::Result<u8> {
                ::core::result::Result::Ok(
                    self.0.access(0, "ConsumeMouseCmd", "button")?.unwrap_or(0),
                )
            }
        }

        impl<'a> ::core::fmt::Debug for ConsumeMouseCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("ConsumeMouseCmdRef");
                f.field("button", &self.button());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<ConsumeMouseCmdRef<'a>> for ConsumeMouseCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: ConsumeMouseCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    button: ::core::convert::TryInto::try_into(value.button()?)?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for ConsumeMouseCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for ConsumeMouseCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[ConsumeMouseCmdRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<ConsumeMouseCmd>> for ConsumeMouseCmd {
            type Value = ::planus::Offset<ConsumeMouseCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<ConsumeMouseCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for ConsumeMouseCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[ConsumeMouseCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `ConsumeKeyCmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `ConsumeKeyCmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:216`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct ConsumeKeyCmd {
            /// The field `key` in the table `ConsumeKeyCmd`
            pub key: u32,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for ConsumeKeyCmd {
            fn default() -> Self {
                Self { key: 0 }
            }
        }

        impl ConsumeKeyCmd {
            /// Creates a [ConsumeKeyCmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> ConsumeKeyCmdBuilder<()> {
                ConsumeKeyCmdBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_key: impl ::planus::WriteAsDefault<u32, u32>,
            ) -> ::planus::Offset<Self> {
                let prepared_key = field_key.prepare(builder, &0);

                let mut table_writer: ::planus::table_writer::TableWriter<6> =
                    ::core::default::Default::default();
                if prepared_key.is_some() {
                    table_writer.write_entry::<u32>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_key) = prepared_key {
                            object_writer.write::<_, _, 4>(&prepared_key);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<ConsumeKeyCmd>> for ConsumeKeyCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ConsumeKeyCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<ConsumeKeyCmd>> for ConsumeKeyCmd {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ConsumeKeyCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<ConsumeKeyCmd> for ConsumeKeyCmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ConsumeKeyCmd> {
                ConsumeKeyCmd::create(builder, self.key)
            }
        }

        /// Builder for serializing an instance of the [ConsumeKeyCmd] type.
        ///
        /// Can be created using the [ConsumeKeyCmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct ConsumeKeyCmdBuilder<State>(State);

        impl ConsumeKeyCmdBuilder<()> {
            /// Setter for the [`key` field](ConsumeKeyCmd#structfield.key).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn key<T0>(self, value: T0) -> ConsumeKeyCmdBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                ConsumeKeyCmdBuilder((value,))
            }

            /// Sets the [`key` field](ConsumeKeyCmd#structfield.key) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn key_as_default(self) -> ConsumeKeyCmdBuilder<(::planus::DefaultValue,)> {
                self.key(::planus::DefaultValue)
            }
        }

        impl<T0> ConsumeKeyCmdBuilder<(T0,)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [ConsumeKeyCmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<ConsumeKeyCmd>
            where
                Self: ::planus::WriteAsOffset<ConsumeKeyCmd>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<T0: ::planus::WriteAsDefault<u32, u32>>
            ::planus::WriteAs<::planus::Offset<ConsumeKeyCmd>> for ConsumeKeyCmdBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<ConsumeKeyCmd>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ConsumeKeyCmd> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<T0: ::planus::WriteAsDefault<u32, u32>>
            ::planus::WriteAsOptional<::planus::Offset<ConsumeKeyCmd>>
            for ConsumeKeyCmdBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<ConsumeKeyCmd>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ConsumeKeyCmd>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<T0: ::planus::WriteAsDefault<u32, u32>> ::planus::WriteAsOffset<ConsumeKeyCmd>
            for ConsumeKeyCmdBuilder<(T0,)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ConsumeKeyCmd> {
                let (v0,) = &self.0;
                ConsumeKeyCmd::create(builder, v0)
            }
        }

        /// Reference to a deserialized [ConsumeKeyCmd].
        #[derive(Copy, Clone)]
        pub struct ConsumeKeyCmdRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> ConsumeKeyCmdRef<'a> {
            /// Getter for the [`key` field](ConsumeKeyCmd#structfield.key).
            #[inline]
            pub fn key(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(self.0.access(0, "ConsumeKeyCmd", "key")?.unwrap_or(0))
            }
        }

        impl<'a> ::core::fmt::Debug for ConsumeKeyCmdRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("ConsumeKeyCmdRef");
                f.field("key", &self.key());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<ConsumeKeyCmdRef<'a>> for ConsumeKeyCmd {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: ConsumeKeyCmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    key: ::core::convert::TryInto::try_into(value.key()?)?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for ConsumeKeyCmdRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for ConsumeKeyCmdRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[ConsumeKeyCmdRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<ConsumeKeyCmd>> for ConsumeKeyCmd {
            type Value = ::planus::Offset<ConsumeKeyCmd>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<ConsumeKeyCmd>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for ConsumeKeyCmdRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[ConsumeKeyCmdRef]", "read_as_root", 0)
                })
            }
        }

        /// The union `Cmd` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Union `Cmd` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:222`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub enum Cmd {
            /// The variant of type `SpawnCmd` in the union `Cmd`
            SpawnCmd(::planus::alloc::boxed::Box<self::SpawnCmd>),

            /// The variant of type `InsertCmd` in the union `Cmd`
            InsertCmd(::planus::alloc::boxed::Box<self::InsertCmd>),

            /// The variant of type `RemoveCmd` in the union `Cmd`
            RemoveCmd(::planus::alloc::boxed::Box<self::RemoveCmd>),

            /// The variant of type `DespawnCmd` in the union `Cmd`
            DespawnCmd(::planus::alloc::boxed::Box<self::DespawnCmd>),

            /// The variant of type `AddChildCmd` in the union `Cmd`
            AddChildCmd(::planus::alloc::boxed::Box<self::AddChildCmd>),

            /// The variant of type `ResourceSetCmd` in the union `Cmd`
            ResourceSetCmd(::planus::alloc::boxed::Box<self::ResourceSetCmd>),

            /// The variant of type `EmitEventCmd` in the union `Cmd`
            EmitEventCmd(::planus::alloc::boxed::Box<self::EmitEventCmd>),

            /// The variant of type `ConsumeMouseCmd` in the union `Cmd`
            ConsumeMouseCmd(::planus::alloc::boxed::Box<self::ConsumeMouseCmd>),

            /// The variant of type `ConsumeKeyCmd` in the union `Cmd`
            ConsumeKeyCmd(::planus::alloc::boxed::Box<self::ConsumeKeyCmd>),
        }

        impl Cmd {
            /// Creates a [CmdBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> CmdBuilder<::planus::Uninitialized> {
                CmdBuilder(::planus::Uninitialized)
            }

            #[inline]
            pub fn create_spawn_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::SpawnCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(1, value.prepare(builder).downcast())
            }

            #[inline]
            pub fn create_insert_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::InsertCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(2, value.prepare(builder).downcast())
            }

            #[inline]
            pub fn create_remove_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::RemoveCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(3, value.prepare(builder).downcast())
            }

            #[inline]
            pub fn create_despawn_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::DespawnCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(4, value.prepare(builder).downcast())
            }

            #[inline]
            pub fn create_add_child_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::AddChildCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(5, value.prepare(builder).downcast())
            }

            #[inline]
            pub fn create_resource_set_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::ResourceSetCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(6, value.prepare(builder).downcast())
            }

            #[inline]
            pub fn create_emit_event_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::EmitEventCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(7, value.prepare(builder).downcast())
            }

            #[inline]
            pub fn create_consume_mouse_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::ConsumeMouseCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(8, value.prepare(builder).downcast())
            }

            #[inline]
            pub fn create_consume_key_cmd(
                builder: &mut ::planus::Builder,
                value: impl ::planus::WriteAsOffset<self::ConsumeKeyCmd>,
            ) -> ::planus::UnionOffset<Self> {
                ::planus::UnionOffset::new(9, value.prepare(builder).downcast())
            }
        }

        impl ::planus::WriteAsUnion<Cmd> for Cmd {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Self> {
                match self {
                    Self::SpawnCmd(value) => Self::create_spawn_cmd(builder, value),
                    Self::InsertCmd(value) => Self::create_insert_cmd(builder, value),
                    Self::RemoveCmd(value) => Self::create_remove_cmd(builder, value),
                    Self::DespawnCmd(value) => Self::create_despawn_cmd(builder, value),
                    Self::AddChildCmd(value) => Self::create_add_child_cmd(builder, value),
                    Self::ResourceSetCmd(value) => Self::create_resource_set_cmd(builder, value),
                    Self::EmitEventCmd(value) => Self::create_emit_event_cmd(builder, value),
                    Self::ConsumeMouseCmd(value) => Self::create_consume_mouse_cmd(builder, value),
                    Self::ConsumeKeyCmd(value) => Self::create_consume_key_cmd(builder, value),
                }
            }
        }

        impl ::planus::WriteAsOptionalUnion<Cmd> for Cmd {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Self>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }

        /// Builder for serializing an instance of the [Cmd] type.
        ///
        /// Can be created using the [Cmd::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct CmdBuilder<T>(T);

        impl CmdBuilder<::planus::Uninitialized> {
            /// Creates an instance of the [`SpawnCmd` variant](Cmd#variant.SpawnCmd).
            #[inline]
            pub fn spawn_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<1, T>>
            where
                T: ::planus::WriteAsOffset<self::SpawnCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }

            /// Creates an instance of the [`InsertCmd` variant](Cmd#variant.InsertCmd).
            #[inline]
            pub fn insert_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<2, T>>
            where
                T: ::planus::WriteAsOffset<self::InsertCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }

            /// Creates an instance of the [`RemoveCmd` variant](Cmd#variant.RemoveCmd).
            #[inline]
            pub fn remove_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<3, T>>
            where
                T: ::planus::WriteAsOffset<self::RemoveCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }

            /// Creates an instance of the [`DespawnCmd` variant](Cmd#variant.DespawnCmd).
            #[inline]
            pub fn despawn_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<4, T>>
            where
                T: ::planus::WriteAsOffset<self::DespawnCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }

            /// Creates an instance of the [`AddChildCmd` variant](Cmd#variant.AddChildCmd).
            #[inline]
            pub fn add_child_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<5, T>>
            where
                T: ::planus::WriteAsOffset<self::AddChildCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }

            /// Creates an instance of the [`ResourceSetCmd` variant](Cmd#variant.ResourceSetCmd).
            #[inline]
            pub fn resource_set_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<6, T>>
            where
                T: ::planus::WriteAsOffset<self::ResourceSetCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }

            /// Creates an instance of the [`EmitEventCmd` variant](Cmd#variant.EmitEventCmd).
            #[inline]
            pub fn emit_event_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<7, T>>
            where
                T: ::planus::WriteAsOffset<self::EmitEventCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }

            /// Creates an instance of the [`ConsumeMouseCmd` variant](Cmd#variant.ConsumeMouseCmd).
            #[inline]
            pub fn consume_mouse_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<8, T>>
            where
                T: ::planus::WriteAsOffset<self::ConsumeMouseCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }

            /// Creates an instance of the [`ConsumeKeyCmd` variant](Cmd#variant.ConsumeKeyCmd).
            #[inline]
            pub fn consume_key_cmd<T>(self, value: T) -> CmdBuilder<::planus::Initialized<9, T>>
            where
                T: ::planus::WriteAsOffset<self::ConsumeKeyCmd>,
            {
                CmdBuilder(::planus::Initialized(value))
            }
        }

        impl<const N: u8, T> CmdBuilder<::planus::Initialized<N, T>> {
            /// Finish writing the builder to get an [UnionOffset](::planus::UnionOffset) to a serialized [Cmd].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd>
            where
                Self: ::planus::WriteAsUnion<Cmd>,
            {
                ::planus::WriteAsUnion::prepare(&self, builder)
            }
        }

        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<1, T>>
        where
            T: ::planus::WriteAsOffset<self::SpawnCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(1, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<1, T>>
        where
            T: ::planus::WriteAsOffset<self::SpawnCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }
        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<2, T>>
        where
            T: ::planus::WriteAsOffset<self::InsertCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(2, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<2, T>>
        where
            T: ::planus::WriteAsOffset<self::InsertCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }
        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<3, T>>
        where
            T: ::planus::WriteAsOffset<self::RemoveCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(3, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<3, T>>
        where
            T: ::planus::WriteAsOffset<self::RemoveCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }
        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<4, T>>
        where
            T: ::planus::WriteAsOffset<self::DespawnCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(4, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<4, T>>
        where
            T: ::planus::WriteAsOffset<self::DespawnCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }
        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<5, T>>
        where
            T: ::planus::WriteAsOffset<self::AddChildCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(5, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<5, T>>
        where
            T: ::planus::WriteAsOffset<self::AddChildCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }
        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<6, T>>
        where
            T: ::planus::WriteAsOffset<self::ResourceSetCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(6, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<6, T>>
        where
            T: ::planus::WriteAsOffset<self::ResourceSetCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }
        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<7, T>>
        where
            T: ::planus::WriteAsOffset<self::EmitEventCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(7, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<7, T>>
        where
            T: ::planus::WriteAsOffset<self::EmitEventCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }
        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<8, T>>
        where
            T: ::planus::WriteAsOffset<self::ConsumeMouseCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(8, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<8, T>>
        where
            T: ::planus::WriteAsOffset<self::ConsumeMouseCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }
        impl<T> ::planus::WriteAsUnion<Cmd> for CmdBuilder<::planus::Initialized<9, T>>
        where
            T: ::planus::WriteAsOffset<self::ConsumeKeyCmd>,
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::UnionOffset<Cmd> {
                ::planus::UnionOffset::new(9, (self.0).0.prepare(builder).downcast())
            }
        }

        impl<T> ::planus::WriteAsOptionalUnion<Cmd> for CmdBuilder<::planus::Initialized<9, T>>
        where
            T: ::planus::WriteAsOffset<self::ConsumeKeyCmd>,
        {
            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::UnionOffset<Cmd>> {
                ::core::option::Option::Some(::planus::WriteAsUnion::prepare(self, builder))
            }
        }

        /// Reference to a deserialized [Cmd].
        #[derive(Copy, Clone, Debug)]
        pub enum CmdRef<'a> {
            SpawnCmd(self::SpawnCmdRef<'a>),
            InsertCmd(self::InsertCmdRef<'a>),
            RemoveCmd(self::RemoveCmdRef<'a>),
            DespawnCmd(self::DespawnCmdRef<'a>),
            AddChildCmd(self::AddChildCmdRef<'a>),
            ResourceSetCmd(self::ResourceSetCmdRef<'a>),
            EmitEventCmd(self::EmitEventCmdRef<'a>),
            ConsumeMouseCmd(self::ConsumeMouseCmdRef<'a>),
            ConsumeKeyCmd(self::ConsumeKeyCmdRef<'a>),
        }

        impl<'a> ::core::convert::TryFrom<CmdRef<'a>> for Cmd {
            type Error = ::planus::Error;

            fn try_from(value: CmdRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(match value {
                    CmdRef::SpawnCmd(value) => Self::SpawnCmd(::planus::alloc::boxed::Box::new(
                        ::core::convert::TryFrom::try_from(value)?,
                    )),

                    CmdRef::InsertCmd(value) => Self::InsertCmd(::planus::alloc::boxed::Box::new(
                        ::core::convert::TryFrom::try_from(value)?,
                    )),

                    CmdRef::RemoveCmd(value) => Self::RemoveCmd(::planus::alloc::boxed::Box::new(
                        ::core::convert::TryFrom::try_from(value)?,
                    )),

                    CmdRef::DespawnCmd(value) => {
                        Self::DespawnCmd(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryFrom::try_from(value)?,
                        ))
                    }

                    CmdRef::AddChildCmd(value) => {
                        Self::AddChildCmd(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryFrom::try_from(value)?,
                        ))
                    }

                    CmdRef::ResourceSetCmd(value) => {
                        Self::ResourceSetCmd(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryFrom::try_from(value)?,
                        ))
                    }

                    CmdRef::EmitEventCmd(value) => {
                        Self::EmitEventCmd(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryFrom::try_from(value)?,
                        ))
                    }

                    CmdRef::ConsumeMouseCmd(value) => {
                        Self::ConsumeMouseCmd(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryFrom::try_from(value)?,
                        ))
                    }

                    CmdRef::ConsumeKeyCmd(value) => {
                        Self::ConsumeKeyCmd(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryFrom::try_from(value)?,
                        ))
                    }
                })
            }
        }

        impl<'a> ::planus::TableReadUnion<'a> for CmdRef<'a> {
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                tag: u8,
                field_offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                match tag {
                    1 => ::core::result::Result::Ok(Self::SpawnCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    2 => ::core::result::Result::Ok(Self::InsertCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    3 => ::core::result::Result::Ok(Self::RemoveCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    4 => ::core::result::Result::Ok(Self::DespawnCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    5 => ::core::result::Result::Ok(Self::AddChildCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    6 => ::core::result::Result::Ok(Self::ResourceSetCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    7 => ::core::result::Result::Ok(Self::EmitEventCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    8 => ::core::result::Result::Ok(Self::ConsumeMouseCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    9 => ::core::result::Result::Ok(Self::ConsumeKeyCmd(
                        ::planus::TableRead::from_buffer(buffer, field_offset)?,
                    )),
                    _ => {
                        ::core::result::Result::Err(::planus::errors::ErrorKind::UnknownUnionTag {
                            tag,
                        })
                    }
                }
            }
        }

        impl<'a> ::planus::VectorReadUnion<'a> for CmdRef<'a> {
            const VECTOR_NAME: &'static str = "[CmdRef]";
        }

        /// The table `CommandBuffer` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `CommandBuffer` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:234`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct CommandBuffer {
            /// The field `cmds` in the table `CommandBuffer`
            pub cmds: ::core::option::Option<::planus::alloc::vec::Vec<self::Cmd>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for CommandBuffer {
            fn default() -> Self {
                Self {
                    cmds: ::core::default::Default::default(),
                }
            }
        }

        impl CommandBuffer {
            /// Creates a [CommandBufferBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> CommandBufferBuilder<()> {
                CommandBufferBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_cmds: impl ::planus::WriteAsOptionalUnionVector<self::Cmd>,
            ) -> ::planus::Offset<Self> {
                let prepared_cmds = field_cmds.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_cmds.is_some() {
                    table_writer.write_entry::<::planus::Offset<[u8]>>(0);
                }
                if prepared_cmds.is_some() {
                    table_writer.write_entry::<::planus::Offset<[::planus::Offset<self::Cmd>]>>(1);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_cmds) = prepared_cmds {
                            object_writer.write::<_, _, 4>(&prepared_cmds.tags_offset());
                        }
                        if let ::core::option::Option::Some(prepared_cmds) = prepared_cmds {
                            object_writer.write::<_, _, 4>(&prepared_cmds.values_offset());
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<CommandBuffer>> for CommandBuffer {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<CommandBuffer> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<CommandBuffer>> for CommandBuffer {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<CommandBuffer>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<CommandBuffer> for CommandBuffer {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<CommandBuffer> {
                CommandBuffer::create(builder, &self.cmds)
            }
        }

        /// Builder for serializing an instance of the [CommandBuffer] type.
        ///
        /// Can be created using the [CommandBuffer::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct CommandBufferBuilder<State>(State);

        impl CommandBufferBuilder<()> {
            /// Setter for the [`cmds` field](CommandBuffer#structfield.cmds).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn cmds<T0>(self, value: T0) -> CommandBufferBuilder<(T0,)>
            where
                T0: ::planus::WriteAsOptionalUnionVector<self::Cmd>,
            {
                CommandBufferBuilder((value,))
            }

            /// Sets the [`cmds` field](CommandBuffer#structfield.cmds) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn cmds_as_null(self) -> CommandBufferBuilder<((),)> {
                self.cmds(())
            }
        }

        impl<T0> CommandBufferBuilder<(T0,)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [CommandBuffer].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<CommandBuffer>
            where
                Self: ::planus::WriteAsOffset<CommandBuffer>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<T0: ::planus::WriteAsOptionalUnionVector<self::Cmd>>
            ::planus::WriteAs<::planus::Offset<CommandBuffer>> for CommandBufferBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<CommandBuffer>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<CommandBuffer> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<T0: ::planus::WriteAsOptionalUnionVector<self::Cmd>>
            ::planus::WriteAsOptional<::planus::Offset<CommandBuffer>>
            for CommandBufferBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<CommandBuffer>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<CommandBuffer>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<T0: ::planus::WriteAsOptionalUnionVector<self::Cmd>>
            ::planus::WriteAsOffset<CommandBuffer> for CommandBufferBuilder<(T0,)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<CommandBuffer> {
                let (v0,) = &self.0;
                CommandBuffer::create(builder, v0)
            }
        }

        /// Reference to a deserialized [CommandBuffer].
        #[derive(Copy, Clone)]
        pub struct CommandBufferRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> CommandBufferRef<'a> {
            /// Getter for the [`cmds` field](CommandBuffer#structfield.cmds).
            #[inline]
            pub fn cmds(
                &self,
            ) -> ::planus::Result<::core::option::Option<::planus::UnionVector<'a, self::CmdRef<'a>>>>
            {
                self.0.access_union_vector(0, "CommandBuffer", "cmds")
            }
        }

        impl<'a> ::core::fmt::Debug for CommandBufferRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("CommandBufferRef");
                if let ::core::option::Option::Some(field_cmds) = self.cmds().transpose() {
                    f.field("cmds", &field_cmds);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<CommandBufferRef<'a>> for CommandBuffer {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: CommandBufferRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    cmds: if let ::core::option::Option::Some(cmds) = value.cmds()? {
                        ::core::option::Option::Some(cmds.to_vec()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for CommandBufferRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for CommandBufferRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[CommandBufferRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<CommandBuffer>> for CommandBuffer {
            type Value = ::planus::Offset<CommandBuffer>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<CommandBuffer>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for CommandBufferRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[CommandBufferRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `ObserverInput` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `ObserverInput` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:241`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct ObserverInput {
            /// The field `obs_id` in the table `ObserverInput`
            pub obs_id: u32,
            /// The field `entity` in the table `ObserverInput`
            pub entity: u64,
            /// The field `value` in the table `ObserverInput`
            pub value: ::core::option::Option<::planus::alloc::boxed::Box<self::CompValue>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for ObserverInput {
            fn default() -> Self {
                Self {
                    obs_id: 0,
                    entity: 0,
                    value: ::core::default::Default::default(),
                }
            }
        }

        impl ObserverInput {
            /// Creates a [ObserverInputBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> ObserverInputBuilder<()> {
                ObserverInputBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_obs_id: impl ::planus::WriteAsDefault<u32, u32>,
                field_entity: impl ::planus::WriteAsDefault<u64, u64>,
                field_value: impl ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>,
            ) -> ::planus::Offset<Self> {
                let prepared_obs_id = field_obs_id.prepare(builder, &0);
                let prepared_entity = field_entity.prepare(builder, &0);
                let prepared_value = field_value.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<10> =
                    ::core::default::Default::default();
                if prepared_entity.is_some() {
                    table_writer.write_entry::<u64>(1);
                }
                if prepared_obs_id.is_some() {
                    table_writer.write_entry::<u32>(0);
                }
                if prepared_value.is_some() {
                    table_writer.write_entry::<::planus::Offset<self::CompValue>>(2);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_entity) = prepared_entity {
                            object_writer.write::<_, _, 8>(&prepared_entity);
                        }
                        if let ::core::option::Option::Some(prepared_obs_id) = prepared_obs_id {
                            object_writer.write::<_, _, 4>(&prepared_obs_id);
                        }
                        if let ::core::option::Option::Some(prepared_value) = prepared_value {
                            object_writer.write::<_, _, 4>(&prepared_value);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<ObserverInput>> for ObserverInput {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverInput> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<ObserverInput>> for ObserverInput {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ObserverInput>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<ObserverInput> for ObserverInput {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverInput> {
                ObserverInput::create(builder, self.obs_id, self.entity, &self.value)
            }
        }

        /// Builder for serializing an instance of the [ObserverInput] type.
        ///
        /// Can be created using the [ObserverInput::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct ObserverInputBuilder<State>(State);

        impl ObserverInputBuilder<()> {
            /// Setter for the [`obs_id` field](ObserverInput#structfield.obs_id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn obs_id<T0>(self, value: T0) -> ObserverInputBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                ObserverInputBuilder((value,))
            }

            /// Sets the [`obs_id` field](ObserverInput#structfield.obs_id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn obs_id_as_default(self) -> ObserverInputBuilder<(::planus::DefaultValue,)> {
                self.obs_id(::planus::DefaultValue)
            }
        }

        impl<T0> ObserverInputBuilder<(T0,)> {
            /// Setter for the [`entity` field](ObserverInput#structfield.entity).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity<T1>(self, value: T1) -> ObserverInputBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsDefault<u64, u64>,
            {
                let (v0,) = self.0;
                ObserverInputBuilder((v0, value))
            }

            /// Sets the [`entity` field](ObserverInput#structfield.entity) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity_as_default(self) -> ObserverInputBuilder<(T0, ::planus::DefaultValue)> {
                self.entity(::planus::DefaultValue)
            }
        }

        impl<T0, T1> ObserverInputBuilder<(T0, T1)> {
            /// Setter for the [`value` field](ObserverInput#structfield.value).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn value<T2>(self, value: T2) -> ObserverInputBuilder<(T0, T1, T2)>
            where
                T2: ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>,
            {
                let (v0, v1) = self.0;
                ObserverInputBuilder((v0, v1, value))
            }

            /// Sets the [`value` field](ObserverInput#structfield.value) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn value_as_null(self) -> ObserverInputBuilder<(T0, T1, ())> {
                self.value(())
            }
        }

        impl<T0, T1, T2> ObserverInputBuilder<(T0, T1, T2)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [ObserverInput].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverInput>
            where
                Self: ::planus::WriteAsOffset<ObserverInput>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsDefault<u64, u64>,
                T2: ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>,
            > ::planus::WriteAs<::planus::Offset<ObserverInput>>
            for ObserverInputBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<ObserverInput>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverInput> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsDefault<u64, u64>,
                T2: ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>,
            > ::planus::WriteAsOptional<::planus::Offset<ObserverInput>>
            for ObserverInputBuilder<(T0, T1, T2)>
        {
            type Prepared = ::planus::Offset<ObserverInput>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<ObserverInput>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsDefault<u32, u32>,
                T1: ::planus::WriteAsDefault<u64, u64>,
                T2: ::planus::WriteAsOptional<::planus::Offset<self::CompValue>>,
            > ::planus::WriteAsOffset<ObserverInput> for ObserverInputBuilder<(T0, T1, T2)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<ObserverInput> {
                let (v0, v1, v2) = &self.0;
                ObserverInput::create(builder, v0, v1, v2)
            }
        }

        /// Reference to a deserialized [ObserverInput].
        #[derive(Copy, Clone)]
        pub struct ObserverInputRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> ObserverInputRef<'a> {
            /// Getter for the [`obs_id` field](ObserverInput#structfield.obs_id).
            #[inline]
            pub fn obs_id(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(
                    self.0.access(0, "ObserverInput", "obs_id")?.unwrap_or(0),
                )
            }

            /// Getter for the [`entity` field](ObserverInput#structfield.entity).
            #[inline]
            pub fn entity(&self) -> ::planus::Result<u64> {
                ::core::result::Result::Ok(
                    self.0.access(1, "ObserverInput", "entity")?.unwrap_or(0),
                )
            }

            /// Getter for the [`value` field](ObserverInput#structfield.value).
            #[inline]
            pub fn value(
                &self,
            ) -> ::planus::Result<::core::option::Option<self::CompValueRef<'a>>> {
                self.0.access(2, "ObserverInput", "value")
            }
        }

        impl<'a> ::core::fmt::Debug for ObserverInputRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("ObserverInputRef");
                f.field("obs_id", &self.obs_id());
                f.field("entity", &self.entity());
                if let ::core::option::Option::Some(field_value) = self.value().transpose() {
                    f.field("value", &field_value);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<ObserverInputRef<'a>> for ObserverInput {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: ObserverInputRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    obs_id: ::core::convert::TryInto::try_into(value.obs_id()?)?,
                    entity: ::core::convert::TryInto::try_into(value.entity()?)?,
                    value: if let ::core::option::Option::Some(value) = value.value()? {
                        ::core::option::Option::Some(::planus::alloc::boxed::Box::new(
                            ::core::convert::TryInto::try_into(value)?,
                        ))
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for ObserverInputRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for ObserverInputRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[ObserverInputRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<ObserverInput>> for ObserverInput {
            type Value = ::planus::Offset<ObserverInput>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<ObserverInput>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for ObserverInputRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[ObserverInputRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `SpawnResolved` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `SpawnResolved` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:253`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct SpawnResolved {
            /// The field `temp_id` in the table `SpawnResolved`
            pub temp_id: u32,
            /// The field `entity` in the table `SpawnResolved`
            pub entity: u64,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for SpawnResolved {
            fn default() -> Self {
                Self {
                    temp_id: 0,
                    entity: 0,
                }
            }
        }

        impl SpawnResolved {
            /// Creates a [SpawnResolvedBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> SpawnResolvedBuilder<()> {
                SpawnResolvedBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_temp_id: impl ::planus::WriteAsDefault<u32, u32>,
                field_entity: impl ::planus::WriteAsDefault<u64, u64>,
            ) -> ::planus::Offset<Self> {
                let prepared_temp_id = field_temp_id.prepare(builder, &0);
                let prepared_entity = field_entity.prepare(builder, &0);

                let mut table_writer: ::planus::table_writer::TableWriter<8> =
                    ::core::default::Default::default();
                if prepared_entity.is_some() {
                    table_writer.write_entry::<u64>(1);
                }
                if prepared_temp_id.is_some() {
                    table_writer.write_entry::<u32>(0);
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_entity) = prepared_entity {
                            object_writer.write::<_, _, 8>(&prepared_entity);
                        }
                        if let ::core::option::Option::Some(prepared_temp_id) = prepared_temp_id {
                            object_writer.write::<_, _, 4>(&prepared_temp_id);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<SpawnResolved>> for SpawnResolved {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnResolved> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<SpawnResolved>> for SpawnResolved {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SpawnResolved>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<SpawnResolved> for SpawnResolved {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnResolved> {
                SpawnResolved::create(builder, self.temp_id, self.entity)
            }
        }

        /// Builder for serializing an instance of the [SpawnResolved] type.
        ///
        /// Can be created using the [SpawnResolved::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct SpawnResolvedBuilder<State>(State);

        impl SpawnResolvedBuilder<()> {
            /// Setter for the [`temp_id` field](SpawnResolved#structfield.temp_id).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn temp_id<T0>(self, value: T0) -> SpawnResolvedBuilder<(T0,)>
            where
                T0: ::planus::WriteAsDefault<u32, u32>,
            {
                SpawnResolvedBuilder((value,))
            }

            /// Sets the [`temp_id` field](SpawnResolved#structfield.temp_id) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn temp_id_as_default(self) -> SpawnResolvedBuilder<(::planus::DefaultValue,)> {
                self.temp_id(::planus::DefaultValue)
            }
        }

        impl<T0> SpawnResolvedBuilder<(T0,)> {
            /// Setter for the [`entity` field](SpawnResolved#structfield.entity).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity<T1>(self, value: T1) -> SpawnResolvedBuilder<(T0, T1)>
            where
                T1: ::planus::WriteAsDefault<u64, u64>,
            {
                let (v0,) = self.0;
                SpawnResolvedBuilder((v0, value))
            }

            /// Sets the [`entity` field](SpawnResolved#structfield.entity) to the default value.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn entity_as_default(self) -> SpawnResolvedBuilder<(T0, ::planus::DefaultValue)> {
                self.entity(::planus::DefaultValue)
            }
        }

        impl<T0, T1> SpawnResolvedBuilder<(T0, T1)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [SpawnResolved].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnResolved>
            where
                Self: ::planus::WriteAsOffset<SpawnResolved>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<T0: ::planus::WriteAsDefault<u32, u32>, T1: ::planus::WriteAsDefault<u64, u64>>
            ::planus::WriteAs<::planus::Offset<SpawnResolved>> for SpawnResolvedBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<SpawnResolved>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnResolved> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<T0: ::planus::WriteAsDefault<u32, u32>, T1: ::planus::WriteAsDefault<u64, u64>>
            ::planus::WriteAsOptional<::planus::Offset<SpawnResolved>>
            for SpawnResolvedBuilder<(T0, T1)>
        {
            type Prepared = ::planus::Offset<SpawnResolved>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SpawnResolved>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<T0: ::planus::WriteAsDefault<u32, u32>, T1: ::planus::WriteAsDefault<u64, u64>>
            ::planus::WriteAsOffset<SpawnResolved> for SpawnResolvedBuilder<(T0, T1)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnResolved> {
                let (v0, v1) = &self.0;
                SpawnResolved::create(builder, v0, v1)
            }
        }

        /// Reference to a deserialized [SpawnResolved].
        #[derive(Copy, Clone)]
        pub struct SpawnResolvedRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> SpawnResolvedRef<'a> {
            /// Getter for the [`temp_id` field](SpawnResolved#structfield.temp_id).
            #[inline]
            pub fn temp_id(&self) -> ::planus::Result<u32> {
                ::core::result::Result::Ok(
                    self.0.access(0, "SpawnResolved", "temp_id")?.unwrap_or(0),
                )
            }

            /// Getter for the [`entity` field](SpawnResolved#structfield.entity).
            #[inline]
            pub fn entity(&self) -> ::planus::Result<u64> {
                ::core::result::Result::Ok(
                    self.0.access(1, "SpawnResolved", "entity")?.unwrap_or(0),
                )
            }
        }

        impl<'a> ::core::fmt::Debug for SpawnResolvedRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("SpawnResolvedRef");
                f.field("temp_id", &self.temp_id());
                f.field("entity", &self.entity());
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<SpawnResolvedRef<'a>> for SpawnResolved {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: SpawnResolvedRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    temp_id: ::core::convert::TryInto::try_into(value.temp_id()?)?,
                    entity: ::core::convert::TryInto::try_into(value.entity()?)?,
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for SpawnResolvedRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for SpawnResolvedRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[SpawnResolvedRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<SpawnResolved>> for SpawnResolved {
            type Value = ::planus::Offset<SpawnResolved>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<SpawnResolved>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for SpawnResolvedRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[SpawnResolvedRef]", "read_as_root", 0)
                })
            }
        }

        /// The table `SpawnedInput` in the namespace `ModAbi`
        ///
        /// Generated from these locations:
        /// * Table `SpawnedInput` in the file `C:\dev\cuo\cuo-agents\external\TinyEcs\src\TinyEcs.Bevy.Modding\abi\mod-abi.fbs:258`
        #[derive(
            Clone,
            Debug,
            PartialEq,
            PartialOrd,
            Eq,
            Ord,
            Hash,
            ::serde::Serialize,
            ::serde::Deserialize,
        )]
        pub struct SpawnedInput {
            /// The field `spawned` in the table `SpawnedInput`
            pub spawned: ::core::option::Option<::planus::alloc::vec::Vec<self::SpawnResolved>>,
        }

        #[allow(clippy::derivable_impls)]
        impl ::core::default::Default for SpawnedInput {
            fn default() -> Self {
                Self {
                    spawned: ::core::default::Default::default(),
                }
            }
        }

        impl SpawnedInput {
            /// Creates a [SpawnedInputBuilder] for serializing an instance of this table.
            #[inline]
            pub fn builder() -> SpawnedInputBuilder<()> {
                SpawnedInputBuilder(())
            }

            #[allow(clippy::too_many_arguments)]
            pub fn create(
                builder: &mut ::planus::Builder,
                field_spawned: impl ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::SpawnResolved>]>,
                >,
            ) -> ::planus::Offset<Self> {
                let prepared_spawned = field_spawned.prepare(builder);

                let mut table_writer: ::planus::table_writer::TableWriter<6> =
                    ::core::default::Default::default();
                if prepared_spawned.is_some() {
                    table_writer
                        .write_entry::<::planus::Offset<[::planus::Offset<self::SpawnResolved>]>>(
                            0,
                        );
                }

                unsafe {
                    table_writer.finish(builder, |object_writer| {
                        if let ::core::option::Option::Some(prepared_spawned) = prepared_spawned {
                            object_writer.write::<_, _, 4>(&prepared_spawned);
                        }
                    });
                }
                builder.current_offset()
            }
        }

        impl ::planus::WriteAs<::planus::Offset<SpawnedInput>> for SpawnedInput {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnedInput> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl ::planus::WriteAsOptional<::planus::Offset<SpawnedInput>> for SpawnedInput {
            type Prepared = ::planus::Offset<Self>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SpawnedInput>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl ::planus::WriteAsOffset<SpawnedInput> for SpawnedInput {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnedInput> {
                SpawnedInput::create(builder, &self.spawned)
            }
        }

        /// Builder for serializing an instance of the [SpawnedInput] type.
        ///
        /// Can be created using the [SpawnedInput::builder] method.
        #[derive(Debug)]
        #[must_use]
        pub struct SpawnedInputBuilder<State>(State);

        impl SpawnedInputBuilder<()> {
            /// Setter for the [`spawned` field](SpawnedInput#structfield.spawned).
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn spawned<T0>(self, value: T0) -> SpawnedInputBuilder<(T0,)>
            where
                T0: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::SpawnResolved>]>,
                >,
            {
                SpawnedInputBuilder((value,))
            }

            /// Sets the [`spawned` field](SpawnedInput#structfield.spawned) to null.
            #[inline]
            #[allow(clippy::type_complexity)]
            pub fn spawned_as_null(self) -> SpawnedInputBuilder<((),)> {
                self.spawned(())
            }
        }

        impl<T0> SpawnedInputBuilder<(T0,)> {
            /// Finish writing the builder to get an [Offset](::planus::Offset) to a serialized [SpawnedInput].
            #[inline]
            pub fn finish(self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnedInput>
            where
                Self: ::planus::WriteAsOffset<SpawnedInput>,
            {
                ::planus::WriteAsOffset::prepare(&self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::SpawnResolved>]>,
                >,
            > ::planus::WriteAs<::planus::Offset<SpawnedInput>> for SpawnedInputBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<SpawnedInput>;

            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnedInput> {
                ::planus::WriteAsOffset::prepare(self, builder)
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::SpawnResolved>]>,
                >,
            > ::planus::WriteAsOptional<::planus::Offset<SpawnedInput>>
            for SpawnedInputBuilder<(T0,)>
        {
            type Prepared = ::planus::Offset<SpawnedInput>;

            #[inline]
            fn prepare(
                &self,
                builder: &mut ::planus::Builder,
            ) -> ::core::option::Option<::planus::Offset<SpawnedInput>> {
                ::core::option::Option::Some(::planus::WriteAsOffset::prepare(self, builder))
            }
        }

        impl<
                T0: ::planus::WriteAsOptional<
                    ::planus::Offset<[::planus::Offset<self::SpawnResolved>]>,
                >,
            > ::planus::WriteAsOffset<SpawnedInput> for SpawnedInputBuilder<(T0,)>
        {
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> ::planus::Offset<SpawnedInput> {
                let (v0,) = &self.0;
                SpawnedInput::create(builder, v0)
            }
        }

        /// Reference to a deserialized [SpawnedInput].
        #[derive(Copy, Clone)]
        pub struct SpawnedInputRef<'a>(#[allow(dead_code)] ::planus::table_reader::Table<'a>);

        impl<'a> SpawnedInputRef<'a> {
            /// Getter for the [`spawned` field](SpawnedInput#structfield.spawned).
            #[inline]
            pub fn spawned(
                &self,
            ) -> ::planus::Result<
                ::core::option::Option<
                    ::planus::Vector<'a, ::planus::Result<self::SpawnResolvedRef<'a>>>,
                >,
            > {
                self.0.access(0, "SpawnedInput", "spawned")
            }
        }

        impl<'a> ::core::fmt::Debug for SpawnedInputRef<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                let mut f = f.debug_struct("SpawnedInputRef");
                if let ::core::option::Option::Some(field_spawned) = self.spawned().transpose() {
                    f.field("spawned", &field_spawned);
                }
                f.finish()
            }
        }

        impl<'a> ::core::convert::TryFrom<SpawnedInputRef<'a>> for SpawnedInput {
            type Error = ::planus::Error;

            #[allow(unreachable_code)]
            fn try_from(value: SpawnedInputRef<'a>) -> ::planus::Result<Self> {
                ::core::result::Result::Ok(Self {
                    spawned: if let ::core::option::Option::Some(spawned) = value.spawned()? {
                        ::core::option::Option::Some(spawned.to_vec_result()?)
                    } else {
                        ::core::option::Option::None
                    },
                })
            }
        }

        impl<'a> ::planus::TableRead<'a> for SpawnedInputRef<'a> {
            #[inline]
            fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::core::result::Result<Self, ::planus::errors::ErrorKind> {
                ::core::result::Result::Ok(Self(::planus::table_reader::Table::from_buffer(
                    buffer, offset,
                )?))
            }
        }

        impl<'a> ::planus::VectorReadInner<'a> for SpawnedInputRef<'a> {
            type Error = ::planus::Error;
            const STRIDE: usize = 4;

            unsafe fn from_buffer(
                buffer: ::planus::SliceWithStartOffset<'a>,
                offset: usize,
            ) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(buffer, offset).map_err(|error_kind| {
                    error_kind.with_error_location(
                        "[SpawnedInputRef]",
                        "get",
                        buffer.offset_from_start,
                    )
                })
            }
        }

        /// # Safety
        /// The planus compiler generates implementations that initialize
        /// the bytes in `write_values`.
        unsafe impl ::planus::VectorWrite<::planus::Offset<SpawnedInput>> for SpawnedInput {
            type Value = ::planus::Offset<SpawnedInput>;
            const STRIDE: usize = 4;
            #[inline]
            fn prepare(&self, builder: &mut ::planus::Builder) -> Self::Value {
                ::planus::WriteAs::prepare(self, builder)
            }

            #[inline]
            unsafe fn write_values(
                values: &[::planus::Offset<SpawnedInput>],
                bytes: *mut ::core::mem::MaybeUninit<u8>,
                buffer_position: u32,
            ) {
                let bytes = bytes as *mut [::core::mem::MaybeUninit<u8>; 4];
                for (i, v) in ::core::iter::Iterator::enumerate(values.iter()) {
                    ::planus::WriteAsPrimitive::write(
                        v,
                        ::planus::Cursor::new(unsafe { &mut *bytes.add(i) }),
                        buffer_position - (Self::STRIDE * i) as u32,
                    );
                }
            }
        }

        impl<'a> ::planus::ReadAsRoot<'a> for SpawnedInputRef<'a> {
            fn read_as_root(slice: &'a [u8]) -> ::planus::Result<Self> {
                ::planus::TableRead::from_buffer(
                    ::planus::SliceWithStartOffset {
                        buffer: slice,
                        offset_from_start: 0,
                    },
                    0,
                )
                .map_err(|error_kind| {
                    error_kind.with_error_location("[SpawnedInputRef]", "read_as_root", 0)
                })
            }
        }
    }
}
