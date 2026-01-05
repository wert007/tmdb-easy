use std::{collections::HashMap, env, fs::File, io::BufWriter, path::PathBuf};

use convert_case::Casing;

mod schema;

struct FunctionParameter {
    name: String,
    type_: String,
    is_required: bool,
}

struct FunctionDefinition {
    doc_comment: String,
    name: String,
    parameters: Vec<FunctionParameter>,
    response: String,
    body: String,
}

struct TypeDefinition {
    name: String,
    ref_name: String,
    definition: Option<String>,
}
impl TypeDefinition {
    fn from_type_schema(schema: &schema::TypeSchema, name: String) -> Vec<TypeDefinition> {
        use std::fmt::Write;
        let schema::TypeSchema::Tagged(schema) = schema else {
            return Vec::new();
        };
        let mut result = Vec::new();
        match schema {
            schema::TaggedTypeSchema::Object { properties } => {
                // let mut fields = HashMap::new();
                let name = name.to_case(convert_case::Case::Pascal);
                let mut definition = format!("pub struct {name} {{\n");
                for (field_name, field_type) in properties {
                    let mut field_types =
                        TypeDefinition::from_type_schema(field_type, combine(&name, field_name));
                    if field_types.is_empty() {
                        // WARN: This means we are skipping fields. but those
                        // are also very much not defined. so. should be ok for
                        // now.
                        continue;
                    }
                    let field_name = field_name
                        .replace('.', "_")
                        .to_case(convert_case::Case::Snake);
                    let field_name = if ["type", "do", "in"].contains(&field_name.as_str()) {
                        format!("r#{field_name}")
                    } else {
                        field_name
                    };
                    result.append(&mut field_types);
                    writeln!(
                        definition,
                        "#[serde(default, deserialize_with = \"null_to_default\")]"
                    )
                    .unwrap();
                    writeln!(
                        definition,
                        "    pub {field_name}: {},",
                        result.last().unwrap().name
                    )
                    .unwrap();
                }
                writeln!(definition, "}}").unwrap();
                result.push(TypeDefinition {
                    ref_name: name.clone(),
                    name,
                    definition: Some(definition),
                });
            }
            schema::TaggedTypeSchema::Boolean { .. } => {
                result.push(Self {
                    ref_name: "bool".into(),
                    name: "bool".into(),
                    definition: None,
                });
            }
            schema::TaggedTypeSchema::String { .. } => {
                result.push(Self {
                    ref_name: "&str".into(),
                    name: "String".into(),
                    definition: None,
                });
            }
            schema::TaggedTypeSchema::Integer { .. } => {
                result.push(Self {
                    ref_name: "i64".into(),
                    name: "i64".into(),
                    definition: None,
                });
            }
            schema::TaggedTypeSchema::Number { .. } => {
                result.push(Self {
                    ref_name: "f64".into(),
                    name: "f64".into(),
                    definition: None,
                });
            }
            schema::TaggedTypeSchema::Array { items } => {
                let mut items = Self::from_type_schema(items, name);
                if items.is_empty() {
                    return Vec::new();
                }
                result.append(&mut items);
                result.push(Self {
                    ref_name: format!("&[{}]", result.last().unwrap().name),
                    name: format!("Vec<{}>", result.last().unwrap().name),
                    definition: None,
                });
            }
            schema::TaggedTypeSchema::Empty => unreachable!(),
        }
        result
    }
}

fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    use std::io::Write;
    let base_dir = env::var_os("OUT_DIR").unwrap();
    let schema: schema::Schema = schema::Schema::download();
    let needed_types: Vec<TypeDefinition> = collect_types_for(&schema);
    let mut w =
        BufWriter::new(File::create(PathBuf::from(base_dir.clone()).join("types.rs")).unwrap());
    writeln!(
        w,
        "
use serde::Deserialize;

fn null_to_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + Default,
{{
    Option::<T>::deserialize(deserializer)
        .map(|opt| opt.unwrap_or_default())
    }}"
    )
    .unwrap();
    for t in needed_types {
        if let Some(def) = t.definition {
            writeln!(
                w,
                "#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]"
            )
            .unwrap();
            writeln!(w, "{def}").unwrap();
        }
    }
    let raw_functions: Vec<FunctionDefinition> = collect_functions_for(
        &schema,
        &FunctionDefinitionArgs {
            is_async: false,
            client_parameter: "client: &reqwest::blocking::Client".into(),
            base_url: schema.servers.0[0].url.clone(),
        },
    );
    if cfg!(feature = "blocking") {
        let mut w = BufWriter::new(
            File::create(PathBuf::from(base_dir.clone()).join("functions.rs")).unwrap(),
        );
        writeln!(w, "use crate::types::*;").unwrap();
        for f in &raw_functions {
            writeln!(w, "{}", f.doc_comment).unwrap();
            writeln!(w, "#[inline]").unwrap();
            write!(w, "pub fn {}(", f.name).unwrap();
            write!(w, "client: &reqwest::blocking::Client, api_key: &str, ").unwrap();
            for p in &f.parameters {
                write!(w, "{}: {}, ", p.name, p.type_).unwrap();
            }
            writeln!(w, ") -> Result<{}, crate::Error> {{", f.response).unwrap();
            writeln!(w, "{}", f.body).unwrap();
            writeln!(w, "}}").unwrap();
        }

        let mut w = BufWriter::new(
            File::create(PathBuf::from(base_dir.clone()).join("parametrized_functions.rs"))
                .unwrap(),
        );
        writeln!(w, "use crate::{{parameter_types::*, types::*, functions}};").unwrap();
        for f in raw_functions {
            let parameter_name = {
                let mut it = f.name.to_case(convert_case::Case::Pascal);
                it.push_str("Parameter");
                it
            };
            writeln!(w, "{}", f.doc_comment).unwrap();
            writeln!(w, "#[inline]").unwrap();
            write!(w, "pub fn {}(", f.name).unwrap();
            write!(w, "client: &reqwest::blocking::Client, api_key: &str, ").unwrap();
            let mut has_optional_fields = false;
            for p in &f.parameters {
                if p.is_required {
                    write!(w, "{}: {}, ", p.name, p.type_).unwrap();
                } else {
                    has_optional_fields = true;
                }
            }
            if has_optional_fields {
                writeln!(w, ") -> Result<{}, crate::Error> {{", f.response).unwrap();
                writeln!(w, "    {}_with_parameter(client, api_key,", f.name).unwrap();
                for p in &f.parameters {
                    if p.is_required {
                        write!(w, "{},", p.name).unwrap();
                    }
                }
                writeln!(w, "        Default::default()\n    )\n}}").unwrap();
                writeln!(w, "{}", f.doc_comment).unwrap();
                writeln!(w, "#[inline]").unwrap();
                write!(w, "pub fn {}_with_parameter(", f.name).unwrap();
                write!(w, "client: &reqwest::blocking::Client, api_key: &str, ").unwrap();
                for p in &f.parameters {
                    if p.is_required {
                        write!(w, "{}: {}, ", p.name, p.type_).unwrap();
                    }
                }

                write!(w, "remaining: {parameter_name}",).unwrap();
            }
            writeln!(w, ") -> Result<{}, crate::Error> {{", f.response).unwrap();
            if has_optional_fields {
                writeln!(w, "    let {parameter_name} {{").unwrap();
                for p in &f.parameters {
                    if !p.is_required {
                        writeln!(w, "        {},", p.name).unwrap();
                    }
                }
                writeln!(w, "    }} = remaining;").unwrap();
            }
            writeln!(
                w,
                "    functions::{}(
        client,
        api_key,",
                f.name
            )
            .unwrap();
            for p in &f.parameters {
                writeln!(
                    w,
                    "        {}{},",
                    p.name,
                    if !p.is_required && p.type_.contains("&str") {
                        ".as_deref()"
                    } else {
                        ""
                    }
                )
                .unwrap();
            }
            writeln!(w, "    )").unwrap();
            writeln!(w, "}}").unwrap();
        }
    }
    let async_functions: Vec<FunctionDefinition> = collect_functions_for(
        &schema,
        &FunctionDefinitionArgs {
            is_async: true,
            client_parameter: "client: &reqwest::Client".into(),
            base_url: schema.servers.0[0].url.clone(),
        },
    );
    if cfg!(feature = "async") {
        let mut w = BufWriter::new(
            File::create(PathBuf::from(base_dir.clone()).join("async_functions.rs")).unwrap(),
        );
        writeln!(w, "use crate::types::*;").unwrap();
        for f in &async_functions {
            writeln!(w, "{}", f.doc_comment).unwrap();
            writeln!(w, "#[inline]").unwrap();
            write!(w, "pub async fn {}(", f.name).unwrap();
            write!(w, "client: &reqwest::Client, api_key: &str, ").unwrap();
            for p in &f.parameters {
                write!(w, "{}: {}, ", p.name, p.type_).unwrap();
            }
            writeln!(w, ") -> Result<{}, crate::Error> {{", f.response).unwrap();
            writeln!(w, "{}", f.body).unwrap();
            writeln!(w, "}}").unwrap();
        }

        let mut w = BufWriter::new(
            File::create(PathBuf::from(base_dir.clone()).join("async_parametrized_functions.rs"))
                .unwrap(),
        );
        writeln!(
            w,
            "use crate::{{parameter_types::*, types::*, async_functions}};"
        )
        .unwrap();
        for f in async_functions {
            let parameter_name = {
                let mut it = f.name.to_case(convert_case::Case::Pascal);
                it.push_str("Parameter");
                it
            };
            writeln!(w, "{}", f.doc_comment).unwrap();
            writeln!(w, "#[inline]").unwrap();
            write!(w, "pub async fn {}(", f.name).unwrap();
            write!(w, "client: &reqwest::Client, api_key: &str, ").unwrap();
            let mut has_optional_fields = false;
            for p in &f.parameters {
                if p.is_required {
                    write!(w, "{}: {}, ", p.name, p.type_).unwrap();
                } else {
                    has_optional_fields = true;
                }
            }
            if has_optional_fields {
                writeln!(w, ") -> Result<{}, crate::Error> {{", f.response).unwrap();
                writeln!(w, "    {}_with_parameter(client, api_key,", f.name).unwrap();
                for p in &f.parameters {
                    if p.is_required {
                        write!(w, "{},", p.name).unwrap();
                    }
                }
                writeln!(w, "        Default::default()\n    ).await\n}}").unwrap();
                writeln!(w, "{}", f.doc_comment).unwrap();
                writeln!(w, "#[inline]").unwrap();
                write!(w, "pub async fn {}_with_parameter(", f.name).unwrap();
                write!(w, "client: &reqwest::Client, api_key: &str, ").unwrap();
                for p in &f.parameters {
                    if p.is_required {
                        write!(w, "{}: {}, ", p.name, p.type_).unwrap();
                    }
                }

                write!(w, "remaining: {parameter_name}",).unwrap();
            }
            writeln!(w, ") -> Result<{}, crate::Error> {{", f.response).unwrap();
            if has_optional_fields {
                writeln!(w, "    let {parameter_name} {{").unwrap();
                for p in &f.parameters {
                    if !p.is_required {
                        writeln!(w, "        {},", p.name).unwrap();
                    }
                }
                writeln!(w, "    }} = remaining;").unwrap();
            }
            writeln!(
                w,
                "    async_functions::{}(
        client,
        api_key,",
                f.name
            )
            .unwrap();
            for p in &f.parameters {
                writeln!(w, "        {},", p.name).unwrap();
            }
            writeln!(w, "    ).await").unwrap();
            writeln!(w, "}}").unwrap();
        }
    }
    let parameter_types: Vec<TypeDefinition> = collect_parameter_types_for(&schema);

    let mut w = BufWriter::new(
        File::create(PathBuf::from(base_dir.clone()).join("parameter_types.rs")).unwrap(),
    );
    writeln!(w, "use crate::types::*;").unwrap();
    for t in parameter_types {
        if let Some(def) = t.definition {
            writeln!(
                w,
                "#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]"
            )
            .unwrap();
            writeln!(w, "{def}").unwrap();
        }
    }
}

