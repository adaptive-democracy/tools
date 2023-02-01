use sycamore::prelude::*;
use sycamore::web::{web_sys, wasm_bindgen::JsValue};
// use sycamore::builder::ElementBuilderOrView;
use persistent_democracy_core::{Constitution, Tree, Keyable, ParentKeyable};

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
	// parent_id: Option<RcSignal<usize>>,
	parent_id: Option<usize>,
}
impl Keyable<usize> for ConstitutionRx {
	fn key(&self) -> usize { self.id }
}

impl ParentKeyable<usize> for ConstitutionRx {
	fn parent_key(&self) -> Option<usize> { self.parent_id }
}

impl From<ConstitutionRx> for Constitution {
	fn from(c: ConstitutionRx) -> Self {
		Constitution{id: c.id, name: c.name.get().to_string(), parent_id: c.parent_id}
	}
}

impl From<Constitution> for ConstitutionRx {
	fn from(c: Constitution) -> Self {
		ConstitutionRx{id: c.id, name: create_rc_signal(c.name), parent_id: c.parent_id}
	}
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
	let constitutions = create_signal(cx, vec![
		ConstitutionRx{ id: 1, name: create_rc_signal("initial root".into()), parent_id: None },
	]);
	// let id_counter = Rc::new(1);
	let mut id_counter = 1;

	let sub_tree_result = constitutions.map(cx, |c| Tree::from_vec(c.clone()));
	// let sub_tree_result = tree_result.map(cx, |r| r.as_ref().map(move |t| Tree::root_sub_tree(t)));
	// let sub_tree_result = create_memo(cx, || {
	// 	(tree_result.get()).map(|t| Tree::root_sub_tree(&t))
	// });

	let push_constitution = |name: String, parent_id: usize| {
		let id = id_counter;
		id_counter += 1;
		constitutions.modify().push(ConstitutionRx{id, name: create_rc_signal(name), parent_id: Some(parent_id)});
	};
	let remove_constitution = |id: usize| {
		constitutions.modify().retain(|c| c.id != id);
	};
	// modification of parent is easy

	let next_name = create_signal(cx, String::new());
	let handle_enter = |event: web_sys::KeyboardEvent| {
		if event.code() != "Enter" {
			return
		}
		let constitution = ConstitutionRx{ id: 1, name: create_rc_signal(next_name.get().as_ref().into()), parent_id: None };
		constitutions.modify().push(constitution);
		next_name.modify().clear();
	};

	view!{cx,
		(match sub_tree_result.borrow() {
			Err(_) => view!{cx, "problem while building tree" },
			Ok(sub_tree) => {
				// let  = tree.root_sub_tree();
				view!{cx,
					p { (sub_tree.item.name.get()) }
					// Indexed(
					// 	iterable=sub_tree.children(),
					// 	view=|cx, c| view!{cx,

					// 	}
					// )
				}
			},
		})

		// ConstitutionSubTreeView(sub_tree=sub_tree)

		// p { "Mutable version" }
		// Keyed(
		// 	iterable=constitutions,
		// 	key=|c| c.id,
		// 	view=|cx, c| view!{cx,
		// 		p {
		// 			input(bind:value=c.name)
		// 		}
		// 	}
		// )
		// p { input(bind:value=next_name, on:keyup=handle_enter) }

		// p { "Immutable version" }
		// Keyed(
		// 	iterable=constitutions,
		// 	key=|c| c.id,
		// 	view=|cx, c| view!{cx,
		// 		p { (c.name.get()) }
		// 	}
		// )
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
