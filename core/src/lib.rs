use std::collections::{HashMap, HashSet};
use core::hash::{Hash, Hasher};
use core::borrow::Borrow;
// use chrono::{DateTime as ChronoDateTime, Utc}

// type DateTime = chrono::DateTime<chrono::Utc>;
type DateTime = i64;

#[derive(Debug)]
struct PolityActionEntry {
	occurred_at: DateTime,
	change: PolityAction,
}

#[derive(Debug)]
enum PolityAction {
	EnterPerson{ person_id: usize, allowed_weight: f64 },
	SetAllocations{ voter_id: usize, allocations: Vec<Allocation> },
	ExitPerson{ person_id: usize },

	EnterCandidacy{ candidacy_id: usize, owner_id: usize, election_id: usize, pitch: String, content: CandidacyContent },
	ExitCandidacy{ candidacy_id: usize },

	Recalculate,
}

#[derive(Debug)]
enum PolityActionError {
	IdConflict{ id: usize, table_kind: TableKind },
	NotFound{ id: usize, table_kind: TableKind },
	NoCandidacy{ candidacy_id: usize, voter_id: usize },
	NoElection{ election_id: usize, voter_id: usize },
	AboveAllowedWeight{ voter_id: usize, found_weight: f64, allowed_weight: f64 },
	MismatchedKind{ candidacy_id: usize, expected_kind: ElectionKind },
	MismatchedMethod{ key: AllocationId, expected_method: SelectionMethod },
	WinningDocumentExit{ candidacy_id: usize },
}


#[derive(Debug, Clone, Copy)]
enum ElectionKind {
	Document,
	Office,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum SelectionMethod {
	Resource{ scale_quadratically: bool },
	ResourceScore{ scale_quadratically: bool },
}

#[derive(Debug)]
enum VoterAllocation {
	Resource(ResourceAllocation),
	ResourceScore(ResourceScoreAllocation),
}

#[derive(Debug)]
struct ResourceAllocation {
	candidacy_id: usize,
	weight: f64,
}

#[derive(Debug)]
struct ResourceScoreAllocation {
	approve_weight: f64,
	disapprove_weight: f64,
	scores: HashMap<usize, f64>,
}


#[derive(Debug)]
struct ElectionDefinition {
	id: usize,
	title: String,
	description: String,
	kind: ElectionKind,
	selection_method: SelectionMethod,