fn collect_parameter_types_for(schema: &schema::Schema) -> Vec<TypeDefinition> {
    let mut result = Vec::new();
    for (_, schema) in &schema.paths {
        if let Some(get) = &schema.get {
            collect_parameter_types_in_path_route(get, &mut result, get.operation_id.clone());
        }
    }
    result
}

fn collect_parameter_types_in_path_route(
    get: &schema::PathRoute,
    result: &mut Vec<TypeDefinition>,
    name: String,
) {
    use std::fmt::Write;
    let name = name.to_case(convert_case::Case::Pascal);
    let needs_references = get
        .parameters
        .iter()
        .filter(|p| !p.required)
        .filter_map(|p| TypeDefinition::from_type_schema(&p.schema, String::new()).pop())
        .any(|t| t.name == "String");
    let mut definition = format!(
        "pub struct {name}Parameter{} {{\n",
        if needs_references { "<'a>" } else { "" }
    );
    let mut has_fields = false;
    for p in get.parameters.iter().filter(|p| !p.required) {
        has_fields = true;
        let field_name = &p.name;
        let field_types = TypeDefinition::from_type_schema(&p.schema, combine(&name, field_name));
        if field_types.is_empty() {
            return;
        }
        let field_name = field_name
            .replace('.', "_")
            .to_case(convert_case::Case::Snake);
        let field_name = if ["type", "do", "in"].contains(&field_name.as_str()) {
            format!("r#{field_name}")
        } else {
            field_name
        };
        if let Some(description) = &p.description {
            writeln!(definition, "    /// {description}").unwrap();
        }
        writeln!(definition, "    #[serde(default)]").unwrap();
        let field_ref_name = field_types.last().unwrap().ref_name.clone();
        writeln!(
            definition,
            "    pub {field_name}: Option<{}>,",
            field_ref_name
                .strip_prefix('&')
                .map(|t| format!("std::borrow::Cow<'a, {t}>"))
                .unwrap_or(field_ref_name),
        )
        .unwrap();
    }
    writeln!(definition, "}}").unwrap();
    if has_fields {
        result.push(TypeDefinition {
            ref_name: name.clone(),
            name,
            definition: Some(definition),
        });
    }
}

