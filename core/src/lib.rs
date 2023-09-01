use std::collections::{HashMap, HashSet};
use core::hash::{Hash, Hasher};
use core::borrow::Borrow;
// use chrono::{DateTime as ChronoDateTime, Utc}
use rust_decimal::prelude::*;

// type DateTime = chrono::DateTime<chrono::Utc>;
type DateTime = i64;
type Weight = Decimal;

#[derive(Debug)]
struct PolityActionEntry {
	occurred_at: DateTime,
	change: PolityAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CandidacyContent {
	Office{ pitch: String },
	Document{ pitch: String, body: String, sub_elections: Vec<InputElection> },
}

#[derive(Debug)]
enum PolityAction {
	EnterPerson{ person_id: usize, given_weight: Weight },
	SetAllocations{ voter_id: usize, resource_allocations: Vec<ResourceAllocation>, resource_score_allocations: Vec<ResourceScoreAllocation> },
	ExitPerson{ person_id: usize },

	EnterCandidacy{ candidacy_id: usize, owner_id: usize, election_id: usize, content: CandidacyContent },
	ExitCandidacy{ candidacy_id: usize },

	Recalculate,
}

#[derive(Debug, PartialEq)]
enum PolityActionError {
	IdConflict{ id: usize, table_kind: TableKind },
	NotFound{ id: usize, table_kind: TableKind },
	NoCandidacy{ candidacy_id: usize, voter_id: usize },
	NoElection{ election_id: usize, voter_id: usize },
	NotRequiredEqualWeight{ person_id: usize, found_weight: Weight, required_equal_weight: Weight },
	AboveAllowedWeight{ voter_id: usize, found_weight: Weight, given_weight: Weight },
	MismatchedKind{ candidacy_id: usize, expected_kind: ElectionKind },
	MismatchedMethod{ voter_id: usize, election_id: usize, expected_method: SelectionMethodKind },
	WinningDocumentExit{ candidacy_id: usize },
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ElectionKind {
	Document,
	Office,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SelectionMethodKind {
	Resource,
	ResourceScore,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum SelectionMethod {
	Resource{ scale_quadratically: bool },
	ResourceScore{ scale_quadratically: bool, use_averaging: bool },
}
impl SelectionMethod {
	fn kind(&self) -> SelectionMethodKind {
		match self {
			SelectionMethod::Resource{..} => SelectionMethodKind::Resource,
			SelectionMethod::ResourceScore{..} => SelectionMethodKind::ResourceScore,
		}
	}
}

trait Allocation {
	fn total_weight(&self) -> Weight;
	fn compatible_method_kind() -> SelectionMethodKind;
	fn iter_candidacies(&self) -> Vec<&usize>;
	fn get_election_id(&self) -> usize;
}

#[derive(Debug, PartialEq)]
struct ResourceAllocation {
	election_id: usize,
	candidacy_id: usize,
	weight: Weight,
}

impl Allocation for ResourceAllocation {
	fn total_weight(&self) -> Weight { self.weight }
	fn compatible_method_kind() -> SelectionMethodKind { SelectionMethodKind::Resource }
	fn iter_candidacies(&self) -> Vec<&usize> { vec![&self.candidacy_id] }
	fn get_election_id(&self) -> usize { self.election_id }
}

fn aggregate_resource_votes(allocations: &Vec<&ResourceAllocation>) -> HashMap<usize, Weight> {
	let mut vote_aggregation = HashMap::new();
	for allocation in allocations {
		let actual_vote = allocation.weight;
		vote_aggregation
			.entry(allocation.candidacy_id)
			.and_modify(|t| *t += actual_vote)
			.or_insert(actual_vote);
	}
	vote_aggregation
}
fn aggregate_quadratic_resource_votes(allocations: &Vec<&ResourceAllocation>) -> HashMap<usize, Weight> {
	let mut vote_aggregation = HashMap::new();
	for allocation in allocations {
		let actual_vote = quadratic_vote(allocation.weight);
		vote_aggregation
			.entry(allocation.candidacy_id)
			.and_modify(|t| *t += actual_vote)
			.or_insert(actual_vote);
	}
	vote_aggregation
}


#[derive(Debug, PartialEq)]
struct ResourceScoreAllocation {
	election_id: usize,
	approve_weight: Weight,
	disapprove_weight: Weight,
	scores: HashMap<usize, Weight>,
}

impl Allocation for ResourceScoreAllocation {
	fn total_weight(&self) -> Weight { self.approve_weight + self.disapprove_weight }
	fn compatible_method_kind() -> SelectionMethodKind { SelectionMethodKind::ResourceScore }
	fn iter_candidacies(&self) -> Vec<&usize> { self.scores.keys().collect() }
	fn get_election_id(&self) -> usize { self.election_id }
}

fn aggregate_resource_score_votes(allocations: &Vec<&ResourceScoreAllocation>) -> HashMap<usize, Weight> {
	let mut vote_aggregation = HashMap::new();
	for allocation in allocations {
		let actual_approve_weight = allocation.approve_weight;
		let actual_disapprove_weight = allocation.disapprove_weight;
		for (candidacy_id, score) in &allocation.scores {
			let actual_vote = score * (if *score >= 0.into() { actual_approve_weight } else { actual_disapprove_weight });
			vote_aggregation
				.entry(*candidacy_id)
				.and_modify(|t| *t += actual_vote)
				.or_insert(actual_vote);
		}
	}
	vote_aggregation
}
fn aggregate_quadratic_resource_score_votes(allocations: &Vec<&ResourceScoreAllocation>) -> HashMap<usize, Weight> {
	let mut vote_aggregation = HashMap::new();
	for allocation in allocations {
		let actual_approve_weight = quadratic_vote(allocation.approve_weight);
		let actual_disapprove_weight = quadratic_vote(allocation.disapprove_weight);
		for (candidacy_id, score) in &allocation.scores {
			let actual_vote = score * (if *score >= 0.into() { actual_approve_weight } else { actual_disapprove_weight });
			vote_aggregation
				.entry(*candidacy_id)
				.and_modify(|t| *t += actual_vote)
				.or_insert(actual_vote);
		}
	}
	vote_aggregation
}

fn quadratic_vote(weight: Weight) -> Weight {
	weight.signum() * weight.abs().sqrt().unwrap()
}



#[derive(Debug, Clone, PartialEq, Eq)]
struct InputElection {
	id: usize,
	title: String,
	description: String,
	kind: ElectionKind,
	selection_method: SelectionMethod,

	nomination_fill_method: NominationFillMethod,
	election_fill_method: ElectionFillMethod,
	// negative_buckets: NegativeBucketsKind,
	// update_frequency: chrono::Duration,
}

impl InputElection {
	fn make_election(&self, defining_document_id: usize) -> StorageElection {
		StorageElection {
			id: self.id,
			title: self.title.clone(),
			description: self.description.clone(),
			kind: self.kind,
			selection_method: self.selection_method,
			nomination_fill_method: self.nomination_fill_method,
			election_fill_method: self.election_fill_method,
			defining_document_id: Some(defining_document_id),
		}
	}
}


#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum NominationFillMethod {
	Constant(Weight),
	// NoiseAdaptive,
	None,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ElectionFillMethod {
	Constant(Weight),
	// OnlyElectorateSize,
	// ElectorateSizeWithWideness,
}

// #[derive(Debug)]
// enum NegativeBucketsKind {
// 	None,
// 	WithoutRemoval,
// 	WithRemoval,
// }

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum CandidacyStatus {
	Nomination(Weight),
	Election(Weight),
	Winner,
}


fn calculate_polity_action(
	state: &PolityState,
	errors: &mut Vec<PolityActionError>,
	changes: &mut Vec<PolityStateChange>,
	action: PolityAction,
) -> Option<()> {
	match action {
		PolityAction::EnterPerson{ person_id, given_weight } => {
			if let Some(required_equal_weight) = state.required_equal_weight {
				if given_weight != required_equal_weight {
					errors.push(PolityActionError::NotRequiredEqualWeight{ person_id, found_weight: given_weight, required_equal_weight });
					return None;
				}
			}
			require_not_present(errors, &state.person_table, &person_id)?;
			changes.push(PolityStateChange::InsertPerson{ person_id, given_weight });
		},
		PolityAction::SetAllocations{ voter_id, resource_allocations, resource_score_allocations } => {
			let person = require_present(errors, &state.person_table, &voter_id)?;
			let (resource_allocations, resource_score_allocations) =
				validate_allocations(errors, state, &person, resource_allocations, resource_score_allocations)?;

			changes.push(PolityStateChange::SetResourceAllocations{ voter_id, allocations: resource_allocations });
			changes.push(PolityStateChange::SetResourceScoreAllocations{ voter_id, allocations: resource_score_allocations });
		},
		PolityAction::ExitPerson{ person_id } => {
			require_present(errors, &state.person_table, &person_id)?;
			changes.push(PolityStateChange::RemovePerson{ person_id });
		},

		PolityAction::EnterCandidacy{ candidacy_id, owner_id, election_id, content } => {
			require_not_present(errors, &state.candidacy_table, &candidacy_id)?;
			require_present(errors, &state.person_table, &owner_id)?;
			let election = require_present(errors, &state.election_table, &election_id)?;
			validate_candidacy_content(errors, &content, election.kind, candidacy_id)?;

			let status = make_initial_status(election.nomination_fill_method);
			let candidacy = StorageCandidacy{ id: candidacy_id, owner_id, election_id, content, status };
			changes.push(PolityStateChange::InsertCandidacy{ candidacy });
		},
		PolityAction::ExitCandidacy{ candidacy_id } => {
			let candidacy = require_present(errors, &state.candidacy_table, &candidacy_id)?;
			validate_not_winning_document(errors, &candidacy.status, &candidacy.content, candidacy_id)?;

			// no need to issue election deletions, this isn't allowed to be a document winner
			// similarly no need to delete allocations, we should just ignore allocations to non-existent candidacies
			changes.push(PolityStateChange::RemoveCandidacy{ candidacy_id });
		},

		PolityAction::Recalculate => {
			perform_polity_recalculation(state, errors, changes)?;
		},
	}

	Some(())
}

fn make_initial_status(nomination_fill_method: NominationFillMethod) -> CandidacyStatus {
	match nomination_fill_method {
		NominationFillMethod::Constant(_) => { CandidacyStatus::Nomination(0.into()) },
		NominationFillMethod::None => { CandidacyStatus::Election(0.into()) },
	}
}

fn perform_polity_recalculation(
	state: &PolityState,
	errors: &mut Vec<PolityActionError>,
	changes: &mut Vec<PolityStateChange>,
	// integrity_warnings: &mut Vec<IntegrityWarning>,
) -> Option<()> {
	let mut grouped_candidacies = HashMap::new();
	for candidacy in &state.candidacy_table {
		grouped_candidacies
			.entry(candidacy.election_id)
			.and_modify(|v: &mut HashSet<&StorageCandidacy>| { v.insert(&candidacy); })
			.or_default();
	}
	let grouped_candidacies = grouped_candidacies;

	// find all allocations and group them by election_id
	let mut resource_allocations_by_election_id = HashMap::new();
	for allocation in state.resource_allocation_table.values().flatten() {
		resource_allocations_by_election_id
			.entry(allocation.election_id)
			.and_modify(|v: &mut Vec<&ResourceAllocation>| { v.push(&allocation); })
			.or_default();
	}
	let resource_allocations_by_election_id = resource_allocations_by_election_id;

	let mut resource_score_allocations_by_election_id = HashMap::new();
	for allocation in state.resource_score_allocation_table.values().flatten() {
		resource_score_allocations_by_election_id
			.entry(allocation.election_id)
			.and_modify(|v: &mut Vec<&ResourceScoreAllocation>| { v.push(&allocation); })
			.or_default();
	}
	let resource_score_allocations_by_election_id = resource_score_allocations_by_election_id;

	for (election_id, candidacies) in grouped_candidacies {
		perform_election_recalculation(
			state, errors, changes, election_id, &candidacies,
			&resource_allocations_by_election_id,
			&resource_score_allocations_by_election_id,
		);
	}

	Some(())
}

fn perform_election_recalculation(
	state: &PolityState,
	errors: &mut Vec<PolityActionError>,
	changes: &mut Vec<PolityStateChange>,
	election_id: usize,
	candidacies: &HashSet<&StorageCandidacy>,
	resource_allocations_by_election_id: &HashMap<usize, Vec<&ResourceAllocation>>,
	resource_score_allocations_by_election_id: &HashMap<usize, Vec<&ResourceScoreAllocation>>,
) -> Option<()> {
	// simply ignore (or mark) allocations that point to candidacies that no longer exist, since that's probably not the fault of the voter
	// we just need to notify them to switch their weights, which they can do whenever they want
	let election = require_present(errors, &state.election_table, &election_id)?;
	let empty_resource_allocations = Vec::new();
	let empty_resource_score_allocations = Vec::new();

	let aggregation = match election.selection_method {
		SelectionMethod::Resource{ scale_quadratically } => {
			let allocations = resource_allocations_by_election_id.get(&election_id).unwrap_or(&empty_resource_allocations);
			if !scale_quadratically { aggregate_resource_votes(allocations) }
			else { aggregate_quadratic_resource_votes(allocations) }
		},
		SelectionMethod::ResourceScore{ scale_quadratically, use_averaging: _use_averaging } => {
			let allocations = resource_score_allocations_by_election_id.get(&election_id).unwrap_or(&empty_resource_score_allocations);
			if !scale_quadratically { aggregate_resource_score_votes(allocations) }
			else { aggregate_quadratic_resource_score_votes(allocations) }
		},
	};

	let mut winner_entries = Vec::new();
	let mut candidacy_entries = Vec::new();
	for candidacy in candidacies {
		let total_vote = *aggregation.get(&candidacy.id).unwrap_or(&0.into());
		match candidacy.status {
			CandidacyStatus::Nomination(bucket) => {
				candidacy_entries.push(CandidacyEntry{ candidacy_id: candidacy.id, is_nomination: true, bucket, total_vote });
			},
			CandidacyStatus::Election(bucket) => {
				candidacy_entries.push(CandidacyEntry{ candidacy_id: candidacy.id, is_nomination: false, bucket, total_vote });
			},
			CandidacyStatus::Winner => {
				winner_entries.push((candidacy.id, total_vote));
			},
		}
	}

	// TODO actually calculate these
	let nomination_fill_requirement = 0.into();
	let election_fill_requirement = 0.into();

	// TODO issue a warning if there's more than one winner
	let current_winner = if winner_entries.len() == 1 { Some(winner_entries[0]) } else { None };
	let (new_winner, candidacy_new_statuses) =
		calculate_next_statuses(nomination_fill_requirement, election_fill_requirement, current_winner, candidacy_entries);

	// issue candidacy updates for all that changed
	for (candidacy_id, status) in candidacy_new_statuses {
		changes.push(PolityStateChange::SetCandidacyStatus{ candidacy_id, status });
	}

	if let ElectionKind::Document = election.kind {
		// create sub elections defined by candidacy
		if let Some(new_winner_id) = new_winner {
			if let Some(new_winner_document) = candidacies.get(&new_winner_id) {
				if let CandidacyContent::Document{ sub_elections, .. } = &new_winner_document.content {
					for sub_election in sub_elections {
						changes.push(PolityStateChange::InsertElection{ election: sub_election.make_election(new_winner_document.id) });
					}
				}
			}
		}

		// issue election and candidacy deletions for those no longer live
		if let Some((old_winner_id, _)) = current_winner {
			delete_under_document(state, changes, old_winner_id);

			fn delete_under_document(state: &PolityState, changes: &mut Vec<PolityStateChange>, exiting_candidacy_id: usize) {
				changes.push(PolityStateChange::RemoveCandidacy{ candidacy_id: exiting_candidacy_id });

				for election in state.election_table.iter().filter(|e| e.defining_document_id == Some(exiting_candidacy_id)) {
					let election_id = election.id;
					changes.push(PolityStateChange::RemoveElection{ election_id });

					for child_candidacy in state.candidacy_table.iter().filter(|c| c.election_id == election_id) {
						delete_under_document(state, changes, child_candidacy.id);
					}
				}
			}
		}
	}

	Some(())
}


#[derive(Debug)]
struct CandidacyEntry {
	candidacy_id: usize,
	is_nomination: bool,
	bucket: Weight,
	total_vote: Weight,
}

fn calculate_next_statuses(
	nomination_fill_requirement: Weight,
	election_fill_requirement: Weight,
	current_winner: Option<(usize, Weight)>,
	candidacy_entries: Vec<CandidacyEntry>,
) -> (Option<usize>, HashMap<usize, CandidacyStatus>) {

	let (current_winner_id, current_winner_total_vote) = current_winner.unwrap_or((0, 0.into()));
	let current_winner_id = if current_winner_id == 0 { None } else { Some(current_winner_id) };
	let mut candidacy_new_statuses = HashMap::new();

	let mut positive_filled_maximum = 0.into();
	let mut current_possible_winners = Vec::new();
	for CandidacyEntry{candidacy_id, is_nomination, bucket, total_vote} in candidacy_entries {
		if is_nomination {
			let candidacy_new_bucket = Weight::max(
				bucket + total_vote,
				0.into(),
			);
			let new_status =
				if candidacy_new_bucket >= nomination_fill_requirement { CandidacyStatus::Election(0.into()) }
				else { CandidacyStatus::Nomination(candidacy_new_bucket) };

			candidacy_new_statuses.insert(candidacy_id, new_status);
		}
		else {
			// TODO consider allowing buckets to *go negative* if total_vote is negative, and even possibly *removing* a candidate if they reach *negative* fill_requirement
			let candidacy_new_bucket = Weight::max(
				bucket + (total_vote - current_winner_total_vote),
				0.into(),
			);
			candidacy_new_statuses.insert(candidacy_id, CandidacyStatus::Election(candidacy_new_bucket));
			// it isn't sound to declare the mere highest candidate the new winner when there isn't a current winner
			// doing so would be vulnerable, where a highly approved current winner resigns, allowing a weak challenger to immediately take the stabilized spot
			// it makes sense to *always* require a bucket fill even in situations where there isn't a current winner
			// the alternative would be to simply change fill_requirement to 0 if there isn't a current winner

			// if this candidacy has reached the requirement then it has the chance to be the *unique* winner
			if total_vote <= 0.into() || candidacy_new_bucket < election_fill_requirement { continue; }

			if total_vote == positive_filled_maximum {
				current_possible_winners.push(candidacy_id);
			}
			else if total_vote > positive_filled_maximum {
				positive_filled_maximum = total_vote;
				current_possible_winners.clear();
				current_possible_winners.push(candidacy_id);
			}
		}
	}

	let new_winner =
		// there's a new unique winner
		if current_possible_winners.len() == 1 {
			let new_winner_id = current_possible_winners[0];
			candidacy_new_statuses.insert(new_winner_id, CandidacyStatus::Winner);
			Some(new_winner_id)
		}
		// there's a tie or no one met the requirements
		else {
			// the current winner (if there is one) remains the current winner
			if let Some(winner_id) = current_winner_id {
				candidacy_new_statuses.insert(winner_id, CandidacyStatus::Winner);
			}
			None
		};

	(new_winner, candidacy_new_statuses)
}


fn validate_allocations(
	errors: &mut Vec<PolityActionError>,
	state: &PolityState,
	person: &StoragePerson,
	resource_allocations: Vec<ResourceAllocation>,
	resource_score_allocations: Vec<ResourceScoreAllocation>,
) -> Option<(Vec<ResourceAllocation>, Vec<ResourceScoreAllocation>)> {
	let found_weight =
		resource_allocations.iter().map(|a| a.total_weight()).sum::<Weight>()
		+ resource_score_allocations.iter().map(|a| a.total_weight()).sum::<Weight>();
	if found_weight > person.given_weight {
		errors.push(PolityActionError::AboveAllowedWeight{ voter_id: person.id, found_weight, given_weight: person.given_weight });
		return None;
	}

	let valid_resource_allocations = resource_allocations.into_iter()
		.filter_map(|allocation| validate_allocation(errors, state, person.id, allocation))
		.collect();
	let valid_resource_score_allocations = resource_score_allocations.into_iter()
		.filter_map(|allocation| validate_allocation(errors, state, person.id, allocation))
		.collect();

	Some((valid_resource_allocations, valid_resource_score_allocations))
}

fn validate_allocation<A: Allocation>(
	errors: &mut Vec<PolityActionError>,
	state: &PolityState,
	voter_id: usize,
	allocation: A,
) -> Option<A> {
	let election_id = allocation.get_election_id();

	let election = match require_present(errors, &state.election_table, &election_id) {
		Some(e) => e,
		None => {
			errors.push(PolityActionError::NoElection{ election_id, voter_id });
			return None;
		},
	};
	let expected_method = election.selection_method.kind();
	if A::compatible_method_kind() != expected_method {
		errors.push(PolityActionError::MismatchedMethod{ voter_id, election_id, expected_method });
		return None;
	}

	let mut have_errors = false;
	for candidacy_id in allocation.iter_candidacies() {
		if !state.candidacy_table.contains(candidacy_id) {
			errors.push(PolityActionError::NoCandidacy{ candidacy_id: *candidacy_id, voter_id });
			have_errors = true;
		}
	}

	if !have_errors { Some(allocation) } else { None }
}

fn validate_not_winning_document(
	errors: &mut Vec<PolityActionError>,
	status: &CandidacyStatus,
	content: &CandidacyContent,
	candidacy_id: usize,
) -> Option<()> {
	match (status, content) {
		(CandidacyStatus::Winner, CandidacyContent::Document{..}) => {
			errors.push(PolityActionError::WinningDocumentExit{ candidacy_id });
			None
		},
		_ => Some(())
	}
}

fn validate_candidacy_content(
	errors: &mut Vec<PolityActionError>,
	content: &CandidacyContent,
	election_kind: ElectionKind,
	candidacy_id: usize,
) -> Option<()> {
	match (content, election_kind) {
		(CandidacyContent::Document{..}, ElectionKind::Document) => { Some(()) },
		(CandidacyContent::Office{..}, ElectionKind::Office) => { Some(()) },

		(_, _) => {
			errors.push(PolityActionError::MismatchedKind{ candidacy_id, expected_kind: election_kind });
			None
		},
	}
}

fn require_not_present<T: Borrow<usize> + TableKindAble + Hash + Eq>(
	errors: &mut Vec<PolityActionError>,
	table: &HashSet<T>,
	id: &usize,
) -> Option<()> {
	if table.contains(id) {
		let table_kind = T::table_kind();
		errors.push(PolityActionError::IdConflict{ id: *id, table_kind });
		return None;
	}
	Some(())
}
fn require_present<'t, T: Borrow<usize> + TableKindAble + Hash + Eq>(
	errors: &mut Vec<PolityActionError>,
	table: &'t HashSet<T>,
	id: &usize,
) -> Option<&'t T> {
	match table.get(id) {
		None => {
			let table_kind = T::table_kind();
			errors.push(PolityActionError::NotFound{ id: *id, table_kind });
			None
		},
		item => item,
	}
}




#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum TableKind {
	StoragePerson,
	StorageElection,
	StorageCandidacy,
	ResourceAllocation,
	ResourceScoreAllocation,
}

trait TableKindAble { fn table_kind() -> TableKind; }
trait IdAble { type Id: Copy + Hash; fn get_id(&self) -> &Self::Id; }

macro_rules! impl_id_traits {
	($structname: ident) => {
		impl Hash for $structname {
			fn hash<H: Hasher>(&self, state: &mut H) {
				self.get_id().hash(state);
			}
		}
		impl Borrow<<$structname as IdAble>::Id> for $structname {
			fn borrow(&self) -> &<$structname as IdAble>::Id {
				&self.get_id()
			}
		}
		impl Borrow<<$structname as IdAble>::Id> for &$structname {
			fn borrow(&self) -> &<$structname as IdAble>::Id {
				&self.get_id()
			}
		}
		impl TableKindAble for $structname {
			fn table_kind() -> TableKind { TableKind::$structname }
		}
	};
}


#[derive(Debug, PartialEq, Eq)]
struct StoragePerson {
	id: usize,
	given_weight: Weight,
	// name: String,
}
impl IdAble for StoragePerson { type Id = usize; fn get_id(&self) -> &Self::Id { &self.id } }
impl_id_traits!(StoragePerson);

#[derive(Debug, PartialEq, Eq)]
struct StorageElection {
	id: usize,
	title: String,
	description: String,
	kind: ElectionKind,
	nomination_fill_method: NominationFillMethod,
	election_fill_method: ElectionFillMethod,
	selection_method: SelectionMethod,
	defining_document_id: Option<usize>,
}
impl IdAble for StorageElection { type Id = usize; fn get_id(&self) -> &Self::Id { &self.id } }
impl_id_traits!(StorageElection);


#[derive(Debug, PartialEq, Eq)]
struct StorageCandidacy {
	id: usize,
	owner_id: usize,
	election_id: usize,
	status: CandidacyStatus,
	content: CandidacyContent,
}
impl IdAble for StorageCandidacy { type Id = usize; fn get_id(&self) -> &Self::Id { &self.id } }
impl_id_traits!(StorageCandidacy);


#[derive(Debug)]
struct PolityState {
	required_equal_weight: Option<Weight>,