	// negative_buckets: NegativeBucketsKind,
	nomination_fill_method: NominationFillMethod,
	// fill_requirement_method: FillRequirementMethod,
	// update_frequency: chrono::Duration,
}

// #[derive(Debug)]
// enum NegativeBucketsKind {
// 	None,
// 	WithoutRemoval,
// 	WithRemoval,
// }

#[derive(Debug)]
enum NominationFillMethod {
	Constant(f64),
	// NoiseAdaptive,
	None,
}

// #[derive(Debug)]
// enum FillRequirementMethod {
// 	Constant(f64),
// 	OnlyElectorateSize,
// 	ElectorateSizeWithWideness,
// }


#[derive(Debug)]
enum CandidacyStatus {
	Nomination(f64),
	Election(f64),
	Winner,
}


#[derive(Debug)]
enum CandidacyContent {
	Document{ body: String, sub_elections: Vec<ElectionDefinition> },
	Office,
}

fn calculate_polity_action(
	state: &mut PolityState,
	errors: &mut Vec<PolityActionError>,
	changes: &mut Vec<PolityStateChange>,
	action: PolityAction,
) -> Option<()> {
	match action {
		PolityAction::EnterPerson{ person_id, allowed_weight } => {
			// TODO check against polity settings to see if everyone must have some specific allowed weight
			require_not_present(errors, &state.person_table, &person_id)?;
			changes.push(PolityStateChange::InsertPerson{ person_id, allowed_weight });
		},
		PolityAction::SetAllocations{ voter_id, allocations } => {
			let person = require_present(errors, &state.person_table, &voter_id)?;
			let allocations = validate_allocations(errors, state, allocations, person)?;
			changes.push(PolityStateChange::SetAllocations{ voter_id, allocations });
		},
		PolityAction::ExitPerson{ person_id } => {
			require_present(errors, &state.person_table, &person_id)?;
			changes.push(PolityStateChange::RemovePerson{ person_id });
		},

		PolityAction::EnterCandidacy{ candidacy_id, owner_id, election_id, pitch, content } => {
			require_not_present(errors, &state.candidacy_table, &candidacy_id)?;
			require_present(errors, &state.person_table, &owner_id)?;
			let election = require_present(errors, &state.election_table, &election_id)?;
			validate_candidacy_content(errors, &content, election.kind, candidacy_id )?;

			let status = match election.nomination_fill_method {
				NominationFillMethod::Constant(_) => { CandidacyStatus::Nomination(0.0) },
				NominationFillMethod::None => { CandidacyStatus::Election(0.0) },
			};
			let candidacy = Candidacy{ id: candidacy_id, owner_id, election_id, pitch, content, status };
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
			// let grouped_candidacies = group_candidacies_by_election_id(&state.candidacy_table);
			let mut grouped_candidacies = HashMap::new::<usize, HashSet<&Candidacy>>();
			for candidacy in &state.candidacy_table {
				grouped_candidacies
					.entry(candidacy.election_id)
					.or_insert_default()
					.and_modify(|v| v.add(&candidacy));
			}
			let grouped_candidacies = grouped_candidacies;

			// find all allocations and group them by election_id
			let mut allocations_by_election_id = HashMap::new::<usize, Vec<&RawAllocation>>();
			for allocation in &state.allocation_table {
				allocations_by_election_id
					.entry(allocation.election_id)
					.or_insert_default()
					.and_modify(|v| v.push(allocation.raw));
			}
			let allocations_by_election_id = allocations_by_election_id;

			for (election_id, candidacy_set) in grouped_candidacies {
				let election = match require_present(errors, &state.election_table, &election_id) {
					Some(election) => election,
					None => { continue; },
				};
				let unparsed_allocations = match allocations_by_election_id.get(&election_id) {
					Some(unparsed_allocations) => unparsed_allocations,
					None => { continue; },
				};
				let aggregated = match election.selection_method {
					SelectionMethod::Resource => {
						aggregate_election_resource_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::Quadratic => {
						aggregate_election_quadratic_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::ResourceScore => {
						aggregate_election_resource_score_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::QuadraticScore => {
						aggregate_election_quadratic_score_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::ResourceApproval => {
						aggregate_election_resource_approval_votes(parse_votes(errors, unparsed_allocations))
					},
					SelectionMethod::QuadraticApproval => {
						aggregate_election_quadratic_approval_votes(parse_votes(errors, unparsed_allocations))
					},
				}

				// simply ignore (or mark) allocations that point to candidacies that no longer exist, since that's probably not the fault of the voter
				// we just need to notify them to switch their weights, which they can do whenever they want

				// TODO separate them by status
				// calculate their new statii
				// issue candidacy updates for all that changed
				// issue election deletions for elections that are no longer live

			}

			find_current_winner
			calculate_next_stabilization_buckets
		},
	}

	Some(())
}

fn validate_allocations(
	errors: &mut Vec<PolityActionError>,
	state: &PolityState,
	allocations: Vec<Allocation>,
	person: &Person,
) -> Option<Vec<Allocation>> {
	let found_weight = allocations.iter().map(|a| a.total_weight()).sum();
	if found_weight > person.allowed_weight {
		errors.push(PolityActionError::AboveAllowedWeight{ voter_id: person.id, found_weight, allowed_weight: person.allowed_weight });
		return None;
	}

	let mut valid_allocations = Vec::new();
	for allocation in allocations.into_iter() {
		// no need to reject all the allocations if some of them have problems
		// those will be reported as errors, and we'll just drop them on the floor here
		if let Some(mut valid_allocation) = validate_allocation(errors, state, allocation) {
			// just force all allocations to match the right voter
			valid_allocation.key.voter_id = person.id;
			valid_allocations.push(valid_allocation);
		}
	}
	Some(valid_allocations)
}

fn validate_allocation(
	errors: &mut Vec<PolityActionError>,
	state: &PolityState,
	allocation: Allocation,
) -> Option<Allocation> {
	let election_id = allocation.key.election_id;
	let voter_id = allocation.key.voter_id;

	let election = match require_present(errors, &state.election_table, &election_id) {
		Some(e) => e,
		None => {
			errors.push(PolityActionError::NoElection{ election_id, voter_id });
			return None;
		},
	};
	if !allocation.compatible_with_method(&election.selection_method) {
		errors.push(PolityActionError::MismatchedMethod{ key: allocation.key, expected_method: election.selection_method });
		return None;
	}

	for candidacy_id in allocation.iter_candidacies() {
		if !state.candidacy_table.contains(candidacy_id) {
			errors.push(PolityActionError::NoCandidacy{ candidacy_id: *candidacy_id, voter_id });
			return None;
		}
	}

	Some(allocation)
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
		(CandidacyContent::Office, ElectionKind::Office) => { Some(()) },

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




#[derive(Debug)]
enum TableKind {
	Person,
	Election,
	Candidacy,
	Allocation,
}

trait TableKindAble { fn table_kind() -> TableKind; }
trait IdAble { type Id: Copy + Hash; fn get_id(&self) -> &Self::Id; }

macro_rules! impl_id_traits {
	($structname: ident) => {
		impl PartialEq for $structname {
			fn eq(&self, other: &Self) -> bool {
				self.get_id() == other.get_id()
			}
		}
		impl Eq for $structname {}
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
		impl TableKindAble for $structname {
			fn table_kind() -> TableKind { TableKind::$structname }
		}
	};
}


#[derive(Debug)]
struct Person {
	id: usize,
	allowed_weight: f64,
	// name: String,
}
impl IdAble for Person { type Id = usize; fn get_id(&self) -> &Self::Id { &self.id } }
impl_id_traits!(Person);

#[derive(Debug)]
struct Election {
	id: usize,
	title: String,
	description: String,
	nomination_fill_method: NominationFillMethod,
	fill_requirement: f64,
	kind: ElectionKind,
	selection_method: SelectionMethod,
	defining_document_id: Option<usize>,
}
impl IdAble for Election { type Id = usize; fn get_id(&self) -> &Self::Id { &self.id } }
impl_id_traits!(Election);

#[derive(Debug)]
struct Candidacy {
	id: usize,
	owner_id: usize,
	election_id: usize,
	pitch: String,
	content: CandidacyContent,
	status: CandidacyStatus,
}
impl IdAble for Candidacy { type Id = usize; fn get_id(&self) -> &Self::Id { &self.id } }
impl_id_traits!(Candidacy);

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
struct AllocationId {
	voter_id: usize,
	election_id: usize,
}
#[derive(Debug)]
struct Allocation {
	key: AllocationId,
	allocation: VoterAllocation,
}
impl IdAble for Allocation { type Id = AllocationId; fn get_id(&self) -> &Self::Id { &self.key } }
impl_id_traits!(Allocation);

impl Allocation {
	fn total_weight(&self) -> f64 {
		match &self.allocation {
			VoterAllocation::Resource(a) => { a.weight },
			VoterAllocation::ResourceScore(a) => { a.approve_weight + a.disapprove_weight },
		}
	}

	fn compatible_with_method(&self, method: &SelectionMethod) -> bool {
		match (&self.allocation, method) {
			(VoterAllocation::Resource(_), SelectionMethod::Resource{..}) => { true },
			(VoterAllocation::ResourceScore(_), SelectionMethod::ResourceScore{..}) => { true },
			_ => false,
		}
	}

	fn iter_candidacies(&self) -> Vec<&usize> {
		match &self.allocation {
			VoterAllocation::Resource(a) => {
				vec![&a.candidacy_id]
			},
			VoterAllocation::ResourceScore(a) => {
				a.scores.keys().collect()
			},
		}
	}
}


#[derive(Debug)]
struct PolityState {
	person_table: HashSet<Person>,
	election_table: HashSet<Election>,
	candidacy_table: HashSet<Candidacy>,
	allocation_table: HashMap<usize, Vec<Allocation>>,
}

#[derive(Debug)]
enum PolityStateChange {
	InsertPerson{ person_id: usize, allowed_weight: f64 },
	SetAllocations{ voter_id: usize, allocations: Vec<Allocation> },
	RemovePerson{ person_id: usize },

	InsertElection{ election: Election },
	RemoveElection{ election_id: usize },

	InsertCandidacy{ candidacy: Candidacy },
	SetCandidacyStatus{ candidacy_id: usize, status: CandidacyStatus },
	RemoveCandidacy{ candidacy_id: usize },
}

// separating changes into a low level makes it possible to use any other persistence layer, as long as we can somehow serialize to that layer
impl PolityState {
	fn apply_changes(&mut self, changes: Vec<PolityStateChange>) {
		for change in changes.into_iter() {
			self.apply_change(change);
		}
	}

	// all of these functions assume validated inputs, calculate_polity_action is responsible for validation
	fn apply_change(&mut self, change: PolityStateChange) {
		match change {
			PolityStateChange::InsertPerson{ person_id, allowed_weight } => {
				let person = Person{ id: person_id, allowed_weight };
				self.person_table.insert(person);
			},
			PolityStateChange::SetAllocations{ voter_id, allocations } => {
				self.allocation_table.insert(voter_id, allocations);
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



// #[cfg(test)]
// mod tests {

// 	#[test]
// 	fn test__empty() {
// 	}
// }
