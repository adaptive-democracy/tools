use std::collections::{HashMap, HashSet};

trait Keyable<K> {
	fn key(&self) -> K;
}

fn index<T: Keyable<K>, K: Eq + std::hash::Hash>(items: Vec<T>) -> HashMap<K, T> {
	items.into_iter()
		.map(|item| (item.key(), item))
		.collect()
}


fn insert_or_conflict<T: Clone, K: Eq + std::hash::Hash>(indexed: &mut HashMap<K, T>, key: K, item: T) -> Result<(), (K, T, T)> {
	match indexed.get(&key) {
		Some(existing_item) => Err((key, item, existing_item.clone())),
		None => {
			indexed.insert(key, item);
			Ok(())
		},
	}
}

fn index_with_conflicts<T: Keyable<K> + Clone, K: Copy + Eq + std::hash::Hash>(items: Vec<T>) -> (HashMap<K, T>, Vec<(K, T, T)>) {
	let mut indexed: HashMap<K, T> = HashMap::new();
	let mut conflicts = vec![];
	for item in items.into_iter() {
		if let Err(conflict) = insert_or_conflict(&mut indexed, item.key(), item) {
			conflicts.push(conflict);
		}
	}

	(indexed, conflicts)
}


#[derive(Debug)]
struct HistoryVec<T> {
	history: Vec<T>,
	current: T,
}

impl <T> HistoryVec<T> {
	fn shift(&mut self, new_current: T) {
		let current = std::mem::replace(&mut self.current, new_current);
		self.history.push(current);
	}

	fn step_shift<F: Fn(&T) -> T>(&mut self, step_fn: F) {
		let new_current = step_fn(&self.current);
		self.shift(new_current);
	}
}


fn step_histories<T, K: std::hash::Hash, F: Fn(&T) -> T>(
	histories: &mut HashMap<K, HistoryVec<T>>,
	step_fn: F,
) {
	for history in histories.values_mut() {
		history.step_shift(&step_fn);
	}
}


// TODO find fixed precision numeric library
type Weight = f32;
// TODO make id wrapper for different id?

#[derive(Debug)]
struct Person {
	id: usize,
	name: String,
}

#[derive(Debug)]
struct Election {
	id: usize,
	title: String,
}

#[derive(Debug)]
struct Candidacy {
	election_id: usize,
	candidate_id: usize,
	stabilization_bucket: Option<Weight>,
}

impl Keyable<(usize, usize)> for Candidacy {
	fn key(&self) -> (usize, usize) {
		(self.election_id, self.candidate_id)
	}
}


#[derive(Debug)]
enum AllocationType {
	For,
	Against,
}

#[derive(Debug)]
struct Allocation {
	voter_id: usize,
	election_id: usize,
	candidate_id: usize,

	weight: Weight,
	allocation_type: AllocationType,
}

impl Keyable<(usize, usize)> for Allocation {
	fn key(&self) -> (usize, usize) {
		(self.election_id, self.candidate_id)
	}
}

impl Allocation {
	fn actual_vote(&self) -> Weight {
		let direction = match self.allocation_type {
			AllocationType::For => 1.0,
			AllocationType::Against => -1.0,
		};

		direction * self.weight.sqrt()
	}
}



fn compute_total_votes(candidacy_vec: Vec<Candidacy>, allocation_vec: Vec<Allocation>) -> HashMap<(usize, usize), Weight> {
	// let invalid_allocation_vec = vec![];
	// let duplicate_candidacy_vec = vec![];

	let mut candidacy_vote_map = HashMap::new();
	for candidacy in candidacy_vec {
		if let Some(_) = candidacy_vote_map.insert(candidacy.key(), 0.0) {
			// duplicate_candidacy_vec.push(candidacy);
			eprintln!("duplicate candidacy: {:?}", candidacy);
		}
	}

	for allocation in allocation_vec {
		match candidacy_vote_map.get_mut(&allocation.key()) {
			Some(candidacy_vote) => {
				*candidacy_vote += allocation.actual_vote();
			},
			None => {
			// invalid_allocation_vec.push(candidacy);
				eprintln!("invalid allocation: {:?}", allocation);
			},
		}
	}

	candidacy_vote_map
}

