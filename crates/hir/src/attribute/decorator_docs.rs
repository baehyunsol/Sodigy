use crate::{
    Alias,
    ArgCount,
    ArgType,
    Assert,
    Enum,
    EnumVariant,
    Func,
    KeywordArgRule,
    Let,
    Module,
    Requirement,
    Struct,
    Use,
};
use sodigy_error::ItemKind;
use sodigy_string::InternedString;
use std::collections::hash_map::{Entry, HashMap};

struct DecoratorInfo {
    name: InternedString,

    // `Vec<(bool, bool)>` is `Vec<(is_top_level, is_std)>`. I want to check whether the decorator is
    // top-level-only and/or std-only.
    items: HashMap<ItemKind, Vec<(bool, bool)>>,

    // It assumes that the same decorator has the same arg_requirement/
    arg_requirement: Requirement,
    arg_count: ArgCount,
    arg_type: ArgType,
    keyword_args: HashMap<InternedString, KeywordArgRule>,
}

// TODO: maybe there's a better return type than `String`...
// TODO: this function requiring `intermediate_dir` is ridiculuous
//       this function calling `unintern_or_default` is also ridiculuous
//
// I want a complete document for decorators, and I want it to be semi-auto-generated.
// I think this is the best place to write/implement the docs.
pub fn generate_decorator_docs(intermediate_dir: &str) -> String {
    let mut doc = vec![];

    // It uses the name of the decorator as a key. The assumption is that
    // the same decorator is used in the same way regardless of `ItemKind`.
    // For example, if `#[impl]` is used in different way for an enum and
    // for a function, that's a wrong design.
    let mut decorators: HashMap<InternedString, DecoratorInfo> = HashMap::new();

    for item in [
        ItemKind::Alias,
        ItemKind::Assert,
        ItemKind::Enum,
        ItemKind::EnumVariant,
        ItemKind::Func,
        ItemKind::Let,
        ItemKind::Module,
        ItemKind::Struct,
        ItemKind::Use,
    ] {
        for (is_top_level, is_std) in [
            (true, true),
            (true, false),
            (false, true),
            (false, false),
        ] {
            let rule = match item {
                ItemKind::Alias => Alias::get_attribute_rule(is_top_level, is_std, intermediate_dir),
                ItemKind::Assert => Assert::get_attribute_rule(is_top_level, is_std, intermediate_dir),
                ItemKind::Enum => Enum::get_attribute_rule(is_top_level, is_std, intermediate_dir),
                ItemKind::EnumVariant => EnumVariant::get_attribute_rule(is_top_level, is_std, intermediate_dir),
                ItemKind::Func => Func::get_attribute_rule(is_top_level, is_std, intermediate_dir),
                ItemKind::Let => Let::get_attribute_rule(is_top_level, is_std, intermediate_dir),
                ItemKind::Module => Module::get_attribute_rule(is_top_level, is_std, intermediate_dir),
                ItemKind::Struct => Struct::get_attribute_rule(is_top_level, is_std, intermediate_dir),
                ItemKind::Use => Use::get_attribute_rule(is_top_level, is_std, intermediate_dir),
            };

            for decorator in rule.decorators.values() {
                if decorator.requirement == Requirement::Never {
                    continue;
                }

                match decorators.entry(decorator.name) {
                    Entry::Occupied(mut e) => match e.get_mut().items.entry(item) {
                        Entry::Occupied(mut e) => {
                            e.get_mut().push((is_top_level, is_std));
                        },
                        Entry::Vacant(e) => {
                            e.insert(vec![(is_top_level, is_std)]);
                        },
                    },
                    Entry::Vacant(e) => {
                        e.insert(DecoratorInfo {
                            name: decorator.name,
                            items: [(item, vec![(is_top_level, is_std)])].into_iter().collect(),
                            arg_requirement: decorator.arg_requirement,
                            arg_count: decorator.arg_count,
                            arg_type: decorator.arg_type,
                            keyword_args: decorator.keyword_args.clone(),
                        });
                    },
                }
            }
        }
    }

    let mut decorators = decorators.into_iter().map(
        |(name, decorator)| (
            decorator.name.unintern_or_default(intermediate_dir),
            decorator,
        )
    ).collect::<Vec<(String, DecoratorInfo)>>();
    decorators.sort_by_key(|(name, _)| name.to_string());

    for (name, decorator) in decorators.iter() {
        doc.push(format!("# {name}"));
        doc.push(String::new());

        for (item, flags) in decorator.items.iter() {
            let top_level_only = (flags.contains(&(true, true)) || flags.contains(&(true, false))) && !flags.contains(&(false, false));
            let std_only = (flags.contains(&(true, true)) || flags.contains(&(false, true))) && !flags.contains(&(false, false));
            let flag = match (top_level_only, std_only) {
                (true, true) => " (top-level, std)",
                (true, false) => " (top-level)",
                (false, true) => " (std)",
                (false, false) => "",
            };

            doc.push(format!("- {}{flag}", item.render()));
        }

        doc.push(String::new());
        // TODO: write the actual doc
    }

    doc.join("\n")
}
