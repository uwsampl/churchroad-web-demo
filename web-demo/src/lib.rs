#![allow(clippy::unused_unit)]
use egglog::SerializeConfig;
use egraph_serialize::{ClassId, NodeId};
use indexmap::IndexMap;
// weird clippy bug with wasm-bindgen
use log::{Level, Log, Metadata, Record};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(getter_with_clone)]
pub struct Result {
    pub text: String,
    pub dot: String,
}

#[wasm_bindgen]
pub fn run_program(input: &str) -> Result {
    let mut egraph = egglog::EGraph::default();
    match egraph.parse_and_run_program(input) {
        Ok(outputs) => {
            let serialized = egraph.serialize_for_graphviz(false);
            let serialized2 = egraph.serialize(SerializeConfig::default());
            let choices = GreedyDagExtractor::default().extract(&serialized2, &[]);
            let mut out = churchroad::to_verilog_egraph_serialize(&serialized2, &choices, "");

            out.push_str("\n\nOutputs:\n");
            for (i, output) in outputs.iter().enumerate() {
                out.push_str(&format!("Output {}: {}\n", i, output));
            }
            
            Result {
                text: out,
                dot: serialized.to_dot(),
            }
        }
        Err(e) => Result {
            text: e.to_string(),
            dot: "".to_string(),
        },
    }
}

#[wasm_bindgen(start)]
pub fn start() {
    init();
    console_error_panic_hook::set_once();
}

/// The log styles
struct Style {
    lvl_trace: String,
    lvl_debug: String,
    lvl_info: String,
    lvl_warn: String,
    lvl_error: String,
}

impl Style {
    fn new() -> Self {
        let base = String::from("color: white; padding: 0 3px; background:");
        Style {
            lvl_trace: format!("{} gray;", base),
            lvl_debug: format!("{} blue;", base),
            lvl_info: format!("{} green;", base),
            lvl_warn: format!("{} orange;", base),
            lvl_error: format!("{} darkred;", base),
        }
    }

    fn get_lvl_style(&self, lvl: Level) -> &str {
        match lvl {
            Level::Trace => &self.lvl_trace,
            Level::Debug => &self.lvl_debug,
            Level::Info => &self.lvl_info,
            Level::Warn => &self.lvl_warn,
            Level::Error => &self.lvl_error,
        }
    }
}

// This is inspired by wasm_logger
struct WebDemoLogger {
    style: Style,
}

impl Log for WebDemoLogger {
    fn enabled(&self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    fn log(&self, record: &Record<'_>) {
        if self.enabled(record.metadata()) {
            let style = &self.style;
            let s = format!(
                "<span style=\"{}\">{}</span>\n{}\n",
                style.get_lvl_style(record.level()),
                record.level(),
                record.args(),
            );
            log(record.level().as_str(), &s);
        }
    }

    fn flush(&self) {}
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    fn log(level: &str, s: &str);
}

pub fn init() {
    let max_level = Level::Debug;
    let wl = WebDemoLogger {
        style: Style::new(),
    };

    match log::set_boxed_logger(Box::new(wl)) {
        Ok(_) => log::set_max_level(max_level.to_level_filter()),
        Err(e) => console::error_1(&JsValue::from(e.to_string())),
    }
}


use rustc_hash::FxHashMap;

pub type Cost = ordered_float::NotNan<f64>;

struct CostSet {
    costs: FxHashMap<ClassId, Cost>,
    total: Cost,
    choice: NodeId,
}

#[derive(Default)]
pub struct GreedyDagExtractor;
impl GreedyDagExtractor {
    fn extract(&self, egraph: &egraph_serialize::EGraph, _roots: &[egraph_serialize::ClassId]) -> IndexMap<ClassId, NodeId>
    {
        let mut costs = FxHashMap::<ClassId, CostSet>::with_capacity_and_hasher(
            egraph.classes().len(),
            Default::default(),
        );

        let mut keep_going = true;

        let mut i = 0;
        while keep_going {
            i += 1;
            println!("iteration {}", i);
            keep_going = false;

            'node_loop: for (node_id, node) in &egraph.nodes {
                let cid = egraph.nid_to_cid(node_id);
                let mut cost_set = CostSet {
                    costs: Default::default(),
                    total: Cost::default(),
                    choice: node_id.clone(),
                };

                // compute the cost set from the children
                for child in &node.children {
                    let child_cid = egraph.nid_to_cid(child);
                    if let Some(child_cost_set) = costs.get(child_cid) {
                        // prevent a cycle
                        if child_cost_set.costs.contains_key(cid) {
                            continue 'node_loop;
                        }
                        cost_set.costs.extend(child_cost_set.costs.clone());
                    } else {
                        continue 'node_loop;
                    }
                }

                // add this node
                cost_set.costs.insert(cid.clone(), node.cost);

                dbg!(node.cost);

                cost_set.total = cost_set.costs.values().sum();

                // if the cost set is better than the current one, update it
                if let Some(old_cost_set) = costs.get(cid) {
                    if cost_set.total < old_cost_set.total {
                        costs.insert(cid.clone(), cost_set);
                        keep_going = true;
                    }
                } else {
                    costs.insert(cid.clone(), cost_set);
                    keep_going = true;
                }
            }
        }

        let mut result = IndexMap::default();
        for (cid, cost_set) in costs {
            result.insert(cid, cost_set.choice);
        }
        result
    }
}
