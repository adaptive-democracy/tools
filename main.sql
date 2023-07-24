drop schema public cascade;
create schema public;
grant all on schema public to public;
comment on schema public is 'standard public schema';


-- https://clarkdave.net/2015/02/historical-records-with-postgresql-and-temporal-tables-and-sql-2011/
-- https://github.com/arkhipov/temporal_tables
-- https://github.com/tcdi/pgx

create table person (
	id uuid primary key default gen_random_uuid(),
	"name" text not null
);

create type election_kind as enum('DOCUMENT', 'OFFICE');

-- what happens to elections when their defining document isn't the current winner?

create table election (
	id uuid primary key default gen_random_uuid(),
	kind election_kind not null,
	unique (id, kind),

	defining_document_id uuid,
	_defining_document_kind election_kind not null default 'DOCUMENT' check (_defining_document_kind = 'DOCUMENT'),
	-- the single root election must be a document election
	check (defining_document_id is not null or kind = 'DOCUMENT'),

	title text not null,
	description text not null
);
-- only one root election allowed
create unique index idx_root_election on election(defining_document_id) nulls not distinct where defining_document_id is null;

-- documents are their own candidacy, unless we choose to put drafts in the same table?
create table candidacy (
	id uuid not null default gen_random_uuid(),
	-- OFFICE candidacy owners are the candidacy themselves
	owner_id uuid not null references person(id),

	election_id uuid not null,
	primary key (id, election_id),

	kind election_kind not null,
	unique (id, kind),
	foreign key (election_id, kind) references election(id, kind),

	"content" text not null
);
create unique index idx_unique_office_candidacy on candidacy(owner_id, election_id) where kind = 'OFFICE';

alter table election add constraint election_defining_document_fk
foreign key (defining_document_id, _defining_document_kind) references candidacy(id, kind);


create table allocation_update (
	voter_id uuid not null references person(id),
	occurred_at timestamptz not null default current_timestamp,
		-- check (occurred_at < now()),
	primary key (voter_id, occurred_at)
);

create table allocation (
	voter_id uuid not null,
	occurred_at timestamptz not null,
	foreign key (voter_id, occurred_at) references allocation_update(voter_id, occurred_at),
	candidacy_id uuid not null,
	-- election_id uuid not null,
	-- foreign key (candidacy_id, election_id) references candidacy(id, election_id),
	foreign key (candidacy_id) references candidacy(id),
	unique (voter_id, occurred_at, candidacy_id),

	weight numeric not null check (weight != 0)
);

-- TODO constraint to make sure weight doesn't exceed max for voter_id

create type new_allocation as (
	candidacy_id uuid,
	weight numeric
);

create function check_allocations_valid(new_allocations new_allocation[]) returns new_allocation[]
language plpgsql as $$
	begin
		if (select coalesce(sum(abs(weight)), 0) from unnest(new_allocations)) > 100 then
			raise exception 'allocating too much weight';
		else
			return new_allocations;
		end if;
	end;
$$;

-- -- TODO make this concurrency safe https://www.cybertec-postgresql.com/en/triggers-to-enforce-constraints/
-- create or replace function allocation_weight_valid() returns trigger
-- language plpgsql as $$
-- 	begin
-- 		if (new.weight + (select coalesce(sum(weight), 0) from allocation where voter_id = new.voter_id)) <= 100 then
-- 			return new;
-- 		else
-- 			raise exception 'allocated too much weight';
-- 		end if;
-- 	end;
-- $$;
-- create trigger check_allocation_weight_valid
-- before insert or update on allocation
-- for each row
-- execute procedure allocation_weight_valid();

create procedure perform_allocation_update(allocating_voter_id uuid, new_allocations new_allocation[])
language sql as $$

	with
	this_allocation_update as (
		insert into allocation_update (voter_id) values (allocating_voter_id)
		returning occurred_at
	)

	insert into allocation (voter_id, occurred_at, candidacy_id, election_id, weight)
	select
		allocating_voter_id, this_allocation_update.occurred_at, new_allocation.candidacy_id, candidacy.election_id, new_allocation.weight
	from
		this_allocation_update, unnest(check_allocations_valid(new_allocations)) as new_allocation
		join candidacy on new_allocation.candidacy_id = candidacy.id;

$$;




-- TODO constraint enforcing allocation_update to only insert occurred_at larger than existing