	person_table: HashSet<StoragePerson>,

	// root_constitution_election: StorageElection,
	election_table: HashSet<StorageElection>,
	candidacy_table: HashSet<StorageCandidacy>,

	resource_allocation_table: HashMap<usize, Vec<ResourceAllocation>>,
	resource_score_allocation_table: HashMap<usize, Vec<ResourceScoreAllocation>>,
}

#[derive(Debug, PartialEq)]
enum PolityStateChange {
	InsertPerson{ person_id: usize, given_weight: Weight },
	SetResourceAllocations{ voter_id: usize, allocations: Vec<ResourceAllocation> },
	SetResourceScoreAllocations{ voter_id: usize, allocations: Vec<ResourceScoreAllocation> },
	RemovePerson{ person_id: usize },

	InsertElection{ election: StorageElection },
	RemoveElection{ election_id: usize },

	InsertCandidacy{ candidacy: StorageCandidacy },
	SetCandidacyStatus{ candidacy_id: usize, status: CandidacyStatus },
	RemoveCandidacy{ candidacy_id: usize },
}

// separating changes into a low level makes it possible to use any other persistence layer, as long as we can somehow serialize to that layer
impl PolityState {
	// fn get_election(&self, election_id: Option<usize>) -> Option<&StorageElection> {
	// 	match election_id {
	// 		None => { self.root_constitution_election },
	// 		Some(election_id) => { self.election_table.get(election_id) },
	// 	}
	// }

