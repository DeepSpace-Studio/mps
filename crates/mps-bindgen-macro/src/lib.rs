//! `#[java_struct]` / `#[java_enum]` / `#[java_field]` — inert marker
//! attributes for the `mps` Java-binding generator.
//!
//! 设计意图（与 `java-bindgen` crate 的 `#[gen_jni]` 不同）：本 crate 只提供
//! **直通注解 (passthrough attribute macro)**，宏本身不改变任何 Rust 代码，仅
//! 把注解留在 AST 上，供 `xtask gen-java` 用 `syn` 解析、按 `#[repr(C)]` 布局
//! 自动生成 Java 值类（POJO）。这样：
//!
//! 1. 零运行时依赖 —— 注解不引入任何 JNI/`jni` 代码，编译期即被宏“擦除”；
//! 2. 贴合现有 `ljni` + `Java_org_polaris2023_mps_rapier_RapierNative_*` 约定，
//!    Java 侧由 `RigidBodyNative` 的 `jni!` 导出函数驱动，而非 `java-bindgen`
//!    那套 `JniClass` 体系；
//! 3. generator 可跨 crate（mps-formula / mps-core / mps-cosmos）扫描，统一产出。
//!
//! 用法：
//!
//! ```rust
//! use mps_bindgen_macro::java_struct;
//!
//! #[repr(C)]
//! #[java_struct(package = "org.polaris2023.mps_rigid_body.ffi")]
//! pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }
//! ```
//!
//! 注解参数：
//! - `java_struct` / `java_enum`：`package = "..."`（Java 包名，缺省
//!   `org.polaris2023.mps.ffi`）、`class = "..."`（Java 类名，
//!   缺省取 Rust 标识符的 PascalCase）；
//! - `java_field`：`skip`（不生成该字段的 getter/setter）、
//!   `name = "..."`（Java 字段名覆盖）、`type = "..."`（Java 类型覆盖，
//!   覆盖布局推断）。
//!
//! 例如：
//!
//! ```rust
//! use mps_bindgen_macro::java_struct;
//!
//! #[repr(C)]
//! #[java_struct(package = "org.polaris2023.mps.ffi")]
//! pub struct Vec3 { pub x: f64, pub y: f64, pub z: f64 }
//! ```
//!
//! 生成器会把它写成一个 Java 源码文件（带 `package org.polaris2023.mps.ffi;`
//! 文件头），由调用方决定输出目录。

use proc_macro::TokenStream;
use quote::ToTokens;
use syn::{DeriveInput, ItemEnum, parse_macro_input};

/// 直通注解：标记一个 `#[repr(C)]` 结构体需要生成 Java 值类。
/// 宏体原样返回，不修改任何代码。
#[proc_macro_attribute]
pub fn java_struct(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    input.into_token_stream().into()
}

/// 直通注解：标记一个 `#[repr(C)]` 枚举需要生成 Java 枚举类。
#[proc_macro_attribute]
pub fn java_enum(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemEnum);
    input.into_token_stream().into()
}

/// 字段级直通注解：控制单个字段的生成（skip / name / type 覆盖）。
#[proc_macro_attribute]
pub fn java_field(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

// ===========================================================================
// `#[jni]` / `#[jni_e_c]` — JNI 导出注解宏
//
// 把 `mps-jni/src/lib.rs` 里现有的 `jni!` / `jni_e_c!` `macro_rules!` 机制
// 升级为注解式：调用点从 `jni!(int abiVersion() { ... });` 变成
// `#[jni] int abiVersion() { ... }`，展开后吐出**完全相同**的
// `Java_org_polaris2023_mps_rapier_RapierNative_<name>` C 符号（同样的
// `extern "system"` 签名、`catch_unwind` 包裹、同样的类型映射表），
// 因此 `RigidBodyNative.java` 一侧零改动。
//
// `jni` 与 `jni_e_c` 的区别（与原 macro_rules! 完全一致）：
//   * `jni`     会自动在参数最前补 `(_env: JNIEnv, _class: jclass, ...)`；
//   * `jni_e_c` 不在 DSL 里写 env/class，调用点须显式写在参数里
//                `（env _env, class _class, ...）`，宏原样透传。
// ===========================================================================

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Ident, Token, parenthesized,
    parse::{Parse, ParseStream},
};

/// DSL 中的类型名（不是 Rust 类型，是 `jni!` 自定义记号）。
enum JniTy {
    Long,
    Boolean,
    ByteArray,
    Double,
    Int,
    Void,
    DoubleArray,
    LongArray,
    BoolArray,
    String,
    Env,
    Class,
}

impl JniTy {
    fn parse_ident(ident: &Ident) -> Option<JniTy> {
        Some(match ident.to_string().as_str() {
            "long" => JniTy::Long,
            "boolean" => JniTy::Boolean,
            "byte_array" => JniTy::ByteArray,
            "double" => JniTy::Double,
            "int" => JniTy::Int,
            "void" => JniTy::Void,
            "double_array" => JniTy::DoubleArray,
            "long_array" => JniTy::LongArray,
            "bool_array" => JniTy::BoolArray,
            "String" => JniTy::String,
            "env" => JniTy::Env,
            "class" => JniTy::Class,
            _ => return None,
        })
    }