-- 	-- this exclusion constraint is equivalent to partial unique index, except it's deferrable
-- 	-- create unique index unique_winner on candidacy(election_id) where stabilization_bucket is null
-- 	-- https://dba.stackexchange.com/questions/166082/deferrable-unique-index-in-postgres
-- 	constraint unique_winner
-- 		exclude (election_id with =)
-- 		where (stabilization_bucket is null)
-- 		deferrable initially deferred

-- TODO constraint enforcing only open range is max

-- create index index_allocation_voter_id on allocation(occurred_at);


create table vote_update (
	-- the upper bound is not inclusive, representing the precise moment the update happened
	occurred_at timestamptz primary key default current_timestamp
);

create table candidacy_vote_update (
	occurred_at timestamptz not null references vote_update(occurred_at),
	candidacy_id uuid not null,
	election_id uuid not null,
	primary key (occurred_at, candidacy_id),
	-- unique (candidacy_id, election_id),
	foreign key (candidacy_id, election_id) references candidacy(id, election_id),

	total_vote numeric not null,
	stabilization_bucket numeric default 0 check (stabilization_bucket >= 0)
);

-- at most one winning candidacy at a time
create unique index idx_winning_candidacy on candidacy_vote_update(occurred_at, election_id, stabilization_bucket)
nulls not distinct where stabilization_bucket is null;


create view current_vote_update as
select max(occurred_at) as occurred_at
from vote_update;

create view current_candidacy_vote_update as
select candidacy_vote_update.*
from
	candidacy_vote_update
	join current_vote_update using (occurred_at);

create view almost_current_candidacy as
select
	candidacy.*,
	current_candidacy_vote_update.occurred_at as recalculated_at,
	coalesce(current_candidacy_vote_update.total_vote, 0) as total_vote,
	case
		when current_candidacy_vote_update.occurred_at is not null then
			current_candidacy_vote_update.stabilization_bucket
		else 0
	end as raw_stabilization_bucket
from
	candidacy
	left join current_candidacy_vote_update on current_candidacy_vote_update.candidacy_id = candidacy.id;

create view current_election as
select
	election.*,
	election.defining_document_id is null or (defining_document.id is not null and defining_document.raw_stabilization_bucket is null) as is_live
from
	election
	left join almost_current_candidacy as defining_document on election.defining_document_id = defining_document.id;

create view current_candidacy as
select
	almost_current_candidacy.*,
	case when current_election.is_live then raw_stabilization_bucket else 0 end as stabilization_bucket,
	case when current_election.is_live then raw_stabilization_bucket is null else false end as is_winner,
	current_election.is_live as is_live
from
	almost_current_candidacy
	join current_election on almost_current_candidacy.election_id = current_election.id;

create view current_candidacy_winner as
select current_candidacy.*
from current_candidacy
where current_candidacy.is_winner;

create view full_allocation as
select allocation.*, not candidacy.is_live as is_orphaned
from
	allocation
	join current_candidacy as candidacy on allocation.candidacy_id = candidacy.id;


create function compute_vote(weight numeric) returns numeric
immutable
language sql as $$
	-- select sign(weight) * sqrt(abs(weight))
	select weight
$$;