// fn compute_next_candidacy_values(arg: Type) -> RetType {
// 	unimplemented!()
// }


#[derive(Debug, Clone, PartialEq)]
struct Constitution {
	id: usize,
	name: String,
	// text: String, someday some governance code ast
	parent_id: Option<usize>,
}

impl Keyable<usize> for Constitution {
	fn key(&self) -> usize {
		self.id
	}
}

#[derive(Debug)]
struct ConstitutionView {
	id: usize,
	name: String,
	children: Vec<ConstitutionView>,
}

fn constitution_view_to_db(constitutions: Vec<ConstitutionView>) -> Vec<Constitution> {
	unimplemented!()
}


use indextree::{Arena, NodeId};

#[derive(Debug)]
enum CreateTreeError {
	NoRoot,
	MultipleRoots(NodeId, NodeId),
	NonExistentParent(usize, usize),
}

fn create_constitution_tree(
	constitutions: Vec<Constitution>,
) -> Result<(Arena<Constitution>, NodeId, HashMap<usize, NodeId>), CreateTreeError> {
	let mut arena = Arena::new();

	let mut constitution_keys = vec![];
	let mut constitution_node_ids = HashMap::new();
	for constitution in constitutions.into_iter() {
		constitution_keys.push((constitution.id, constitution.parent_id));
		constitution_node_ids.insert(constitution.id, arena.new_node(constitution));
	}

	let mut root_node_id = None;
	for (id, parent_id) in constitution_keys {
		let node_id = constitution_node_ids.get(&id).unwrap();
		match (parent_id, root_node_id) {
			(Some(parent_id), _) => {
				match constitution_node_ids.get(&parent_id) {
					Some(parent_node_id) => {
						parent_node_id.append(*node_id, &mut arena);
					},
					None => {
						return Err(CreateTreeError::NonExistentParent(id, parent_id));
					},
				}
			},
			(None, None) => {
				root_node_id = Some(node_id)
			},
			(None, Some(root_node_id)) => {
				return Err(CreateTreeError::MultipleRoots(*root_node_id, *node_id));
			}
		}
	}

	match root_node_id {
		Some(root_node_id) => Ok((arena, *root_node_id, constitution_node_ids)),
		None => Err(CreateTreeError::NoRoot),
	}
}


#[derive(Debug)]
enum ConstitutionMutation {
	Keep,
	Delete,
	Change(Constitution),
}

fn check_constitution_change() {
	unimplemented!()
}


#[derive(Debug)]
enum ChangeError {
	MissingMutation(usize),
	NextTreeInvalid(CreateTreeError)
}

fn apply_constitution_changes(
	constitutions: Vec<Constitution>,
	mutations: HashMap<usize, ConstitutionMutation>,
	additions: Vec<Constitution>,
) -> Result<Vec<Constitution>, ChangeError> {
	// let live_constitution_ids: HashSet<usize> =
	// 	constitutions.iter().map(|c| c.id)
	// 	.chain(additions.iter().map(|a| a.id))
	// 	.collect();

	// let dead_constitution_ids: HashSet<usize> = mutations.iter().filter_map(|(id, m)| match m {
	// 	ConstitutionMutation::Delete => Some(*id),
	// 	_ => None,
	// }).collect();
	// let live_constitution_ids = live_constitution_ids.difference(&dead_constitution_ids);

	let mut next_constitutions = vec![];
	for constitution in constitutions.into_iter() {
		let mutation = mutations.get(&constitution.id).ok_or_else(|| ChangeError::MissingMutation(constitution.id))?;
		match mutation {
			Keep => { next_constitutions.append(constitution); },
			Delete => { dead_constitution_ids.append(constitution.id); },
			Change(next_constitution) => {

			},
		}

		next_constitutions.append();
	}
	next_constitutions.extend(additions);

	// let all_constitutions = [constitutions, additions].concat();
	// let (arena, root_node_id, constitution_node_ids) = create_constitution_tree(all_constitutions.clone())
	// 	.map_err(ChangeError::NextTreeInvalid)?;

	// _apply_constitution_changes(current_node_id, arena, mutations, &mut live_constitution_ids, &mut next_constitutions);

	// Ok(all_constitutions);

	unimplemented!()


	// if a constitution is deleted then all its descendants must either be deleted or have their parent moved

	// starting from the root node, we walk the tree
	// for each node, we look up it's corresponding change (or the items in the tree are tuples of node/change)
	// changes can be:
	// - Keep, this constitution itself is unchanged. however children can still be changed, so we walk down
	// - Delete. we walk all children and ensure all of them are either Delete or have their parent changed to a live node
	// 		this probably means we should walk all the deletes *first*, and then process all normal changes after
	// - Change. this can change any simple field, including changing the parent (which must be changed to something live).
	// if you change a parent, you don't *necessarily* have to change its descendants, even if they all represent geographic areas

	// separately apart from all these mutations there is a "new" constitution list, which must all point to live nodes after all mutations

	// also have to check additions don't conflict with existing

}

