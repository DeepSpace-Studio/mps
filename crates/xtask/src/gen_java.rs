//! `gen-java` 子命令：扫描 `#[java_struct]` / `#[java_enum]` 注解，按
//! `#[repr(C)]` 布局自动生成 Java 值类（POJO）。
//!
//! 包名写在注解里：`#[java_struct(package = "org.polaris2023.mps.ffi")]`。
//! 缺省 `org.polaris2023.mps.ffi`。生成文件按包名落到
//! `test21/src/main/java/<包名路径>/<Name>.java`，并带上对应的 `package` 行。
//!
//! 每个生成的 Java 类：
//! - 持有一个 `NativeMemory mem` + `long offset`；
//! - 提供每字段 getter/setter（通过 `NativeMemory` 的 `getDouble`/`getLong`/
//!   `getInt`/`getBool`/`putXxx` 读写，不直接碰 `sun.misc.Unsafe`）；
//! - 提供 `sizeOf()` 返回 C 布局字节数；
//! - `java_enum` 生成一组 `public static final int` 常量 + `fromRaw`/`toRaw`。
//!
//! 布局计算遵循 C ABI 规则（与 cbindgen 生成的 `rigid_body.h` 对齐）：
//! - f64 → 8B,align 8
//! - u32/i32 → 4B,align 4
//! - u64 → 8B,align 8
//! - Bool(u8) → 1B,align 1
//! - Vec3/Quat/嵌套 struct → 递归 size,align = 成员最大 align
//! - 数组 [T; N] → N * sizeof(T),align = align(T)

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use quote::ToTokens;
use syn::{Item, ItemEnum, ItemStruct, Visibility};

