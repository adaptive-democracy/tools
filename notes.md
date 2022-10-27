```sql
-- create function select_suggestions(suggestions suggestion[]) returns setof suggestion as $$
--  declare
--    current_record suggestion;
--    chosen_suggestion suggestion;
--    chosen_suggestions suggestion[];
--  begin
--    chosen_suggestions := array[]::suggestion[];

--    <<input_selections>>
--    foreach current_record in array suggestions loop
--      foreach chosen_suggestion in array chosen_suggestions loop
--        if chosen_suggestion.id = current_record.id or chosen_suggestion.label = current_record.label then
--          continue input_selections;
--        end if;
--      end loop;

--      chosen_suggestions := array_append(chosen_suggestions, current_record);
--    end loop;

--    return query
--      select
--        suggestions.id,
--        suggestions.label,
--        chosen_suggestions.label is not null as selected
--      from
--        unnest(suggestions) as suggestions(id, label)
--      left join
--        unnest(chosen_suggestions) as chosen_suggestions(id, label) on
--          chosen_suggestions.id = suggestions.id and
--          chosen_suggestions.label = suggestions.label;
--  end;
-- $$ language plpgsql;


-- -- candidacy totals
-- select
--  election_id, election.title as election_title, candidate_id, person.name as candidate_name,
--  sum((case when allocation."type" = 'FOR' then 1 else -1 end) * sqrt(weight)) as total_vote

-- from
--  allocation
--  join election on allocation.election_id = election.id
--  join person on allocation.candidate_id = person.id
-- group by election_id, election_title, candidate_id, candidate_name
-- ;
```
