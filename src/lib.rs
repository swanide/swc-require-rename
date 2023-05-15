#![allow(clippy::not_unsafe_ptr_arg_deref)]
use std::{collections::HashMap};
use swc_core::ecma::ast::{Lit, ModuleItem, ModuleDecl};
use swc_core::ecma::visit::VisitMutWith;
use swc_core::ecma::{
    ast::{Program, CallExpr, Callee, Expr, Module, Str},
    transforms::testing::test,
    visit::{as_folder, FoldWith, VisitMut},
};
use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};

pub struct Config {
    pub module_prefix: String,
}

pub struct TransformVisitor {
    config: Config,
}

impl VisitMut for TransformVisitor {


    // https://rustdoc.swc.rs/swc_ecma_visit/trait.VisitMut.html
    fn visit_mut_call_expr(&mut self, call_expr: &mut CallExpr) {
        if let Callee::Expr(expr) = &mut call_expr.callee {
            if let Expr::Ident(ident) = &mut **expr {
                if ident.sym.eq("require") {
                    if let Some(expr0) = call_expr.args.get_mut(0) {
                        if let Expr::Lit(Lit::Str(module_name)) = &mut *expr0.expr {
                            let raw_name = module_name.value.to_string();
                            if !raw_name.starts_with(&self.config.module_prefix) {
                                let mut new_name = String::from(&self.config.module_prefix);
                                new_name.push_str(&raw_name);
                                *module_name = Str::from(new_name.as_ref());
                            }
                        }
                    }
                    return;
                }
            }
        }
        call_expr.callee.visit_mut_children_with(self);
        call_expr.args.visit_mut_children_with(self);
    }


   fn visit_mut_module(&mut self, module: &mut Module) {
        for item in &mut module.body {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::Import(decl)) => {
                    let raw_name = decl.src.value.to_string();
                    let mut new_name = String::from(&self.config.module_prefix);
                    new_name.push_str(&raw_name);
                    decl.src = Box::new(Str::from(new_name.as_ref()))
                },
                ModuleItem::ModuleDecl(ModuleDecl::ExportNamed(decl)) => {
                    if let Some(src) = &decl.src {
                        let raw_name = src.value.to_string();
                        let mut new_name = String::from(&self.config.module_prefix);
                        new_name.push_str(&raw_name);
                        decl.src = Some(Box::new(Str::from(new_name.as_ref())));
                    }
                },
                ModuleItem::ModuleDecl(ModuleDecl::ExportAll(decl)) => {
                    let raw_name = decl.src.value.to_string();
                    let mut new_name = String::from(&self.config.module_prefix);
                    new_name.push_str(&raw_name);
                    decl.src = Box::new(Str::from(new_name.as_ref()));
                },
                _ => {
                    item.visit_mut_children_with(self);
                }
            }
        }
    }
}

#[plugin_transform]
pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {
    let json: HashMap<String, String> = serde_json::from_str(&_metadata
            .get_transform_plugin_config()
            .expect("failed to get plugin config for require-rename"),
    ).expect("invalid config");

    let config = Config{
        module_prefix: match json.get("modulePrefix") {
            Some(name) => name.to_string(),
            None => "".to_string(),
        }
    };

    if config.module_prefix.ne("") {
        program.fold_with(&mut as_folder(TransformVisitor{config}))
    }
    else {
        program
    }
}


test!(
    Default::default(),
    |_| {
        let config = Config{module_prefix: "@swan-module/".to_string()};
        as_folder(TransformVisitor{
            config
        })
    },
    test_trans_import,
    // Input codes
    r#"
import { resolve } from 'path';
import fs from 'fs';
const a = 1 || require('a');
require('bcd');
a(require("_name").start());
// not transform
t.require('no');
require('@swan-module/m');
var t = require("./typeof.js").default;
"#,
    // Output codes after transformed with plugin
    r#"import { resolve } from "@swan-module/path";
import fs from "@swan-module/fs";
const a = 1 || require("@swan-module/a");
require("@swan-module/bcd");
a(require("@swan-module/_name").start());
// not transform
t.require('no');
require('@swan-module/m');
var t = require("@swan-module/./typeof.js").default;
"#
);

test!(
    Default::default(),
    |_| {
        let config = Config{module_prefix: "@swan-module/".to_string()};
        as_folder(TransformVisitor{
            config
        })
    },
    test_trans_require,
    // Input codes
    r#"(swan.webpackJsonp=swan.webpackJsonp||[]).push({141:function(x,e){

const a = 1 || require('a');
require('bcd');
a(require("_name").start());
// not transform
t.require('no');
require('@swan-module/m');

var t = require("./typeof.js").default;
return t;
}});
"#,
    // Output codes after transformed with plugin
    r#"(swan.webpackJsonp=swan.webpackJsonp||[]).push({141:function(x,e){

const a = 1 || require("@swan-module/a");
require("@swan-module/bcd");
a(require("@swan-module/_name").start());
// not transform
t.require('no');
require('@swan-module/m');

var t = require("@swan-module/./typeof.js").default;
return t;
}});
"#
);

test!(
    Default::default(),
    |_| {
        let config = Config{module_prefix: "@swan-module/".to_string()};
        as_folder(TransformVisitor{
            config
        })
    },
    test_trans_export,
    // Input codes
    r#"
export * from 'eee';
export * as abc from 'fff';
export { ggg } from 'ggg';
"#,
    // Output codes after transformed with plugin
    r#"export * from "@swan-module/eee";
export * as abc from "@swan-module/fff";
export { ggg } from "@swan-module/ggg";
"#
);

