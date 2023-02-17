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
create table candidate (
	id uuid primary key default gen_random_uuid(),
	-- OFFICE candidate owners are the candidate themselves
	owner_id uuid not null references person(id),

	election_id uuid not null,
	kind election_kind not null,
	foreign key (election_id, kind) references election(id, kind),

	unique (id, kind),

	-- enacted_during tstzmultirange not null default tstzmultirange(),

	"content" text not null
);
create unique index idx_unique_office_candidate on candidate(owner_id, election_id) where kind = 'OFFICE';

alter table election add constraint election_defining_document_fk
foreign key (defining_document_id, _defining_document_kind) references candidate(id, kind);


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
	candidate_id uuid not null references candidate(id),
	unique (voter_id, occurred_at, candidate_id),

	weight numeric not null check (weight != 0)
);

-- TODO constraint to make sure weight doesn't exceed max for voter_id


create type new_allocation as (
	candidate_id uuid,
	weight numeric
);

create or replace function perform_allocation_update(allocating_voter_id uuid, new_allocations new_allocation[])
returns table(voter_id uuid, occurred_at timestamptz, candidate_id uuid, weight numeric)
volatile
language sql as $$

	with
	current_update as (
		insert into allocation_update (voter_id) values (allocating_voter_id)
		returning occurred_at
	)
	insert into allocation (voter_id, occurred_at, candidate_id, weight)
	select
		allocating_voter_id, current_update.occurred_at, new_allocation.candidate_id, new_allocation.weight
	from current_update, unnest(new_allocations) as new_allocation
	returning *

$$;

-- -- TODO make this concurrency safe https://www.cybertec-postgresql.com/en/triggers-to-enforce-constraints/
-- create or replace function allocation_weight_valid() returns trigger as
-- $$
-- begin
-- 	if (new.weight + (select coalesce(sum(weight), 0) from allocation where voter_id = new.voter_id)) <= 100 then
-- 		return new;
-- 	else
-- 		raise exception 'allocated too much weight';
-- 	end if;
-- end;
-- $$ language plpgsql;

-- create trigger check_allocation_weight_valid
-- before insert or update on allocation
-- for each row
-- execute procedure allocation_weight_valid();


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

create table candidate_vote_update (
	occurred_at timestamptz not null references vote_update(occurred_at),
	candidate_id uuid not null references candidate(id),
	primary key (occurred_at, candidate_id),

	stabilization_bucket numeric default 0 check (stabilization_bucket >= 0)
);



create or replace function perform_vote_update()
-- returns table(voter_id uuid, occurred_at timestamptz, candidate_id uuid, weight numeric)
volatile
language sql as $$

	with
	,

	,

	current_update as (
		insert into vote_update () values ()
		returning occurred_at
	),

	insert into candidate_vote_update
	select
		current_update.occurred_at,
		candidate_id,
		most_recent_update.stabilization_bucket + ,
	from
		current_update, most_recent_update
	where occurred_at = most_recent_update.occurred_at

$$;



create view most_recent_update as
select max(occurred_at) as occurred_at from vote_update;


-- TODO give this an argument of date to consider?
create view global_most_recent_allocation as
select voter_id, max(occurred_at) as occurred_at from allocation_update
where occurred_at < most_recent_update.occurred_at
group by voter_id;

-- for all these tables there's the "current" state (max(occurred_at) where occurred_at < most_recent_update.occurred_at)
-- and the "next" state (max(occurred_at))


create view most_recent_allocation as
select voter_id, max(occurred_at) as occurred_at from allocation_update
group by voter_id;

create view current_vote as
select
	election_id, candidate_id,
	coalesce(sum(sign(weight) * sqrt(abs(weight))), 0) as total
from
	allocation
	join most_recent_allocation
		on allocation.voter_id = most_recent_allocation.voter_id
		and allocation.occurred_at = most_recent_allocation.occurred_at
	-- start from candidate instead?
	right join candidate
		on allocation.candidate_id = candidate.id
group by election_id, candidate_id;







create view candidacy_votes as
select
	election_id, candidate_id, stabilization_bucket,
	coalesce(sum(sign(weight) * sqrt(abs(weight))), 0) as total_vote
from
	candidate
	left join allocation using (election_id, candidate_id)
group by election_id, candidate_id, stabilization_bucket;

create view current_winner as
select election_id, candidate_id, total_vote
from candidacy_votes
where stabilization_bucket is null;

