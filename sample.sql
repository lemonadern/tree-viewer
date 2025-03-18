select id, case grade
    when 'A' then 5
    when 'B' then 4
    when 'C' then 3
    else 0 end as p
;

