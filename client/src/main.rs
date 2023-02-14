use uuid::Uuid;
use sycamore::prelude::*;
use sycamore_router::{Route, Router, HistoryIntegration, /*navigate*/};
// use persistent_democracy_core::{Constitution, Tree, Keyable, ParentKeyable};

type Weight = f64;

// TODO use env PROFILE=debug|release instead of debug_assertions?
// https://doc.rust-lang.org/cargo/reference/environment-variables.html
// #[cfg(debug_assertions)]
// const API_BASE_URL: &str = "https://localhost::5050";

#[cfg(not(debug_assertions))]
const API_BASE_URL: &str = env!("API_BASE_URL", "need to specify API_BASE_URL when compiling for release");

#[derive(Clone, PartialEq)]
struct Constitution {
	id: Uuid,
	title: RcSignal<String>,
	text: RcSignal<String>,
	sub_constitutions: RcSignal<Vec<Constitution>>,
	// sub_elections: RcSignal<Vec<Election>>,
}

// #[derive(Clone, PartialEq)]
// struct Election {
// 	id: Uuid,
// 	title: RcSignal<String>,
// 	candidates: RcSignal<Vec<String>>,
// }

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

#[component]
fn App<G: Html>(cx: Scope) -> View<G> {
	let root_constitution = create_ref(cx, Constitution::new_using("root".into(), String::new()));
	let save_constitutions = |_| {
		let db_constitutions = root_constitution.to_db();
		for db_constitution in db_constitutions {
			utils::log(format!("{:?}", db_constitution.id));
			utils::log(format!("{:?}", db_constitution.title));
			utils::log(format!("{:?}", db_constitution.text));
			utils::log(format!("{:?}", db_constitution.parent_id));
			utils::log("");
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
		div(class="border p-2") {
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



#[derive(Route)]
enum AppRoutes {
	#[to("/")]
	Index,

	// the current user's info, containing their allocations and candidacies and constitution drafts
	// this page probably needs some "state" concept, so other pages can navigate here with the intent to perform some particular action
	#[to("/me")]
	Me,

	// the constitution tree explorer
	#[to("/constitution")]
	ConstitutionTree,

	// the constitution tree editor, where users can edit constitutions and
	#[to("/constitution/draft/<id>")]
	ConstitutionDraft(Uuid),

	// #[to("/constitution/draft/<next>/compare/<prev>")]
	// ConstitutionCompare(Uuid, Uuid),

	// some particular constitution, with its current election
	#[to("/constitution/<id>")]
	Constitution(Uuid),

	// some particular election
	#[to("/election/<id>")]
	Election(Uuid),

	#[not_found]
	NotFound,
}

mod election;

fn main() {
	sycamore::render(|cx| view!{cx,
		Router(
			integration=HistoryIntegration::new(),
			view=|cx, route: &ReadSignal<AppRoutes>| {
				view!{cx,
					div(class="app") {
						(match route.get().as_ref() {
							AppRoutes::Index => view!{cx,
								App{}
							},
							AppRoutes::Me => view!{cx, Me{} },
							AppRoutes::ConstitutionTree => view!{cx, ConstitutionTree{} },
							AppRoutes::ConstitutionDraft(_) => view!{cx, ConstitutionDraft{} },
							// AppRoutes::ConstitutionCompare => view!{cx, ConstitutionCompare{} },
							AppRoutes::Constitution(_) => view!{cx, Constitution{} },
							AppRoutes::Election(id) => view!{cx, election::ElectionView(id=*id) },
							AppRoutes::NotFound => view!{cx, NotFound{} },
						})
					}
				}
			}
		)
	});
}



#[component]
fn Me<G: Html>(cx: Scope) -> View<G> {
	view!{cx, "Me"}
}

#[component]
fn ConstitutionTree<G: Html>(cx: Scope) -> View<G> {
	view!{cx, "ConstitutionTree"}
}

#[component]
fn ConstitutionDraft<G: Html>(cx: Scope) -> View<G> {
	view!{cx, "ConstitutionDraft"}
}

#[component]
fn ConstitutionCompare<G: Html>(cx: Scope) -> View<G> {
	view!{cx, "ConstitutionCompare"}
}

#[component]
fn Constitution<G: Html>(cx: Scope) -> View<G> {
	view!{cx, "Constitution"}
}

#[component]
fn NotFound<G: Html>(cx: Scope) -> View<G> {
	view!{cx, "NotFound" }
}

#[derive(Debug)]
struct ConstitutionDb {
	id: Uuid,
	title: String,
	text: String,
	parent_id: Option<Uuid>,
}

// #[derive(Debug)]
// struct ElectionDb {
// 	id: Uuid,
// 	title: String,
// 	constitution_id: Uuid,
// }

// #[derive(Debug)]
// struct CandidacyDb {
// 	election_id: Uuid,
// 	candidate_id: Uuid,
// 	stabilization_bucket: Weight,
// }

// #[derive(Debug)]
// enum AllocationType {
// 	For,
// 	Against,
// }

// #[derive(Debug)]
// struct AllocationDb {
// 	voter_id: Uuid,
// 	election_id: Uuid,
// 	candidate_id: Uuid,
// 	weight: Weight,
// 	type: AllocationType,
// }

pub mod utils {
	use std::future::Future;
	use sycamore::prelude::*;

	pub fn create_async_signal<'s, U: 'static, F>(cx: Scope<'s>, f: F) -> &'s Signal<Option<U>>
	where
			F: Future<Output = U> + 's,
	{
		let signal = create_signal(cx, None);

		sycamore::futures::spawn_local_scoped(cx, {
			let signal = signal.clone();
			async move {
				signal.set(Some(f.await));
			}
		});

		signal
	}

	use sycamore::web::{web_sys, wasm_bindgen::JsValue};
	pub fn log_str(s: &'static str) {
		web_sys::console::log_1(&JsValue::from_str(s));
	}
	pub fn log<T: Into<JsValue>>(value: T) {
		web_sys::console::log_1(&value.into());
	}
}