struct FunctionDefinitionArgs {
    is_async: bool,
    client_parameter: String,
    base_url: String,
}

fn collect_functions_for(
    schema: &schema::Schema,
    fun: &FunctionDefinitionArgs,
) -> Vec<FunctionDefinition> {
    let mut result = Vec::new();
    for (path, schema) in &schema.paths {
        if let Some(get) = &schema.get {
            collect_functions_in_path_route(
                path,
                get,
                &mut result,
                get.operation_id.clone(),
                fun,
                "get",
            );
        }
    }
    result
}

fn collect_functions_in_path_route(
    path: &str,
    get: &schema::PathRoute,
    result: &mut Vec<FunctionDefinition>,
    namespace: String,
    fun: &FunctionDefinitionArgs,
    request_function_name: &'static str,
) {
    use std::fmt::Write;
    let name = get.operation_id.to_case(convert_case::Case::Snake);
    let mut parameters = Vec::new();
    let path_parameters: HashMap<&String, String> = get
        .parameters
        .iter()
        .filter(|p| p.r#in.path())
        .map(|p| {
            (
                &p.name,
                p.name.replace('.', "_").to_case(convert_case::Case::Snake),
            )
        })
        .collect();
    let mut path = path.to_string();
    for (n, v) in path_parameters {
        path = path.replace(&format!("{{{n}}}"), &format!("{{{v}}}"));
    }
    for p in &get.parameters {
        let name = p.name.replace('.', "_").to_case(convert_case::Case::Snake);
        let Some(type_) =
            TypeDefinition::from_type_schema(&p.schema, combine(&namespace, &name)).pop()
        else {
            return;
        };
        parameters.push(FunctionParameter {
            name,
            type_: if p.required {
                type_.ref_name
            } else {
                format!("Option<{}>", type_.ref_name)
            },
            is_required: p.required,
        });
    }
    let mut responses = Vec::new();
    for (response_kind, route_response) in &get.responses {
        assert_eq!(route_response.content.len(), 1);

        let Some(type_) = TypeDefinition::from_type_schema(
            &route_response.content.iter().next().unwrap().1.schema,
            combine(&namespace, format!("Response{response_kind}")),
        )
        .pop() else {
            return;
        };

        responses.push((response_kind, type_.name));
    }
    let response = if responses.is_empty() {
        "()".into()
    } else if responses.len() == 1 {
        responses[0].1.clone()
    } else {
        responses.sort_by_key(|r| r.0);
        if responses.len() == 2 && responses[0].0 == "200" && responses[1].0.starts_with("40") {
            format!("Result<{}, {}>", responses[0].1, responses[1].1)
        } else {
            dbg!(responses);
            todo!();
        }
    };
    let mut body = String::new();
    let query_parameters = get.parameters.iter().filter(|p| p.r#in.query()).peekable();
    writeln!(
        body,
        "    let mut r = client.{request_function_name}(format!(\"{}{path}\"));",
        fun.base_url,
    )
    .unwrap();
    writeln!(body, "    r = r.query(&[(\"api_key\", api_key)]);").unwrap();
    for p in query_parameters {
        let name = p.name.replace('.', "_").to_case(convert_case::Case::Snake);
        if !p.required {
            writeln!(body, "    if let Some({name}) = {name} {{",).unwrap();
        }
        let to_string_necessary = TypeDefinition::from_type_schema(&p.schema, String::new())
            .last()
            .is_some_and(|t| t.name != "String");
        writeln!(
            body,
            "        r = r.query(&[(\"{}\", {name}{})]);",
            p.name,
            if to_string_necessary {
                ".to_string()"
            } else {
                ""
            },
        )
        .unwrap();
        if !p.required {
            writeln!(body, "    }}").unwrap();
        }
    }
    let r#await = if fun.is_async { ".await" } else { "" };
    writeln!(
        body,
        "    let r = r.build().map_err(|e| crate::Error::without_context(e))?;"
    )
    .unwrap();
    writeln!(body, "    let url = r.url().clone();").unwrap();
    writeln!(
        body,
        "    let r = client.execute(r){await}.map_err(|e| crate::Error::new_with_url(&url, e))?;"
    )
    .unwrap();
    writeln!(body, "    let status = r.status();").unwrap();
    writeln!(
        body,
        "    let text = r.text(){await}.map_err(|e| crate::Error::new(&url, status, e))?;"
    )
    .unwrap();
    writeln!(body, "    let result = serde_json::from_str(&text).map_err(|e| crate::Error::new_with_text(&url, status, &text, e))?;").unwrap();
    writeln!(body, "    Ok(result)",).unwrap();
    let doc_comment = format!("/// {}\n///\n/// {}", get.summary, get.description);
    result.push(FunctionDefinition {
        name,
        parameters,
        response,
        body,
        doc_comment,
    });
}

fn collect_types_for(schema: &schema::Schema) -> Vec<TypeDefinition> {
    let mut result = Vec::new();
    for (path, schema) in &schema.paths {
        if let Some(get) = &schema.get {
            collect_types_in_path_route(get, &mut result, get.operation_id.clone());
        }
    }
    result
}

fn collect_types_in_path_route(
    get: &schema::PathRoute,
    result: &mut Vec<TypeDefinition>,
    namespace: String,
) {
    for parameter in &get.parameters {
        result.append(&mut TypeDefinition::from_type_schema(
            &parameter.schema,
            combine(&namespace, &parameter.name),
        ));
    }
    for (response_kind, response) in &get.responses {
        for (_, content) in &response.content {
            result.append(&mut TypeDefinition::from_type_schema(
                &content.schema,
                combine(&namespace, format!("Response{response_kind}")),
            ))
        }
    }
}

fn combine(namespace: &str, name: impl Into<String>) -> String {
    let name = name.into();
    if namespace.is_empty() {
        name
    } else {
        format!("{namespace}_{name}")
    }
}

fn handle_path_route(path: String, path_route: schema::PathRoute) {
    println!("{path}:");
    print!("fn {}(", path_route.operation_id.replace('-', "_"));
    for parameter in path_route.parameters {}
    print!(") -> ");
    for (status, response) in path_route.responses {
        for (type_, content) in response.content {
            // handle_schema(content.schema);
        }
    }
    println!();
}

fn handle_schema(schema: schema::TypeSchema) {
    match schema {
        schema::TypeSchema::Tagged(tagged_type_schema) => {
            handle_type_schema(tagged_type_schema);
        }
        schema::TypeSchema::Empty(hash_map) => print!("$$$Unknown"),
    }
}

fn handle_type_schema(type_schema: schema::TaggedTypeSchema) {
    match type_schema {
        schema::TaggedTypeSchema::Object { properties } => {
            println!();
            println!("struct $$$NoName?");
            for (field, field_t) in properties {
                println!("   {field}: ");
                handle_schema(field_t);
                println!(",");
            }
        }
        schema::TaggedTypeSchema::Boolean { default } => print!("bool"),
        schema::TaggedTypeSchema::String { default } => print!("string"),
        schema::TaggedTypeSchema::Integer { default } => print!("i64"),
        schema::TaggedTypeSchema::Number { default } => print!("f64"),
        schema::TaggedTypeSchema::Array { items } => {
            println!("Vec<");
            handle_schema(*items);
            println!(">");
        }
        schema::TaggedTypeSchema::Empty => unreachable!(),
    }
}