// fn _apply_constitution_changes(
// 	arena,
// 	current_node_id,
// 	mutations,
// 	live_constitution_ids,
// 	next_constitutions,
// ) -> Result<> {
// 	use ConstitutionMutation::*;

// 	let current_constitution = arena.get(current_node_id)?.get();
// 	match mutations.get(current_constitution.id)? {
// 		Keep => {
// 			live_constitution_ids.push(current_constitution.id);
// 			next_constitutions.push(current_constitution.clone());
// 		}
// 		Delete => {
// 			// TODO detach subtree
// 			// walk all children and ensure they are deleted or change parents
// 		}
// 		Change(next_constitution) => {
// 			if next_constitution.id != current_constitution.id {
// 				return Err()
// 			}
// 			match (next_constitution.parent_id, current_constitution.parent_id) {
// 				(Some(next_parent_id), Some(current_parent_id)) => {
// 					if next_parent_id != current_parent_id {
// 						current_node_id
// 						// TODO move subtree
// 					}
// 				},
// 				(Some(_), None) => {
// 					unimplemented!();
// 					// TODO attempting to move the root somewhere else
// 				},
// 				(None, Some(_)) => {
// 					unimplemented!();
// 					// TODO attempting to make something else the root
// 				},
// 				_ => {},
// 			}

// 			live_constitution_ids.push(next_constitution.id);
// 			next_constitutions.push(next_constitution);
// 		}
// 	}

// 	for child_node_id in current_node_id.children(arena) {
// 		_apply_constitution_changes(child_node_id, arena, mutations, live_constitution_ids, next_constitutions)
// 	}
// }



#[cfg(test)]
mod tests {
	fn cons(id: usize, name: String, parent_id: Option<usize>) -> Constitution {
		Constitution { id, name, parent_id }
	}

	#[test]
	fn test_create_constitution_tree() {
		let (arena, root_node_id, constitution_node_ids) = create_constitution_tree(vec![
			cons(1, "root".into(), None),
			cons(2, "a".into(), Some(1)),
			cons(3, "b".into(), Some(1)),
		]).unwrap();

		assert_eq!(*arena.get(root_node_id).unwrap().get(), cons(1, "root".into(), None));

		let mut iter = root_node_id.descendants(&arena);
		assert_eq!(iter.next(), Some(*constitution_node_ids.get(&1).unwrap()));
		assert_eq!(iter.next(), Some(*constitution_node_ids.get(&2).unwrap()));
		assert_eq!(iter.next(), Some(*constitution_node_ids.get(&3).unwrap()));
		assert_eq!(iter.next(), None);
		// println!("{:?}\n", root_node_id.debug_pretty_print(&arena));
	}


	#[test]
	fn test_create_constitution_tree_detach() {
		let mut arena = Arena::new();
		let root = arena.new_node("root");
		let a = arena.new_node("a");
		let b = arena.new_node("b");
		root.append(a, &mut arena);
		root.append(b, &mut arena);
		let a1 = arena.new_node("a1");
		let a2 = arena.new_node("a2");
		a.append(a1, &mut arena);
		a.append(a2, &mut arena);
		assert_eq!(root.descendants(&arena).collect::<Vec<NodeId>>(), vec![
			root, a, a1, a2, b,
		]);

		a.detach(&mut arena);
		assert_eq!(root.descendants(&arena).collect::<Vec<NodeId>>(), vec![
			root, b,
		]);

		b.append(a, &mut arena);
		assert_eq!(root.descendants(&arena).collect::<Vec<NodeId>>(), vec![
			root, b, a, a1, a2,
		]);
	}

