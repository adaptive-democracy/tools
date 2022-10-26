drop schema public cascade;
create schema public;
grant all on schema public to public;
comment on schema public is 'standard public schema';


create table person (
	id serial primary key,
	"name" text not null
);

-- create table document (
-- 	id serial primary key,
-- 	"text" text not null
-- );

create table election (
	id serial primary key,
	title text not null
);

create table candidacy (
	election_id int references election(id) not null,
	candidate_id int references person(id) not null,
	stabilization_bucket numeric default 0 check (stabilization_bucket >= 0),

	primary key (election_id, candidate_id)
);

create unique index unique_winner
on candidacy(election_id)
where stabilization_bucket is null;

create type allocation_type as enum('FOR', 'AGAINST');
create table allocation (
	voter_id int references person(id) not null,
	election_id int not null,
	candidate_id int not null,

	weight int not null check (weight > 0),
	"type" allocation_type not null default 'FOR',

	-- every allocation must reference a valid candidacy
	foreign key (election_id, candidate_id) references candidacy(election_id, candidate_id),

	-- each voter can only allocate to each candidacy once
	primary key (voter_id, election_id, candidate_id)
);

create index index_allocation_voter_id on allocation(voter_id);

-- TODO make this concurrency safe https://www.cybertec-postgresql.com/en/triggers-to-enforce-constraints/
create or replace function allocation_weight_valid() returns trigger as
$$
begin
	if (new.weight + (select coalesce(sum(weight), 0) from allocation where voter_id = new.voter_id)) <= 100 then
		return new;
	else
		raise exception 'allocated too much weight';
	end if;
end;
$$ language plpgsql;

create trigger check_allocation_weight_valid
before insert or update on allocation
for each row
execute procedure allocation_weight_valid();


insert into person (id, "name") values (1, 'han'), (2, 'luke'), (3, 'leia'), (4, 'vader');
insert into election (id, title) values (1, 'leader');
insert into candidacy (election_id, candidate_id) values (1, 3), (1, 4);


-- valid allocations
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (1, 1, 3, 50, 'FOR');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (2, 1, 3, 50, 'FOR');

insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (1, 1, 4, 50, 'AGAINST');
insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (2, 1, 4, 50, 'AGAINST');

insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (4, 1, 3, 100, 'AGAINST');
-- -- invalid allocations
-- insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (4, 1, 4, 500, 'FOR');
-- insert into allocation (voter_id, election_id, candidate_id, weight, "type") values (4, 1, 4, 1, 'FOR');




-- -- candidacy totals
-- select
-- 	election_id, election.title as election_title, candidate_id, person.name as candidate_name,
-- 	sum((case when allocation."type" = 'FOR' then 1 else -1 end) * sqrt(weight)) as total_vote

-- from
-- 	allocation
-- 	join election on allocation.election_id = election.id
-- 	join person on allocation.candidate_id = person.id
-- group by election_id, election_title, candidate_id, candidate_name
-- ;


with

candidacy_votes as (
	select
		election_id, candidate_id, stabilization_bucket,
		sum((case when allocation."type" = 'FOR' then 1 else -1 end) * sqrt(weight)) as total_vote
	from
		allocation
		join candidacy using (election_id, candidate_id)
	group by election_id, candidate_id, stabilization_bucket
),

current_winner as (
	select election_id, candidate_id, total_vote
	from candidacy_votes
	where stabilization_bucket is null
),

max_votes_in_election as (
	select election_id, max(total_vote) as max_votes
	from candidacy_votes
	group by election_id
),

max_votes as (
	select c.election_id, candidate_id, total_vote
	from
		candidacy_votes as c
		join max_votes_in_election as m on c.election_id = m.election_id and c.total_vote = m.max_votes
),

max_filled_in_election as (
	select election_id, max(stabilization_bucket) as max_bucket
	from candidacy_votes
	-- use arbitrary fill requirement for now
	where stabilization_bucket is not null and stabilization_bucket >= 10
	group by election_id
),

max_filled as (
	select c.election_id, candidate_id, stabilization_bucket
	from
		candidacy_votes as c
		join max_filled_in_election as m on c.election_id = m.election_id and c.stabilization_bucket = m.max_bucket
)

-- if there isn't a winner then merely whoever has the most votes becomes the new winner (this triggers a reset)
-- if there is a winner then
-- - if anyone has filled their stabilization_bucket then the most full bucket becomes the winner and all other buckets are set to 0
-- - else all stabilization_bucket are calculated, they increase or decrease by their difference with the winner

select
	c.election_id, c.candidate_id,
	case
		-- if there's no current winner
		when current_winner.total_vote is null then
			case
				-- this row has max votes
				when c.candidate_id = max_votes.candidate_id then null
				-- should we hard reset or use previous bucket value?
				else 0
			end

		-- if there's a new winner
		when max_filled.stabilization_bucket is not null then
			case
				-- this row is the new winner
				when c.stabilization_bucket = max_filled.stabilization_bucket then null
				-- this row is not the new winner
				else 0
			end

		-- otherwise just update
		else greatest(
			-- the fill amount should be designed to make sense when a raw difference is taken
			-- if the candidate has never overtaken the winner, their total_vote will be 0 and the greatest op keeps them there
			-- if the candidate has overtaken the winner, either now or previously, then this difference is meaningful
			c.stabilization_bucket + (c.total_vote - current_winner.total_vote),
			0
		)

	end as new_stabilization_bucket

from
	candidacy_votes as c
	left join current_winner using (election_id)
	left join max_votes using (election_id)
	left join max_filled using (election_id)




-- at each snapshot the current votes is merely the sum considering only this shapshot
-- the stabilization bucket status is what really determines the winner
-- the single candidate with a null stabilization bucket is the winner
-- a candidate who has never been in the lead necessarily has a 0 stabilization bucket
-- if a candidate has more votes than the null bucket their bucket increases according to the size of the difference
-- if any candidate fills their stabilization bucket then the candidate with the most full bucket becomes the new winner and all buckets are reset
-- for now we can simply assign each election a dummy "scale" parameter representing the fill size of the bucket

-- it seems true that we ignore the previous vote state? only the previous bucket state and the *current* vote state matters?








-- create function select_suggestions(suggestions suggestion[]) returns setof suggestion as $$
-- 	declare
-- 		current_record suggestion;
-- 		chosen_suggestion suggestion;
-- 		chosen_suggestions suggestion[];
-- 	begin
-- 		chosen_suggestions := array[]::suggestion[];

-- 		<<input_selections>>
-- 		foreach current_record in array suggestions loop
-- 			foreach chosen_suggestion in array chosen_suggestions loop
-- 				if chosen_suggestion.id = current_record.id or chosen_suggestion.label = current_record.label then
-- 					continue input_selections;
-- 				end if;
-- 			end loop;

-- 			chosen_suggestions := array_append(chosen_suggestions, current_record);
-- 		end loop;

-- 		return query
-- 			select
-- 				suggestions.id,
-- 				suggestions.label,
-- 				chosen_suggestions.label is not null as selected
-- 			from
-- 				unnest(suggestions) as suggestions(id, label)
-- 			left join
-- 				unnest(chosen_suggestions) as chosen_suggestions(id, label) on
-- 					chosen_suggestions.id = suggestions.id and
-- 					chosen_suggestions.label = suggestions.label;
-- 	end;
-- $$ language plpgsql;
