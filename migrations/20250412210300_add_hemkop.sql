ALTER TABLE ingredients
RENAME COLUMN axfood_category_name TO willys_category_name;


ALTER TABLE ingredients
ADD COLUMN hemkop_category_name TEXT;

UPDATE ingredients
SET hemkop_category_name = 'mejeri-ost-och-agg/agg/agg'
WHERE name = 'Ägg';

UPDATE ingredients
SET hemkop_category_name = 'skafferi/bakning/mjol'
WHERE name = 'Vetemjöl';

UPDATE ingredients
SET hemkop_category_name = 'skafferi/bakning/socker-och-honung'
WHERE name = 'Strösocker';

UPDATE ingredients
SET hemkop_category_name = 'skafferi/bakning/baktillbehor'
WHERE name = 'Kakao';

UPDATE ingredients
SET hemkop_category_name = 'mejeri-ost-och-agg/smor-margarin-och-jast/smor-och-margarin'
WHERE name = 'Smör';

ALTER TABLE ingredients 
ALTER COLUMN hemkop_category_name SET NOT NULL;
