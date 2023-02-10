use uuid::Uuid;
use sycamore::prelude::*;
use sycamore::web::{web_sys, wasm_bindgen::JsValue};
// use sycamore::builder::ElementBuilderOrView;
// use persistent_democracy_core::{Constitution, Tree, Keyable, ParentKeyable};

fn log_str(s: &'static str) {
	web_sys::console::log_1(&JsValue::from_str(s));
}
fn log<T: Into<JsValue>>(value: T) {
	web_sys::console::log_1(&value.into());
}

#[derive(Debug, Clone, PartialEq)]
struct Constitution {
	id: Uuid,
	title: RcSignal<String>,
	text: RcSignal<String>,
	sub_constitutions: RcSignal<Vec<Constitution>>,
}

impl Constitution {
	fn new_using(title: String, text: String) -> Constitution {
		let id = Uuid::new_v4();
		let title = create_rc_signal(title);
		let text = create_rc_signal(text);
		let sub_constitutions = create_rc_signal(Vec::new());
		Constitution{id, title, text, sub_constitutions}
	}

	fn to_db(&self) -> Vec<ConstitutionDb> {
		let mut db_constitutions = vec![];
		self.to_db_recursive(&mut db_constitutions, None);
		db_constitutions
	}

	fn to_db_recursive(&self, db_constitutions: &mut Vec<ConstitutionDb>, parent_id: Option<Uuid>) {
		db_constitutions.push(ConstitutionDb{
			id: self.id.clone(),
			title: self.title.to_string(),
			text: self.text.to_string(),
			parent_id,
		});

		let current_id = Some(self.id);
		for sub_constitution in &*self.sub_constitutions.get_untracked() {
			sub_constitution.to_db_recursive(db_constitutions, current_id);
		}
	}
}

#[derive(Debug)]
struct ConstitutionDb {
	id: Uuid,
	title: String,
	text: String,
	parent_id: Option<Uuid>,
}

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
	let root_constitution = create_ref(cx, Constitution::new_using("root".into(), String::new()));
	let save_constitutions = |_| {
		let db_constitutions = root_constitution.to_db();
		for db_constitution in db_constitutions {
			log(format!("{:?}", db_constitution.id));
			log(format!("{:?}", db_constitution.title));
			log(format!("{:?}", db_constitution.text));
			log(format!("{:?}", db_constitution.parent_id));
			log("");
		}
	};

	view!{cx,
		ConstitutionView(constitution=root_constitution)

		div { button(on:click=save_constitutions) { "save constitutions" } }
	}
}

#[component(inline_props)]
fn ConstitutionView<'s, G: Html>(
	cx: Scope<'s>,
	constitution: &'s Constitution,
) -> View<G> {
	let new_title = create_signal(cx, String::new());
	let new_text = create_signal(cx, String::new());

	let push_constitution = |_| {
		let mut new_title = new_title.modify();
		let mut new_text = new_text.modify();
		constitution.sub_constitutions.modify().push(Constitution::new_using(
			new_title.to_string(),
			new_text.to_string(),
		));
		new_title.clear();
		new_text.clear();
	};

	view!{cx,
		div(style="border: solid; padding: 2px;") {
			p { input(bind:value=constitution.title, placeholder="constitution title") }
			p { textarea(bind:value=constitution.text, placeholder="constitution text") }

			(if constitution.sub_constitutions.get().len() == 0 {
				view!{cx, p { "no children" } }
			} else { view!{cx,
				Keyed(
					iterable=&constitution.sub_constitutions,
					key=|c| c.id,
					view=|cx, sub_constitution| view!{cx, ConstitutionView(constitution=create_ref(cx, sub_constitution)) },
				)
			} })

			div { input(bind:value=new_title, placeholder="child constitution title") }
			div { textarea(bind:value=new_text, placeholder="child constitution text") }
			div { button(on:click=push_constitution) { "add child constitution" } }
		}
	}
}


fn main() {
	sycamore::render(|cx| view!{cx, App{} });
}