use crate::JAVA_PACKAGE_DEFAULT;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct FieldInfo {
    pub rust_name: String,
    pub java_name: String,
    pub rust_type: String,
    pub java_type: String,
    pub offset: u64,
    pub size: u64,
    pub kind: FieldKind,
    pub skip: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FieldKind {
    Double,
    Float,
    U32,
    I32,
    U64,
    I64,
    Bool,
    Struct(String),
    Enum(String),
    Array(Box<FieldKind>, usize),
}

#[derive(Debug, Clone)]
pub struct StructInfo {
    pub name: String,
    pub java_name: String,
    pub fields: Vec<FieldInfo>,
    pub size: u64,
    pub align: u64,
    pub package: String,
}

#[derive(Debug, Clone)]
pub struct EnumInfo {
    pub name: String,
    pub java_name: String,
    pub variants: Vec<(String, i64)>,
    pub package: String,
}

pub fn run(workspace_root: &Path, output_dir: Option<&str>) -> Result<String, String> {
    // Scan all #[repr(C)] structs/enums with #[java_struct]/#[java_enum] in
    // mps-formula + mps-core ffi types.
    let scan_dirs = [
        workspace_root.join("crates/mps-formula/src/ffi/types"),
        workspace_root.join("crates/mps-core/src/rapier/ffi"),
    ];

    let mut structs: BTreeMap<String, StructInfo> = BTreeMap::new();
    let mut enums: BTreeMap<String, EnumInfo> = BTreeMap::new();

    for dir in &scan_dirs {
        if !dir.is_dir() {
            continue;
        }
        for entry in walkdir::WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("rs") {
                continue;
            }
            let content = std::fs::read_to_string(path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            let file =
                syn::parse_file(&content).map_err(|e| format!("parse {}: {e}", path.display()))?;

            for item in &file.items {
                match item {
                    Item::Struct(s) => {
                        if has_attr(&s.attrs, "java_struct")
                            && is_repr_c(&s.attrs)
                            && is_pub(&s.vis)
                        {
                            let si = parse_struct(s, &structs, &enums)?;
                            structs.insert(si.name.clone(), si);
                        }
                    }
                    Item::Enum(e) => {
                        if has_attr(&e.attrs, "java_enum") && is_repr_c(&e.attrs) && is_pub(&e.vis)
                        {
                            let ei = parse_enum(e)?;
                            enums.insert(ei.name.clone(), ei);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Two-pass: after collecting all, re-resolve nested struct sizes/aligns
    // (fields reference earlier structs/enums which are now all known).
    let mut all_structs: Vec<StructInfo> = structs.values().cloned().collect();
    for s in &mut all_structs {
        finalize_layout(s, &structs, &enums);
    }

    let out_base = match output_dir {
        Some(d) => PathBuf::from(d),
        // Java source root: each file is placed under <root>/<package-as-path>/
        None => workspace_root.join("test21/src/main/java"),
    };
    std::fs::create_dir_all(&out_base).map_err(|e| format!("mkdir {}: {e}", out_base.display()))?;

    let mut count = 0usize;
    for s in &all_structs {
        let java = gen_struct_java(s);
        let path = package_dir(&out_base, &s.package).join(format!("{}.java", s.java_name));
        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| format!("mkdir {}: {e}", path.display()))?;
        std::fs::write(&path, &java).map_err(|e| format!("write {}: {e}", path.display()))?;
        count += 1;
    }
    for e in enums.values() {
        let java = gen_enum_java(e);
        let path = package_dir(&out_base, &e.package).join(format!("{}.java", e.java_name));
        std::fs::create_dir_all(path.parent().unwrap())
            .map_err(|e| format!("mkdir {}: {e}", path.display()))?;
        std::fs::write(&path, &java).map_err(|e| format!("write {}: {e}", path.display()))?;
        count += 1;
    }

    Ok(format!(
        "xtask gen-java: {} classes → {}\n  ({} structs, {} enums)",
        count,
        out_base.display(),
        all_structs.len(),
        enums.len()
    ))
}

fn has_attr(attrs: &[syn::Attribute], name: &str) -> bool {
    attrs.iter().any(|a| {
        if let Some(p) = a.path().get_ident() {
            p == name
        } else {
            false
        }
    })
}

/// Read `#[java_struct(package = "org.x.y")]` (or `#[java_enum(...)]`) and
/// return the explicit package, falling back to the default when the
/// attribute carries no `package = "..."` argument.
fn attr_package(attrs: &[syn::Attribute], name: &str) -> Option<String> {
    for a in attrs {
        if !a.path().is_ident(name) {
            continue;
        }
        // Accept `#[java_struct(package = "X")]` (Meta::List) and
        // `#[java_struct = "X"]` (Meta::NameValue) forms.
        match &a.meta {
            syn::Meta::List(ml) => {
                let mut found: Option<String> = None;
                let _ = ml.parse_nested_meta(|meta| {
                    if meta.path.is_ident("package") {
                        if let Ok(v) = meta.value()?.parse::<syn::LitStr>() {
                            found = Some(v.value());
                        }
                    }
                    Ok(())
                });
                if let Some(p) = found {
                    return Some(p);
                }
            }
            syn::Meta::NameValue(nv) if nv.path.is_ident("package") => {
                if let syn::Expr::Lit(expr) = &nv.value {
                    if let syn::Lit::Str(s) = &expr.lit {
                        return Some(s.value());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

fn is_repr_c(attrs: &[syn::Attribute]) -> bool {
    attrs.iter().any(|a| {
        if a.path().is_ident("repr") {
            if let syn::Meta::List(ml) = &a.meta {
                return ml.tokens.to_string().contains("C");
            }
        }
        false
    })
}

fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

fn parse_struct(
    s: &ItemStruct,
    known_structs: &BTreeMap<String, StructInfo>,
    known_enums: &BTreeMap<String, EnumInfo>,
) -> Result<StructInfo, String> {
    let name = s.ident.to_string();
    let java_name = rust_to_pascal(&name);
    let package =
        attr_package(&s.attrs, "java_struct").unwrap_or_else(|| JAVA_PACKAGE_DEFAULT.to_string());

    let mut fields = Vec::new();
    let mut offset: u64 = 0;
    let mut max_align: u64 = 1;

    for f in &s.fields {
        let field_name = match &f.ident {
            Some(id) => id.to_string(),
            None => continue, // tuple struct like Bool(u8)
        };
        // Check #[java_field(skip)]
        let skip = f.attrs.iter().any(|a| {
            if let Some(p) = a.path().get_ident() {
                if p == "java_field" {
                    return a.meta.to_token_stream().to_string().contains("skip");
                }
            }
            false
        });

        let rust_type = f.ty.to_token_stream().to_string().replace(' ', "");
        let kind = classify_type(&rust_type, known_structs, known_enums);
        let size = kind_size(&kind, known_structs);
        let align = kind_align(&kind, known_structs);
        // C struct layout: pad to align, then place
        offset = align_up(offset, align);
        let java_name_f = rust_field_to_java(&field_name);
        let java_type = kind_java_type(&kind, known_structs);
        fields.push(FieldInfo {
            rust_name: field_name,
            java_name: java_name_f,
            rust_type,
            java_type,
            offset,
            size,
            kind,
            skip,
        });
        offset += size;
        if align > max_align {
            max_align = align;
        }
    }

    let size = align_up(offset, max_align);
    Ok(StructInfo {
        name,
        java_name,
        fields,
        size,
        align: max_align,
        package,
    })
}

fn parse_enum(e: &ItemEnum) -> Result<EnumInfo, String> {
    let name = e.ident.to_string();
    let java_name = rust_to_pascal(&name);
    let package =
        attr_package(&e.attrs, "java_enum").unwrap_or_else(|| JAVA_PACKAGE_DEFAULT.to_string());
    let mut variants: Vec<(String, i64)> = Vec::new();
    for v in &e.variants {
        let vname = v.ident.to_string();
        let value: i64 = match &v.discriminant {
            Some((_id, expr)) => {
                let s = expr.to_token_stream().to_string().replace(' ', "");
                s.parse::<i64>().unwrap_or(0)
            }
            None => {
                if variants.is_empty() {
                    0
                } else {
                    variants.last().unwrap().1 + 1
                }
            }
        };
        variants.push((vname, value));
    }
    Ok(EnumInfo {
        name,
        java_name,
        variants,
        package,
    })
}

fn finalize_layout(
    s: &mut StructInfo,
    _known: &BTreeMap<String, StructInfo>,
    _enums: &BTreeMap<String, EnumInfo>,
) {
    // Already computed in parse_struct; this is a no-op placeholder for future
    // two-pass fixes (e.g., forward references).
    let _ = s;
}

fn classify_type(
    rust: &str,
    structs: &BTreeMap<String, StructInfo>,
    enums: &BTreeMap<String, EnumInfo>,
) -> FieldKind {
    let r = rust.trim();
    // Array [T; N]
    if r.starts_with('[') && r.ends_with(']') {
        let inner = r.trim_start_matches('[').trim_end_matches(']');
        // split on last ';'
        if let Some(pos) = inner.rfind(';') {
            let ty_str = inner[..pos].trim();
            let n_str = inner[pos + 1..].trim();
            if let Ok(n) = n_str.parse::<usize>() {
                let inner_kind = classify_type(ty_str, structs, enums);
                return FieldKind::Array(Box::new(inner_kind), n);
            }
        }
    }
    match r {
        "f64" | "f32" => {
            if r == "f64" {
                FieldKind::Double
            } else {
                FieldKind::Float
            }
        }
        "u32" => FieldKind::U32,
        "i32" => FieldKind::I32,
        "u64" | "usize" => FieldKind::U64,
        "i64" | "isize" => FieldKind::I64,
        "bool" | "Bool" => FieldKind::Bool,
        _ => {
            if enums.contains_key(r) {
                FieldKind::Enum(r.to_string())
            } else if structs.contains_key(r) {
                FieldKind::Struct(r.to_string())
            } else {
                // Unknown → treat as u64 (handles like RigidBodyHandleRaw)
                FieldKind::U64
            }
        }
    }
}

fn kind_size(kind: &FieldKind, structs: &BTreeMap<String, StructInfo>) -> u64 {
    match kind {
        FieldKind::Double | FieldKind::U64 | FieldKind::I64 => 8,
        FieldKind::Float | FieldKind::U32 | FieldKind::I32 => 4,
        FieldKind::Bool => 1,
        FieldKind::Struct(name) => structs.get(name).map(|s| s.size).unwrap_or(24),
        FieldKind::Enum(_) => 4,
        FieldKind::Array(inner, n) => kind_size(inner, structs) * (*n as u64),
    }
}

fn kind_align(kind: &FieldKind, structs: &BTreeMap<String, StructInfo>) -> u64 {
    match kind {
        FieldKind::Double | FieldKind::U64 | FieldKind::I64 => 8,
        FieldKind::Float | FieldKind::U32 | FieldKind::I32 | FieldKind::Enum(_) => 4,
        FieldKind::Bool => 1,
        FieldKind::Struct(name) => structs.get(name).map(|s| s.align).unwrap_or(8),
        FieldKind::Array(inner, _) => kind_align(inner, structs),
    }
}

fn kind_java_type(kind: &FieldKind, structs: &BTreeMap<String, StructInfo>) -> String {
    match kind {
        FieldKind::Double => "double".into(),
        FieldKind::Float => "float".into(),
        FieldKind::U32 | FieldKind::I32 | FieldKind::Enum(_) => "int".into(),
        FieldKind::U64 | FieldKind::I64 => "long".into(),
        FieldKind::Bool => "boolean".into(),
        FieldKind::Struct(name) => structs
            .get(name)
            .map(|s| s.java_name.clone())
            .unwrap_or_else(|| rust_to_pascal(name)),
        FieldKind::Array(_, _) => "double[]".into(),
    }
}

fn align_up(offset: u64, align: u64) -> u64 {
    if align == 0 {
        return offset;
    }
    (offset + align - 1) & !(align - 1)
}

/// `org.polaris2023.mps.ffi` → `<root>/org/polaris2023/mps/ffi`.
fn package_dir(root: &Path, package: &str) -> PathBuf {
    let mut dir = root.to_path_buf();
    for seg in package.split('.') {
        dir = dir.join(seg);
    }
    dir
}

fn rust_to_pascal(s: &str) -> String {
    // Vec3 → Vec3 (already Pascal). snake_case → PascalCase.
    let mut out = String::new();
    let mut upper = true;
    for ch in s.chars() {
        if ch == '_' {
            upper = true;
            continue;
        }
        if upper {
            out.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn rust_field_to_java(s: &str) -> String {
    // snake_case → camelCase for getter/setter name base
    let mut chars = s.chars();
    let first = chars
        .next()
        .map(|c| c.to_ascii_lowercase())
        .unwrap_or_default();
    let rest: String = chars.collect();
    let mut out = String::new();
    out.push(first);
    // Convert rest: _x → X (camelCase)
    let mut upper = false;
    for ch in rest.chars() {
        if ch == '_' {
            upper = true;
            continue;
        }
        if upper {
            out.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

fn gen_enum_java(e: &EnumInfo) -> String {
    let mut sb = String::new();
    sb.push_str(&format!(
        "// Auto-generated by `cargo run -p xtask -- gen-java`. Do NOT edit by hand.\n"
    ));
    sb.push_str(&format!("package {};\n\n", e.package));
    sb.push_str(&format!("public final class {} {{\n", e.java_name));
    sb.push_str(&format!("    private {}() {{}}\n\n", e.java_name));
    for (vn, val) in &e.variants {
        sb.push_str(&format!(
            "    public static final int {} = {};\n",
            pascal_to_screaming_snake(vn),
            val
        ));
    }
    sb.push_str("\n");
    sb.push_str(&format!(
        "    public static int fromRaw(int raw) {{\n        return raw;\n    }}\n\n"
    ));
    sb.push_str(&format!(
        "    public static int toRaw(int value) {{\n        return value;\n    }}\n"
    ));
    sb.push_str("}\n");
    sb
}

fn gen_struct_java(s: &StructInfo) -> String {
    let mut sb = String::new();
    sb.push_str("// Auto-generated by `cargo run -p xtask -- gen-java`. Do NOT edit by hand.\n");
    sb.push_str(&format!("package {};\n\n", s.package));
    sb.push_str("import org.polaris2023.mps_rigid_body.util.NativeMemory;\n\n");

    sb.push_str(&format!("public final class {} {{\n", s.java_name));
    sb.push_str(&format!(
        "    public static final long SIZEOF = {}L;\n\n",
        s.size
    ));
    sb.push_str("    private final NativeMemory mem;\n");
    sb.push_str("    private final long offset;\n\n");

    // Constructor from a NativeMemory + offset (mirrors VoxelBuildStats/statsFrom idiom)
    sb.push_str(&format!(
        "    public {jt}(NativeMemory mem, long offset) {{\n        this.mem = mem;\n        this.offset = offset;\n    }}\n\n",
        jt = s.java_name
    ));

    sb.push_str("    public static long sizeOf() {\n        return SIZEOF;\n    }\n\n");
    sb.push_str("    public NativeMemory mem() {\n        return mem;\n    }\n\n");
    sb.push_str("    public long offset() {\n        return offset;\n    }\n\n");

    for f in &s.fields {
        if f.skip {
            continue;
        }
        gen_getter(&mut sb, s, f);
        gen_setter(&mut sb, s, f);
    }

    sb.push_str("}\n");
    sb
}

fn gen_getter(sb: &mut String, _s: &StructInfo, f: &FieldInfo) {
    let getter_name = f.java_name.clone(); // camelCase: x, y, halfExtents
    match &f.kind {
        FieldKind::Double => {
            sb.push_str(&format!(
                "    public double {g}() {{\n        return mem.getDouble(offset + {off}L);\n    }}\n\n",
                g = getter_name,
                off = f.offset,
            ));
        }
        FieldKind::Float => {
            sb.push_str(&format!(
                "    public float {g}() {{\n        return (float) mem.getDouble(offset + {off}L);\n    }}\n\n",
                g = getter_name,
                off = f.offset,
            ));
        }
        FieldKind::U32 | FieldKind::I32 | FieldKind::Enum(_) => {
            sb.push_str(&format!(
                "    public int {g}() {{\n        return mem.getInt(offset + {off}L);\n    }}\n\n",
                g = getter_name,
                off = f.offset,
            ));
        }
        FieldKind::U64 | FieldKind::I64 => {
            sb.push_str(&format!(
                "    public long {g}() {{\n        return mem.getLong(offset + {off}L);\n    }}\n\n",
                g = getter_name,
                off = f.offset,
            ));
        }
        FieldKind::Bool => {
            sb.push_str(&format!(
                "    public boolean {g}() {{\n        return mem.getBool(offset + {off}L);\n    }}\n\n",
                g = getter_name,
                off = f.offset,
            ));
        }
        FieldKind::Struct(_) => {
            // Return a nested wrapper reusing the same NativeMemory at offset
            let java_type = f.java_type.clone();
            sb.push_str(&format!(
                "    public {jt} {g}() {{\n        return new {jt}(mem, offset + {off}L);\n    }}\n\n",
                jt = java_type,
                g = getter_name,
                off = f.offset,
            ));
        }
        FieldKind::Array(inner, n) => {
            let elem_size = match inner.as_ref() {
                FieldKind::Double => 8,
                FieldKind::Float => 4,
                _ => 24, // assume Vec3
            };
            sb.push_str(&format!(
                "    public double[] {g}() {{\n        double[] out = new double[{n}];\n        for (int i = 0; i < {n}; i++) {{\n            out[i] = mem.getDouble(offset + {off}L + (long)i * {es}L);\n        }}\n        return out;\n    }}\n\n",
                g = getter_name,
                n = n,
                off = f.offset,
                es = elem_size,
            ));
        }
    }
}

fn gen_setter(sb: &mut String, _s: &StructInfo, f: &FieldInfo) {
    let setter_name = format!("set{}", capitalize_first(&f.java_name));
    match &f.kind {
        FieldKind::Double => {
            sb.push_str(&format!(
                "    public void {g}(double value) {{\n        mem.putDouble(offset + {off}L, value);\n    }}\n\n",
                g = setter_name,
                off = f.offset,
            ));
        }
        FieldKind::Float => {
            sb.push_str(&format!(
                "    public void {g}(float value) {{\n        mem.putDouble(offset + {off}L, value);\n    }}\n\n",
                g = setter_name,
                off = f.offset,
            ));
        }
        FieldKind::U32 | FieldKind::I32 | FieldKind::Enum(_) => {
            sb.push_str(&format!(
                "    public void {g}(int value) {{\n        mem.putInt(offset + {off}L, value);\n    }}\n\n",
                g = setter_name,
                off = f.offset,
            ));
        }
        FieldKind::U64 | FieldKind::I64 => {
            sb.push_str(&format!(
                "    public void {g}(long value) {{\n        mem.putLong(offset + {off}L, value);\n    }}\n\n",
                g = setter_name,
                off = f.offset,
            ));
        }
        FieldKind::Bool => {
            sb.push_str(&format!(
                "    public void {g}(boolean value) {{\n        mem.putByte(offset + {off}L, value ? 1 : 0);\n    }}\n\n",
                g = setter_name,
                off = f.offset,
            ));
        }
        FieldKind::Struct(_) => {
            // Copy SIZEOF bytes from another instance, 8-byte chunks, via
            // NativeMemory get/put (no sun.misc.Unsafe in generated code).
            let jt = f.java_type.clone();
            sb.push_str(&format!(
                "    public void {g}({jt} value) {{\n        for (long k = 0L; k < {sz}L; k += 8L) {{\n            mem.putLong(offset + {off}L + k, value.mem().getLong(value.offset() + k));\n        }}\n    }}\n\n",
                g = setter_name,
                jt = jt,
                off = f.offset,
                sz = f.size,
            ));
        }
        FieldKind::Array(inner, n) => {
            let elem_size = match inner.as_ref() {
                FieldKind::Double => 8,
                FieldKind::Float => 4,
                _ => 24,
            };
            sb.push_str(&format!(
                "    public void {g}(double[] values) {{\n        for (int i = 0; i < {n}; i++) {{\n            mem.putDouble(offset + {off}L + (long)i * {es}L, values[i]);\n        }}\n    }}\n\n",
                g = setter_name,
                n = n,
                off = f.offset,
                es = elem_size,
            ));
        }
    }
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_ascii_uppercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

/// `KinematicPositionBased` → `KINEMATIC_POSITION_BASED` (SCREAMING_SNAKE_CASE).
fn pascal_to_screaming_snake(s: &str) -> String {
    let mut out = String::new();
    let mut prev_lower = false;
    for ch in s.chars() {
        if ch.is_ascii_uppercase() {
            if prev_lower {
                out.push('_');
            }
            out.push(ch.to_ascii_uppercase());
            prev_lower = false;
        } else {
            out.push(ch.to_ascii_uppercase());
            prev_lower = true;
        }
    }
    out
}