    /// 展开为真实 Rust 参数/返回类型（与 `jni!` 的 `@ty` 表一致）。
    fn rust_ty(&self) -> TokenStream2 {
        match self {
            JniTy::Long => quote!(jlong),
            JniTy::Boolean => quote!(jbyte),
            JniTy::ByteArray => quote!(jbyteArray),
            JniTy::Double => quote!(jdouble),
            JniTy::Int => quote!(jint),
            JniTy::Void => quote!(()),
            JniTy::DoubleArray => quote!(jdoubleArray),
            JniTy::LongArray => quote!(jlongArray),
            JniTy::BoolArray => quote!(jbooleanArray),
            JniTy::String => quote!(jstring),
            JniTy::Env => quote!(JNIEnv),
            JniTy::Class => quote!(jclass),
        }
    }

    /// panic 时的默认返回值（与 `jni!` 的 `@default` 表一致）。
    fn default_value(&self) -> TokenStream2 {
        match self {
            JniTy::Long | JniTy::Boolean | JniTy::Int => quote!(0),
            JniTy::Double => quote!(0.0),
            JniTy::Void => quote!(()),
            JniTy::ByteArray
            | JniTy::DoubleArray
            | JniTy::LongArray
            | JniTy::BoolArray
            | JniTy::String => quote!(::std::ptr::null_mut()),
            // env/class 不会作为返回类型出现
            JniTy::Env | JniTy::Class => quote!(()),
        }
    }
}

struct JniFn {
    ret: JniTy,
    name: Ident,
    args: Vec<(JniTy, Ident)>,
    body: syn::Block,
}

impl Parse for JniFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ret_ident: Ident = input.parse()?;
        let ret = JniTy::parse_ident(&ret_ident)
            .ok_or_else(|| input.error("unknown jni type in return position"))?;
        let name: Ident = input.parse()?;
        let content;
        parenthesized!(content in input);
        let mut args = Vec::new();
        while !content.is_empty() {
            let kind_ident: Ident = content.parse()?;
            let kind = JniTy::parse_ident(&kind_ident)
                .ok_or_else(|| content.error("unknown jni type in argument position"))?;
            let arg: Ident = content.parse()?;
            args.push((kind, arg));
            if content.peek(Token![,]) {
                content.parse::<Token![,]>()?;
            }
        }
        let body: syn::Block = input.parse()?;
        Ok(JniFn {
            ret,
            name,
            args,
            body,
        })
    }
}

/// 把 DSL 函数展开为 `Java_org_polaris2023_mps_rapier_RapierNative_<name>`
/// 导出函数。`prepend_env_class` 为 true 时（即 `jni`，非 `jni_e_c`），
/// 在参数最前补 `(_env: JNIEnv, _class: jclass, ...)`。
fn expand_jni(item: TokenStream, prepend_env_class: bool) -> TokenStream {
    let parsed = parse_macro_input!(item as JniFn);
    let name = &parsed.name;
    let ret_ty = parsed.ret.rust_ty();
    let ret_default = parsed.ret.default_value();
    let body = &parsed.body;

    // 参数列表：jni 先补 env/class，再透传 DSL 参数。
    let mut params: Vec<TokenStream2> = Vec::new();
    if prepend_env_class {
        params.push(quote!(_env: JNIEnv));
        params.push(quote!(_class: jclass));
    }
    for (kind, arg) in &parsed.args {
        let t = kind.rust_ty();
        params.push(quote!(#arg: #t));
    }

    let export_name = format!("Java_org_polaris2023_mps_rapier_RapierNative_{}", name);

    let expanded = quote! {
        #[unsafe(export_name = #export_name)]
        #[allow(non_snake_case)]
        pub extern "system" fn #name(#(#params),*) -> #ret_ty {
            match ::std::panic::catch_unwind(::std::panic::AssertUnwindSafe(|| #body)) {
                ::std::result::Result::Ok(value) => value,
                ::std::result::Result::Err(_) => {
                    er::set_error(er::ERR_INTERNAL, "internal panic");
                    #ret_default
                }
            }
        }
    };
    expanded.into()
}

/// `#[jni] int foo(...) { ... }` —— 自动补充 `(_env: JNIEnv, _class: jclass)`。
#[proc_macro_attribute]
pub fn jni(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_jni(item, true)
}

/// `#[jni_e_c] double_array foo(env _env, class _class, ...) { ... }`
/// —— env/class 由调用点显式写在参数里。
#[proc_macro_attribute]
pub fn jni_e_c(_attr: TokenStream, item: TokenStream) -> TokenStream {
    expand_jni(item, false)
}