create view candidacy_updated as
select
	c.election_id, c.candidate_id, c.total_vote,

	case
		-- current candidate is the winner, will be elsewhere updated to 0 if overtaken
		when c.stabilization_bucket is null then null
		-- current candidate isn't the winner
		else greatest(
			-- if the candidate has never overtaken the winner, their total_vote will be 0 and the greatest op keeps them there
			-- if the candidate has overtaken the winner either now or previously, then this difference op makes sense
			-- (will be positive when candidate is ahead, will be negative if they've fallen behind)
			c.stabilization_bucket + (c.total_vote - current_winner.total_vote),
			-- NOTE the fill amount should be designed to make sense when a raw difference is taken
			0
		)
	end as stabilization_bucket
from
	candidacy_votes as c
	left join current_winner using (election_id);

create view next_candidacy_values as
with
election_maxes as (
	select
		election_id,
		-- if all candidates are negative or zero, then:
		-- - if there isn't a current winner the new current winner must have non-negative votes
		-- - if there is a current winner who is negative and someone is less negative then the less negative should fill
		-- - if there is a current winner who is zero and everyone is negative or zero then no one should fill
		max(total_vote) as max_votes,
		-- TODO arbitrary fill requirement for now
		max(case when stabilization_bucket is not null and stabilization_bucket >= 10 then stabilization_bucket else null end) as max_bucket

	from candidacy_updated
	group by election_id
),

max_filled as (
	select c.election_id, stabilization_bucket, count(candidate_id) as num_candidates
	from
		candidacy_updated as c
		join election_maxes as m on c.election_id = m.election_id and c.stabilization_bucket = m.max_bucket
	group by c.election_id, stabilization_bucket
),

max_votes as (
	select c.election_id, total_vote, count(candidate_id) as num_candidates
	from
		candidacy_updated as c
		join election_maxes as m on c.election_id = m.election_id and c.total_vote = m.max_votes
	group by c.election_id, total_vote
)

select
	c.election_id, c.candidate_id,
	case
		-- there's no current winner
		when current_winner.total_vote is null then
			case
				-- this row uniquely has max non-negative votes
				when max_votes.num_candidates = 1 and max_votes.total_vote >= 0 and c.total_vote = max_votes.total_vote then null
				-- there's a tie, so we simply "do nothing", making no one the winner yet and keeping all the buckets at 0
				else 0
			end

		-- there's a new unique winner
		when max_filled.num_candidates = 1 and max_filled.stabilization_bucket is not null then
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
	candidacy_updated as c
	left join current_winner using (election_id)
	left join max_votes using (election_id)
	left join max_filled using (election_id);


create procedure perform_vote_update()
language sql as $$
update candidacy as c
set stabilization_bucket = n.stabilization_bucket
from next_candidacy_values as n
where c.election_id = n.election_id and c.candidate_id = n.candidate_id
$$;








































insert into election (id, kind, defining_document_id, title, description) values
('ffffffffffffffffffffffffffffffff', 'DOCUMENT', null, 'root constitution', 'root constitution');


insert into person (id, "name") values
('11111111111111111111111111111111', 'Han'),
('22222222222222222222222222222222', 'Luke'),
('33333333333333333333333333333333', 'Leia'),
('44444444444444444444444444444444', 'Lando'),
('55555555555555555555555555555555', 'Ackbar'),
('66666666666666666666666666666666', 'Mothma');


insert into candidate (id, kind, owner_id, election_id, "content") values
('11111111111111111111111111111111', 'DOCUMENT', '11111111111111111111111111111111', 'ffffffffffffffffffffffffffffffff', 'Han cand'),
('22222222222222222222222222222222', 'DOCUMENT', '22222222222222222222222222222222', 'ffffffffffffffffffffffffffffffff', 'Luke cand');


select * from perform_allocation_update('11111111111111111111111111111111', ARRAY[
	row('11111111111111111111111111111111', 50)::new_allocation,
	row('22222222222222222222222222222222', -50)::new_allocation
]);



-- insert into person (id, "name") values (1, 'han'), (2, 'luke'), (3, 'leia'), (4, 'vader'), (5, 'palpatine'), (6, 'lando'), (7, 'jabba');
-- insert into election (id, title) values (1, 'chancellor'), (2, 'hutt');

-- -- for chancellor: leia, vader, palpatine
-- insert into candidacy (election_id, candidate_id) values (1, 3), (1, 4), (1, 5);
-- -- for hutt: han, lando, jabba
-- insert into candidacy (election_id, candidate_id) values (2, 1), (2, 6), (2, 7);

-- -- X no winner to new winner (chancellor now leia)
-- -- X no winner to no winner because max vote tie (hutt)
-- truncate allocation;
-- -- luke votes for leia and against vader and palpatine
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (1, 1, 3, 50, 'FOR');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (1, 1, 4, 25, 'AGAINST');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (1, 1, 5, 25, 'AGAINST');

-- -- han does same, also votes for himself and against other two
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (2, 1, 3, 10, 'FOR');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (2, 1, 4, 10, 'AGAINST');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (2, 1, 5, 10, 'AGAINST');

-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (2, 2, 1, 40, 'FOR');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (2, 2, 6, 15, 'AGAINST');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (2, 2, 7, 15, 'AGAINST');

-- -- vader merely votes against leia
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (4, 1, 3, 100, 'AGAINST');

-- -- lando votes to perfectly balance out han
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (6, 2, 1, 40, 'AGAINST');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (6, 2, 6, 15, 'FOR');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (6, 2, 7, 15, 'FOR');

-- select * from candidacy_updated order by election_id, candidate_id;
-- call perform_vote_update();
-- select * from candidacy order by election_id, candidate_id;


-- -- X current winner to same winner because no filled (chancellor still leia)
-- -- X current winner to same winner because filled tie (hutt)
-- truncate allocation;
-- -- arbitrarily make jabba hutt winner for test
-- update candidacy set stabilization_bucket = null where election_id = 2 and candidate_id = 7;

-- -- palpatine slightly overtakes leia but not enough to fill, because luke and han drop those votes
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (3, 1, 3, 100, 'FOR');

-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (4, 1, 5, 50, 'FOR');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (5, 1, 5, 50, 'FOR');

-- -- both han and lando overtake jabba enough to fill, but they're tied
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (1, 2, 1, 100, 'FOR');
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (6, 2, 6, 100, 'FOR');

-- select * from candidacy_updated order by election_id, candidate_id;
-- call perform_vote_update();
-- select * from candidacy order by election_id, candidate_id;


-- -- X current winner to new winner because filled (hutt now han)
-- truncate allocation;
-- -- han has still filled, lando doesn't keep up
-- insert into allocation (voter_id, election_id, candidate_id, weight, kind) values (1, 2, 1, 100, 'FOR');

-- select * from candidacy_updated order by election_id, candidate_id;
-- call perform_vote_update();
-- select * from candidacy order by election_id, candidate_id;
