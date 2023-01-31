use sycamore::prelude::*;
use sycamore::web::{web_sys, wasm_bindgen::JsValue};
// use sycamore::builder::ElementBuilderOrView;
use persistent_democracy_core::{Constitution};

fn log_str(s: &'static str) {
	web_sys::console::log_1(&JsValue::from_str(s));
}
fn log<T: Into<JsValue>>(value: T) {
	web_sys::console::log_1(&value.into());
}

#[derive(Debug, Clone, PartialEq)]
struct ConstitutionRx {
	id: usize,
	name: RcSignal<String>,
	// parent_id: Option<usize>,
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
	let constitutions = create_signal(cx, vec![]);

	// let tree = create_signal(cx, Err("uninit"));
	// create_effect(cx, || {
	// 	let i = ConstitutionTree::from_vec(*constitutions.get());
	// 	tree.set(i.arena);
	// });

	let next_name = create_signal(cx, String::new());
	let handle_enter = |event: web_sys::KeyboardEvent| {
		if event.code() != "Enter" {
			return
		}
		let constitution = ConstitutionRx{ id: 1, name: create_rc_signal(next_name.get().as_ref().into()) };
		constitutions.modify().push(constitution);
		next_name.modify().clear();
	};

	view!{cx,
		// ConstitutionTreeView(tree=tree)

		p { "Mutable version" }
		Keyed(
			iterable=constitutions,
			key=|c| c.id,
			view=|cx, c| view!{cx,
				p {
					input(bind:value=c.name)
				}
			}
		)
		p { input(bind:value=next_name, on:keyup=handle_enter) }

		p { "Immutable version" }
		Keyed(
			iterable=constitutions,
			key=|c| c.id,
			view=|cx, c| view!{cx,
				p { (c.name.get()) }
			}
		)
	}
}

// #[component(inline_props)]
// fn ConstitutionRxView<'a, 'b: 'a, G: Html>(
// 	cx: Scope<'a>,
// 	&'a constitution: ConstitutionRx<'b>,
// ) -> View<G> {
// 	let name = &constitution.name;
// 	view!{cx,
// 		input(bind:value=name, on:keyup=|e| &handle_enter(cx, e))
// 	}
// }

// #[component(inline_props)]
// fn ConstitutionTreeView<'a, G: Html>(
// 	cx: Scope<'a>,
// 	constitutions: &'a ReadSignal<Vec<Constitution>>,

// ) -> View<G> {

// 	view!{cx,
// 		// Keyed(
// 		Indexed(
// 			iterable=current_node_id.children(arena.get()),
// 			// key=|c| c.id,
// 			view=|cx, c| view!{cx,

// 			}
// 		)

// 		// p {
// 		// 	input(bind:value=next_name, on:keyup=handle_enter)
// 		// }
// 	}
// }


fn main() {
	sycamore::render(|cx| view!{cx, App{} });
}