	#[test]
	fn test_apply_constitution_changes() {
		let before = vec![
			cons(1, "root".into(), None),

			cons(2, "a".into(), Some(1)),
			cons(3, "a_1".into(), Some(2)),
			cons(4, "a_2".into(), Some(2)),

			cons(5, "b".into(), Some(1)),
			cons(6, "b_1".into(), Some(5)),
			cons(7, "b_2".into(), Some(5)),
		];

		let mutations = HashMap::from([
			(1, ConstitutionMutation::Keep),

			(2, ConstitutionMutation::Delete),
			(3, ConstitutionMutation::Change(cons(3, "1_3".into(), Some(5)))),
			(4, ConstitutionMutation::Delete),

			(5, ConstitutionMutation::Change(cons(5, "1".into(), Some(1)))),
			(6, ConstitutionMutation::Change(cons(6, "1_1".into(), Some(5)))),
			(7, ConstitutionMutation::Change(cons(7, "1_2".into(), Some(5)))),
		]);
		let additions = vec![
			cons(8, "2".into(), Some(1)),
			cons(9, "2_1".into(), Some(8)),
			cons(10, "2_2".into(), Some(8)),
			cons(11, "2_3".into(), Some(8)),
		];

		let expected = vec![
			cons(1, "root".into(), None),

			cons(5, "1".into(), Some(1)),
			cons(6, "1_1".into(), Some(5)),
			cons(7, "1_2".into(), Some(5)),
			cons(3, "1_3".into(), Some(5)),

			cons(8, "2".into(), Some(1)),
			cons(9, "2_1".into(), Some(8)),
			cons(10, "2_2".into(), Some(8)),
			cons(11, "2_3".into(), Some(8)),
		];

		let after = apply_constitution_changes(before, mutations, additions).unwrap();
		assert_eq!(after, expected);

		// TODO guard against trivial cases: delete all, keep all
	}

	use super::*;
	use AllocationType::*;

	fn allo(voter_id: usize, (election_id, candidate_id): (usize, usize), weight: Weight, allocation_type: AllocationType) -> Allocation {
		Allocation { voter_id, election_id, candidate_id, weight, allocation_type }
	}

	fn cand((election_id, candidate_id): (usize, usize), stabilization_bucket: Option<Weight>) -> Candidacy {
		Candidacy { election_id, candidate_id, stabilization_bucket }
	}

	#[test]
	fn test_actual_vote() {
		let mut allocation = Allocation {
			voter_id: 0, election_id: 0, candidate_id: 0,
			weight: 4.0,
			allocation_type: For,
		};

		assert_eq!(allocation.actual_vote(), 2.0);

		allocation.weight = 1.0;
		assert_eq!(allocation.actual_vote(), 1.0);

		allocation.allocation_type = Against;
		assert_eq!(allocation.actual_vote(), -1.0);
	}


	#[test]
	fn test_compute_total_votes() {
		let candidacy_vec = vec![cand((1, 1), Some(0.0))];
		let candidacy_map: HashMap<(usize, usize), Weight> = candidacy_vec.iter()
			.map(|c| (c.key(), c.stabilization_bucket.unwrap_or(0.0)))
			.collect();
		dbg!(candidacy_map);




		let candidacy_vec = vec![
			cand((1, 1), None),
		];
		let allocation_vec = vec![
			allo(0, (1, 1), 4.0, For),
			allo(0, (0, 0), 4.0, For),
		];

		assert_eq!(compute_total_votes(candidacy_vec, allocation_vec), HashMap::from([
			((1, 1), 2.0),
			// ((1, 2), 0.7),
			// ((2, 1), 1.0),
			// ((2, 2), 1.0),
		]));
	}
}