create procedure perform_vote_update()
language sql as $$

	with
	next_allocation_update as (
		select
			voter_id,
			max(occurred_at) as occurred_at
		from allocation_update
		group by voter_id
	),
	next_allocation as (
		select allocation.*
		from allocation join next_allocation_update using (voter_id, occurred_at)
	),

	next_candidacy_votes as (
		select
			candidacy.election_id,
			candidacy.id as candidacy_id,
			max(case when candidacy.is_live then candidacy.stabilization_bucket else 0 end) as stabilization_bucket,
			coalesce(sum(case when candidacy.is_live then compute_vote(next_allocation.weight) else 0 end), 0) as total_vote
		from
			current_candidacy as candidacy
			left join next_allocation on candidacy.id = next_allocation.candidacy_id
		-- group by candidacy.election_id, candidacy.id, candidacy.stabilization_bucket
		group by candidacy.election_id, candidacy.id, candidacy.is_live
	),

	next_candidacy as (
		select
			candidacy.election_id as election_id, candidacy.candidacy_id, candidacy.total_vote,

			case
				-- current candidacy is the winner, will be elsewhere updated to 0 if overtaken
				when candidacy.stabilization_bucket is null then null
				-- current candidacy isn't the winner
				else greatest(
					-- if the candidacy has never overtaken the winner, their total_vote will be 0 and the greatest op keeps them there
					-- if the candidacy has overtaken the winner either now or previously, then this difference op makes sense
					-- (will be positive when candidacy is ahead, will be negative if they've fallen behind)
					candidacy.stabilization_bucket + (candidacy.total_vote - current_winner.total_vote),
					-- NOTE the fill amount should be designed to make sense when a raw difference is taken
					0
				)
			end as stabilization_bucket
		from
			next_candidacy_votes as candidacy
			left join current_candidacy_winner as current_winner using (election_id)
	),

	election_maxes as (
		select
			election_id,
			-- if all candidacies are negative or zero, then:
			-- - if there isn't a current winner the new current winner must have non-negative votes
			-- - if there is a current winner who is negative and someone is less negative then the less negative should fill
			-- - if there is a current winner who is zero and everyone is negative or zero then no one should fill
			max(total_vote) as max_votes,
			-- TODO arbitrary fill requirement for now
			max(case when stabilization_bucket is not null and stabilization_bucket >= 10 then stabilization_bucket else null end) as max_bucket

		from next_candidacy
		group by election_id
	),

	max_filled as (
		select c.election_id, c.stabilization_bucket, count(c.candidacy_id) as count_candidacy
		from
			next_candidacy as c
			join election_maxes as m on c.election_id = m.election_id and c.stabilization_bucket = m.max_bucket
		group by c.election_id, c.stabilization_bucket
	),

	max_votes as (
		select c.election_id, c.total_vote, count(c.candidacy_id) as count_candidacy
		from
			next_candidacy as c
			join election_maxes as m on c.election_id = m.election_id and c.total_vote = m.max_votes
		group by c.election_id, c.total_vote
	),

	this_vote_update as (
		insert into vote_update (occurred_at) values (current_timestamp)
		returning occurred_at
	)

	insert into candidacy_vote_update (occurred_at, candidacy_id, election_id, total_vote, stabilization_bucket)
	select
		this_vote_update.occurred_at, c.candidacy_id, c.election_id, c.total_vote,
		case
			-- there's no current winner
			when current_winner.total_vote is null then
				case
					-- this row uniquely has max non-negative votes
					when max_votes.count_candidacy = 1 and max_votes.total_vote >= 0 and c.total_vote = max_votes.total_vote then null
					-- there's a tie, so we simply "do nothing", making no one the winner yet and keeping all the buckets at 0
					else 0
				end

			-- there's a new unique winner
			when max_filled.count_candidacy = 1 and max_filled.stabilization_bucket is not null then
				case
					-- this row is the new winner
					when c.stabilization_bucket = max_filled.stabilization_bucket then null
					-- this row is not the new winner
					else 0
				end

			-- otherwise there's a tie for max filled bucket or no one has filled the bucket
			else c.stabilization_bucket

		end as stabilization_bucket

	from
		this_vote_update, next_candidacy as c
		left join current_candidacy_winner as current_winner using (election_id)
		left join max_votes using (election_id)
		left join max_filled using (election_id)

$$;





create function u(i text) returns uuid immutable language sql as $$
	select lpad(i, 32, '0')::uuid;
$$;

create function r() returns uuid immutable language sql as $$
	select 'ffffffffffffffffffffffffffffffff'::uuid;
$$;

insert into election (id, kind, defining_document_id, title, description) values
(r(), 'DOCUMENT', null, 'root constitution', 'root constitution');

\echo 'current_election';
select title, description, kind, is_live from current_election;
\echo 'current_candidacy';
select "content", total_vote, stabilization_bucket, is_winner, is_live from current_candidacy;


insert into person (id, "name") values
(u('1'), 'han'),
(u('2'), 'luke'),
(u('3'), 'leia'),
(u('4'), 'lando'),
(u('5'), 'ackbar'),
(u('6'), 'mothma');

insert into candidacy (id, kind, owner_id, election_id, "content") values
(u('1'), 'DOCUMENT', u('1'), r(), 'han cand'),
(u('2'), 'DOCUMENT', u('2'), r(), 'luke cand');

insert into election (id, kind, defining_document_id, title, description) values
(u('11'), 'DOCUMENT', u('1'), 'hand cand doc 11', ''),
(u('12'), 'DOCUMENT', u('1'), 'hand cand doc 12', ''),

(u('21'), 'OFFICE', u('2'), 'luke cand off 21', ''),
(u('22'), 'OFFICE', u('2'), 'luke cand off 22', ''),
(u('23'), 'OFFICE', u('2'), 'luke cand off 23', '');

\echo 'current_election';
select title, description, kind, is_live from current_election;
\echo 'current_candidacy';
select "content", total_vote, stabilization_bucket, is_winner, is_live from current_candidacy;

call perform_allocation_update(u('1'), ARRAY[
	row(u('1'), 50)::new_allocation,
	row(u('2'), -50)::new_allocation
]);

call perform_vote_update();
\echo 'current_election';
select title, description, kind, is_live from current_election;
\echo 'current_candidacy';
select "content", total_vote, stabilization_bucket, is_winner, is_live from current_candidacy;


insert into candidacy (id, kind, owner_id, election_id, "content") values
(u('211'), 'OFFICE', u('3'), u('21'), 'leia to luke cand');

call perform_allocation_update(u('3'), ARRAY[
	row(u('211'), 100)::new_allocation
]);

call perform_vote_update();
\echo 'current_election';
select title, description, kind, is_live from current_election;
\echo 'current_candidacy';
select "content", total_vote, stabilization_bucket, is_winner, is_live from current_candidacy;






-- -- insert into person (id, "name") values (1, 'han'), (2, 'luke'), (3, 'leia'), (4, 'vader'), (5, 'palpatine'), (6, 'lando'), (7, 'jabba');
-- -- insert into election (id, title) values (1, 'chancellor'), (2, 'hutt');

-- -- -- for chancellor: leia, vader, palpatine
-- -- insert into candidacy (election_id, candidacy_id) values (1, 3), (1, 4), (1, 5);
-- -- -- for hutt: han, lando, jabba
-- -- insert into candidacy (election_id, candidacy_id) values (2, 1), (2, 6), (2, 7);

-- -- -- X no winner to new winner (chancellor now leia)
-- -- -- X no winner to no winner because max vote tie (hutt)
-- -- truncate allocation;
-- -- -- luke votes for leia and against vader and palpatine
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (1, 1, 3, 50, 'FOR');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (1, 1, 4, 25, 'AGAINST');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (1, 1, 5, 25, 'AGAINST');

-- -- -- han does same, also votes for himself and against other two
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (2, 1, 3, 10, 'FOR');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (2, 1, 4, 10, 'AGAINST');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (2, 1, 5, 10, 'AGAINST');

-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (2, 2, 1, 40, 'FOR');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (2, 2, 6, 15, 'AGAINST');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (2, 2, 7, 15, 'AGAINST');

-- -- -- vader merely votes against leia
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (4, 1, 3, 100, 'AGAINST');

-- -- -- lando votes to perfectly balance out han
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (6, 2, 1, 40, 'AGAINST');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (6, 2, 6, 15, 'FOR');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (6, 2, 7, 15, 'FOR');

-- -- select * from candidacy_updated order by election_id, candidacy_id;
-- -- call perform_vote_update();
-- -- select * from candidacy order by election_id, candidacy_id;


-- -- -- X current winner to same winner because no filled (chancellor still leia)
-- -- -- X current winner to same winner because filled tie (hutt)
-- -- truncate allocation;
-- -- -- arbitrarily make jabba hutt winner for test
-- -- update candidacy set stabilization_bucket = null where election_id = 2 and candidacy_id = 7;

-- -- -- palpatine slightly overtakes leia but not enough to fill, because luke and han drop those votes
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (3, 1, 3, 100, 'FOR');

-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (4, 1, 5, 50, 'FOR');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (5, 1, 5, 50, 'FOR');

-- -- -- both han and lando overtake jabba enough to fill, but they're tied
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (1, 2, 1, 100, 'FOR');
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (6, 2, 6, 100, 'FOR');

-- -- select * from candidacy_updated order by election_id, candidacy_id;
-- -- call perform_vote_update();
-- -- select * from candidacy order by election_id, candidacy_id;


-- -- -- X current winner to new winner because filled (hutt now han)
-- -- truncate allocation;
-- -- -- han has still filled, lando doesn't keep up
-- -- insert into allocation (voter_id, election_id, candidacy_id, weight, kind) values (1, 2, 1, 100, 'FOR');

-- -- select * from candidacy_updated order by election_id, candidacy_id;
-- -- call perform_vote_update();
-- -- select * from candidacy order by election_id, candidacy_id;