	fn build() -> PolityStateBuilder { PolityStateBuilder::new() }

	fn apply_changes(&mut self, changes: Vec<PolityStateChange>) {
		for change in changes.into_iter() {
			self.apply_change(change);
		}
	}

	// all of these functions assume validated inputs, calculate_polity_action is responsible for validation
	fn apply_change(&mut self, change: PolityStateChange) {
		match change {
			PolityStateChange::InsertPerson{ person_id, given_weight } => {
				let person = StoragePerson{ id: person_id, given_weight };
				self.person_table.insert(person);
			},
			PolityStateChange::SetResourceAllocations{ voter_id, allocations } => {
				self.resource_allocation_table.insert(voter_id, allocations);
			},
			PolityStateChange::SetResourceScoreAllocations{ voter_id, allocations } => {
				self.resource_score_allocation_table.insert(voter_id, allocations);
			},
			PolityStateChange::RemovePerson{ person_id } => {
				self.person_table.remove(&person_id);
			},

			PolityStateChange::InsertElection{ election } => {
				self.election_table.insert(election);
			},
			PolityStateChange::RemoveElection{ election_id } => {
				self.election_table.remove(&election_id);
			},

			PolityStateChange::InsertCandidacy{ candidacy } => {
				self.candidacy_table.insert(candidacy);
			},
			PolityStateChange::SetCandidacyStatus{ candidacy_id, status } => {
				if let Some(mut candidacy) = self.candidacy_table.take(&candidacy_id) {
					candidacy.status = status;
					self.candidacy_table.insert(candidacy);
				}
			},
			PolityStateChange::RemoveCandidacy{ candidacy_id } => {
				self.candidacy_table.remove(&candidacy_id);
			},
		}
	}
}


#[derive(Debug)]
struct PolityStateBuilder {
	required_equal_weight: Option<Weight>,
	root_constitution: StorageElection,
}

impl PolityStateBuilder {
	fn new() -> PolityStateBuilder {
		PolityStateBuilder {
			required_equal_weight: None,
			root_constitution: StorageElection {
				id: 0,
				title: "root constitution".into(),
				description: "root constitution".into(),
				kind: ElectionKind::Document,
				nomination_fill_method: NominationFillMethod::None,
				election_fill_method: ElectionFillMethod::Constant(100.into()),
				selection_method: SelectionMethod::ResourceScore{ scale_quadratically: false, use_averaging: false },
				defining_document_id: None,
			}
		}
	}
	fn with_required_equal_weight(mut self, required_equal_weight: Weight) -> PolityStateBuilder {
		self.required_equal_weight = Some(required_equal_weight);
		self
	}
	fn with_resource(mut self) -> PolityStateBuilder {
		self.root_constitution.selection_method = SelectionMethod::Resource{ scale_quadratically: false };
		self
	}
	fn with_resource_score(mut self) -> PolityStateBuilder {
		self.root_constitution.selection_method = SelectionMethod::ResourceScore{ scale_quadratically: false, use_averaging: false };
		self
	}
	fn with_quadratic_resource(mut self) -> PolityStateBuilder {
		self.root_constitution.selection_method = SelectionMethod::Resource{ scale_quadratically: true };
		self
	}
	fn with_quadratic_resource_score(mut self) -> PolityStateBuilder {
		self.root_constitution.selection_method = SelectionMethod::ResourceScore{ scale_quadratically: true, use_averaging: false };
		self
	}
	fn finish(self) -> PolityState {
		PolityState {
			required_equal_weight: self.required_equal_weight,
			person_table: HashSet::new(),
			election_table: HashSet::from([self.root_constitution]), candidacy_table: HashSet::new(),
			resource_allocation_table: HashMap::new(), resource_score_allocation_table: HashMap::new(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_basic_actions() {
		let mut state = PolityState::build().finish();
		let mut errors = Vec::new();

		// success EnterPerson
		let mut changes = Vec::new(); errors.clear();
		let action = PolityAction::EnterPerson{ person_id: 1, given_weight: 10.into() };
		assert!(calculate_polity_action(&state, &mut errors, &mut changes, action).is_some());
		assert_eq!(errors, vec![]);
		assert_eq!(changes, vec![PolityStateChange::InsertPerson{ person_id: 1, given_weight: 10.into() }]);
		state.apply_changes(changes);

		// fail EnterPerson (id conflict)
		let mut changes = Vec::new(); errors.clear();
		let action = PolityAction::EnterPerson{ person_id: 1, given_weight: 10.into() };
		assert!(calculate_polity_action(&state, &mut errors, &mut changes, action).is_none());
		assert_eq!(errors, vec![PolityActionError::IdConflict{ id: 1, table_kind: TableKind::StoragePerson }]);
		assert_eq!(changes, vec![]);

		// success ExitPerson
		let mut changes = Vec::new(); errors.clear();
		let action = PolityAction::ExitPerson{ person_id: 1 };
		assert!(calculate_polity_action(&state, &mut errors, &mut changes, action).is_some());
		assert_eq!(errors, vec![]);
		assert_eq!(changes, vec![PolityStateChange::RemovePerson{ person_id: 1 }]);

		// fail ExitPerson (person not found)
		let mut changes = Vec::new(); errors.clear();
		let action = PolityAction::ExitPerson{ person_id: 2 };
		assert!(calculate_polity_action(&state, &mut errors, &mut changes, action).is_none());
		assert_eq!(errors, vec![PolityActionError::NotFound{ id: 2, table_kind: TableKind::StoragePerson }]);
		assert_eq!(changes, vec![]);

		// success SetAllocations
		let mut changes = Vec::new(); errors.clear();
		let action = PolityAction::SetAllocations{ voter_id: 1, resource_allocations: vec![], resource_score_allocations: vec![] };
		assert!(calculate_polity_action(&state, &mut errors, &mut changes, action).is_some());
		assert_eq!(errors, vec![]);
		assert_eq!(changes, vec![
			PolityStateChange::SetResourceAllocations{ voter_id: 1, allocations: vec![] },
			PolityStateChange::SetResourceScoreAllocations{ voter_id: 1, allocations: vec![] },
		]);

		// fail SetAllocations (person not found)
		let mut changes = Vec::new(); errors.clear();
		let action = PolityAction::SetAllocations{ voter_id: 2, resource_allocations: vec![], resource_score_allocations: vec![] };
		assert!(calculate_polity_action(&state, &mut errors, &mut changes, action).is_none());
		assert_eq!(errors, vec![PolityActionError::NotFound{ id: 2, table_kind: TableKind::StoragePerson }]);
		assert_eq!(changes, vec![]);

		// success EnterCandidacy (intended winner document under root)
		let mut changes = Vec::new(); errors.clear();
		let new_content = CandidacyContent::Document{
			pitch: "gonna win".into(), body: "".into(), sub_elections: vec![InputElection {
				id: 1,
				title: "gonna win doc".into(),
				description: "".into(),
				kind: ElectionKind::Office,
				selection_method: SelectionMethod::ResourceScore{scale_quadratically: false, use_averaging: false},
				nomination_fill_method: NominationFillMethod::Constant(10.into()),
				election_fill_method: ElectionFillMethod::Constant(20.into()),
			}],
		};
		let action = PolityAction::EnterCandidacy{ candidacy_id: 10, owner_id: 1, election_id: 0, content: new_content.clone() };
		assert!(calculate_polity_action(&state, &mut errors, &mut changes, action).is_some());
		assert_eq!(errors, vec![]);
		assert_eq!(changes, vec![
			PolityStateChange::InsertCandidacy{ candidacy: StorageCandidacy {
				id: 10, owner_id: 1, election_id: 0, content: new_content, status: CandidacyStatus::Election(0.into()),
			} },
		]);
		state.apply_changes(changes);

		// success EnterCandidacy (intended loser document under root)
		// success SetAllocations (make above outcomes true)
		// fail SetAllocations (too much weight)
		// fail SetAllocations (NoElection)
		// fail SetAllocations (MismatchedMethod)
		// fail SetAllocations (NoCandidacy, do multiple)
		// Recalculate
		// observe that now new elections undder winner exist

		// success EnterCandidacy (office)
		// fail EnterCandidacy (either) (id conflict)
		// fail EnterCandidacy (either) (owner not found)
		// fail EnterCandidacy (either) (election not found)
		// fail EnterCandidacy (office) (content mismatched)
		// fail EnterCandidacy (document) (content mismatched)

		// success ExitCandidacy (document)
		// success ExitCandidacy (office)
		// fail ExitCandidacy (either) (candidacy not found)
		// fail ExitCandidacy (document) (winner)
	}

	// some possible properties
	// - it's impossible to do anything for a person/candidate/election that doesn't exist
	// - id conflicts are always prevented
	// - voting with greater than given_weight is always prevented
	// - kind mismatches are always caught
	// - scale_quadratically is always respected

	// in general most tests will focus around perform_polity_recalculation, but especially perform_election_recalculation and calculate_next_statuses

	// #[test]
	// fn test_calculate_next_statuses() {
	// 	// CandidacyEntry {
	// 	// 	candidacy_id: usize,
	// 	// 	is_nomination: bool,
	// 	// 	bucket: Weight,
	// 	// 	total_vote: Weight,
	// 	// }

	// 	let (new_winner, candidacy_new_statuses) =
	// 		calculate_next_statuses(nomination_fill_requirement, election_fill_requirement, current_winner, candidacy_entries);
	// }

	// https://proptest-rs.github.io/proptest/proptest/tutorial/compound-strategies.html
	// https://docs.rs/proptest/latest/proptest/index.html
	// use proptest::prelude::*;

	// proptest! {
	// 	#[test]
	// 	fn test_add(a in 0..1000i32, b in 0..1000i32) {
	// 		let sum = a + b;
	// 		prop_assert!(sum >= a);
	// 		prop_assert!(sum >= b);
	// 	}
	// }
}
